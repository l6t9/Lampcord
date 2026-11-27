use std::alloc;
use std::cell::Cell;
use std::cell::RefCell;
use std::cmp::Reverse;
use std::cmp::Ordering as CmpOrdering;
use std::collections::HashMap;
use std::env;
#[cfg(windows)]
use std::mem;
#[cfg(windows)]
use std::ptr;
use std::sync::OnceLock;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Instant;
#[cfg(windows)]
use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW;
#[cfg(windows)]
use windows_sys::Win32::System::Memory::GetProcessHeaps;
#[cfg(windows)]
use windows_sys::Win32::System::Memory::HEAP_SUMMARY;
#[cfg(windows)]
use windows_sys::Win32::System::Memory::HeapSummary;
#[cfg(windows)]
use windows_sys::Win32::System::Memory::MEMORY_BASIC_INFORMATION;
#[cfg(windows)]
use windows_sys::Win32::System::Memory::MEM_COMMIT;
#[cfg(windows)]
use windows_sys::Win32::System::Memory::MEM_IMAGE;
#[cfg(windows)]
use windows_sys::Win32::System::Memory::MEM_MAPPED;
#[cfg(windows)]
use windows_sys::Win32::System::Memory::MEM_PRIVATE;
#[cfg(windows)]
use windows_sys::Win32::System::Memory::VirtualQuery;
#[cfg(windows)]
use windows_sys::Win32::System::ProcessStatus::GetProcessMemoryInfo;
#[cfg(windows)]
use windows_sys::Win32::System::ProcessStatus::K32QueryWorkingSetEx;
#[cfg(windows)]
use windows_sys::Win32::System::ProcessStatus::PROCESS_MEMORY_COUNTERS_EX;
#[cfg(windows)]
use windows_sys::Win32::System::ProcessStatus::PSAPI_WORKING_SET_EX_INFORMATION;
#[cfg(windows)]
use windows_sys::Win32::System::Threading::GetCurrentProcess;

#[derive(Clone, Copy, Default, Debug)]
pub struct Counters {
    pub cascade_count: u32,
    pub cascade_ms: f64,
    pub flush_ms: f64,
    pub layout_ms: f64,
    pub paint_ms: f64,
    pub present_ms: f64,

    pub full_frames: u32,
    pub partial_frames: u32,

    pub worst_frame_ms: f64,
    pub over_budget_frames: u32,
}

thread_local! {
    static COUNTERS: Cell<Counters> = const { Cell::new(Counters { cascade_count: 0, cascade_ms: 0.0, flush_ms: 0.0, layout_ms: 0.0, paint_ms: 0.0, present_ms: 0.0, full_frames: 0, partial_frames: 0, worst_frame_ms: 0.0, over_budget_frames: 0 }) };

    static FLUSH_SINCE_FRAME: Cell<f64> = const { Cell::new(0.0) };
    static BUDGET_MS: Cell<f64> = const { Cell::new(1000.0 / 144.0) };
}

pub fn set_frame_budget_ms(ms: f64) {
    BUDGET_MS.with(|b| b.set(ms.max(1.0)));
}

pub fn note_frame_work(render_ms: f64) {
    let work = render_ms + FLUSH_SINCE_FRAME.with(|f| f.replace(0.0));
    let budget = BUDGET_MS.with(Cell::get);
    COUNTERS.with(|c| {
        let mut counters = c.get();
        counters.worst_frame_ms = counters.worst_frame_ms.max(work);
        if work > budget {
            counters.over_budget_frames += 1;
        }
        c.set(counters);
    });
}

#[derive(Clone, Copy)]
pub enum Phase {
    Cascade,
    Flush,
    Layout,
    Paint,
    Present,
}

pub fn time<R>(phase: Phase, f: impl FnOnce() -> R) -> R {
    let start = Instant::now();
    let result = f();
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    COUNTERS.with(|c| {
        let mut counters = c.get();
        match phase {
            Phase::Cascade => {
                counters.cascade_count += 1;
                counters.cascade_ms += ms;
            }
            Phase::Flush => {
                counters.flush_ms += ms;
                FLUSH_SINCE_FRAME.with(|f| f.set(f.get() + ms));
            }
            Phase::Layout => counters.layout_ms += ms,
            Phase::Paint => counters.paint_ms += ms,
            Phase::Present => counters.present_ms += ms,
        }
        c.set(counters);
    });
    result
}

pub fn count_frame(full: bool) {
    COUNTERS.with(|c| {
        let mut counters = c.get();
        if full {
            counters.full_frames += 1;
        } else {
            counters.partial_frames += 1;
        }
        c.set(counters);
    });
}

pub fn take() -> Counters {
    COUNTERS.with(|c| c.replace(Counters::default()))
}

pub fn process_memory() -> Option<(usize, usize)> {
    #[cfg(windows)]
    {
        // SAFETY: plain Win32 calls: `counters` is a zero-initialised struct whose size is stored
        // in `cb`, and the pseudo-handle for the current process is always valid.
        unsafe {
            let mut counters: PROCESS_MEMORY_COUNTERS_EX = mem::zeroed();
            counters.cb = size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
            if GetProcessMemoryInfo(
                GetCurrentProcess(),
                (&mut counters as *mut PROCESS_MEMORY_COUNTERS_EX).cast(),
                counters.cb,
            ) != 0
            {
                return Some((counters.WorkingSetSize, counters.PrivateUsage));
            }
        }
    }
    None
}

thread_local! {
    static MEM_PROBES: RefCell<Vec<Box<dyn Fn() -> String>>> = const { RefCell::new(Vec::new()) };
}

pub fn register_mem_probe(probe: impl Fn() -> String + 'static) {
    MEM_PROBES.with(|probes| probes.borrow_mut().push(Box::new(probe)));
}

pub fn mem_probe_reports() -> Vec<String> {
    MEM_PROBES.with(|probes| probes.borrow().iter().map(|probe| probe()).collect())
}

pub fn mb(bytes: usize) -> String {
    format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
}

pub struct CountingAlloc;

static HEAP_BYTES: AtomicUsize = AtomicUsize::new(0);

// SAFETY: every method forwards to `System` with the arguments it was given, so `System`'s
// allocator contract carries over; the counters have no effect on the returned memory.
unsafe impl alloc::GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: alloc::Layout) -> *mut u8 {
        // SAFETY: the caller upholds `GlobalAlloc::alloc`'s contract for `layout`.
        let ptr = unsafe { alloc::System.alloc(layout) };
        if !ptr.is_null() {
            HEAP_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: alloc::Layout) {
        // SAFETY: the caller upholds `GlobalAlloc::dealloc`'s contract: `ptr` came from this
        // allocator with `layout`.
        unsafe { alloc::System.dealloc(ptr, layout) };
        HEAP_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
    }
    unsafe fn alloc_zeroed(&self, layout: alloc::Layout) -> *mut u8 {
        // SAFETY: the caller upholds `GlobalAlloc::alloc_zeroed`'s contract for `layout`.
        let ptr = unsafe { alloc::System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            HEAP_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: alloc::Layout, new_size: usize) -> *mut u8 {
        // SAFETY: the caller upholds `GlobalAlloc::realloc`'s contract: `ptr` came from this
        // allocator with `layout`, and `new_size` is valid.
        let new_ptr = unsafe { alloc::System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            HEAP_BYTES.fetch_add(new_size, Ordering::Relaxed);
            HEAP_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
        }
        new_ptr
    }
}

pub fn heap_bytes() -> usize {
    HEAP_BYTES.load(Ordering::Relaxed)
}

pub fn win_heap_bytes() -> Option<usize> {
    #[cfg(windows)]
    {
        // SAFETY: `heaps` is a buffer of the length passed to GetProcessHeaps, only the first `n`
        // entries (clamped to its length) are read, and `summary` is a zero-initialised struct
        // whose size is stored in `cb`.
        unsafe {
            let mut heaps = [ptr::null_mut(); 256];
            let n = GetProcessHeaps(heaps.len() as u32, heaps.as_mut_ptr()) as usize;
            let mut total = 0usize;
            for &heap in heaps.iter().take(n.min(heaps.len())) {
                let mut summary: HEAP_SUMMARY = mem::zeroed();
                summary.cb = size_of::<HEAP_SUMMARY>() as u32;
                if HeapSummary(heap, 0, &mut summary) != 0 {
                    total += summary.cbAllocated;
                }
            }
            return Some(total);
        }
    }
    #[allow(unreachable_code)]
    None
}

pub fn committed_by_type() -> Option<(usize, usize, usize)> {
    #[cfg(windows)]
    {
        let (mut image, mut mapped, mut private) = (0usize, 0usize, 0usize);
        let mut address = 0usize;

        // SAFETY: `info` is a zero-initialised out-struct whose size is passed to VirtualQuery,
        // which only reads the address and writes `info`.
        unsafe {
            loop {
                let mut info: MEMORY_BASIC_INFORMATION = mem::zeroed();
                if VirtualQuery(
                    address as *const _,
                    &mut info,
                    size_of::<MEMORY_BASIC_INFORMATION>(),
                ) == 0
                {
                    break;
                }
                if info.State == MEM_COMMIT {
                    match info.Type {
                        MEM_IMAGE => image += info.RegionSize,
                        MEM_MAPPED => mapped += info.RegionSize,
                        MEM_PRIVATE => private += info.RegionSize,
                        _ => {}
                    }
                }
                let next = (info.BaseAddress as usize).wrapping_add(info.RegionSize);
                if next <= address {
                    break;
                }
                address = next;
            }
        }
        return Some((image, mapped, private));
    }
    #[allow(unreachable_code)]
    None
}

pub fn working_set_breakdown() -> Option<String> {
    #[cfg(windows)]
    {
        const PAGE: usize = 4096;
        let (mut mapped, mut private) = (0usize, 0usize);
        let mut modules: HashMap<usize, usize> = HashMap::new();
        let mut address = 0usize;

        // SAFETY: `info` is a zero-initialised out-struct whose size is passed to VirtualQuery, and
        // the process handle is the always-valid current-process pseudo-handle.
        unsafe {
            let process = GetCurrentProcess();
            loop {
                let mut info: MEMORY_BASIC_INFORMATION = mem::zeroed();
                if VirtualQuery(
                    address as *const _,
                    &mut info,
                    size_of::<MEMORY_BASIC_INFORMATION>(),
                ) == 0
                {
                    break;
                }
                if info.State == MEM_COMMIT {
                    let base = info.BaseAddress as usize;
                    let pages = info.RegionSize / PAGE;
                    let mut resident = 0usize;
                    let mut start = 0usize;
                    while start < pages {
                        let n = (pages - start).min(4096);
                        let mut entries: Vec<PSAPI_WORKING_SET_EX_INFORMATION> = (0..n)
                            .map(|i| PSAPI_WORKING_SET_EX_INFORMATION {
                                VirtualAddress: (base + (start + i) * PAGE) as *mut _,
                                VirtualAttributes: mem::zeroed(),
                            })
                            .collect();
                        if K32QueryWorkingSetEx(
                            process,
                            entries.as_mut_ptr().cast(),
                            (n * size_of::<PSAPI_WORKING_SET_EX_INFORMATION>()) as u32,
                        ) != 0
                        {
                            resident += entries
                                .iter()
                                .filter(|e| (e.VirtualAttributes.Flags & 1) != 0)
                                .count()
                                * PAGE;
                        }
                        start += n;
                    }
                    match info.Type {
                        MEM_IMAGE => {
                            *modules.entry(info.AllocationBase as usize).or_default() += resident
                        }
                        MEM_MAPPED => mapped += resident,
                        MEM_PRIVATE => private += resident,
                        _ => {}
                    }
                }
                let next = (info.BaseAddress as usize).wrapping_add(info.RegionSize);
                if next <= address {
                    break;
                }
                address = next;
            }
            let mut named: Vec<(usize, String)> = modules
                .into_iter()
                .map(|(base, bytes)| {
                    let mut buf = [0u16; 260];
                    let len = GetModuleFileNameW(base as _, buf.as_mut_ptr(), 260) as usize;
                    let name = String::from_utf16_lossy(&buf[..len]);
                    (bytes, name.rsplit('\\').next().unwrap_or("?").to_string())
                })
                .collect();
            named.sort_unstable_by_key(|a| Reverse(a.0));
            let image_total: usize = named.iter().map(|(b, _)| b).sum();
            let top: Vec<String> = named
                .iter()
                .take(8)
                .map(|(b, n)| format!("{n} {}", mb(*b)))
                .collect();
            return Some(format!(
                "resident: private {} mapped {} dlls {} [{}]",
                mb(private),
                mb(mapped),
                mb(image_total),
                top.join(", ")
            ));
        }
    }
    #[allow(unreachable_code)]
    None
}

thread_local! {
    static WIDGET_PAINT: RefCell<HashMap<String, (f64, u32)>> = RefCell::new(HashMap::new());
}

pub fn profile_paint() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| env::var("UI_PROFILE_PAINT").is_ok())
}

pub fn record_widget_paint(name: String, ms: f64) {
    WIDGET_PAINT.with(|m| {
        let mut m = m.borrow_mut();
        let entry = m.entry(name).or_insert((0.0, 0));
        entry.0 += ms;
        entry.1 += 1;
    });
}

pub fn take_widget_paint_report() -> Option<String> {
    WIDGET_PAINT.with(|m| {
        let mut m = m.borrow_mut();
        if m.is_empty() {
            return None;
        }
        let mut rows: Vec<(String, f64, u32)> = m.drain().map(|(k, (ms, n))| (k, ms, n)).collect();
        rows.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(CmpOrdering::Equal));
        let plain = rows
            .iter()
            .filter(|r| !r.0.starts_with("SIGNAL-SET"))
            .take(8)
            .map(|(k, ms, n)| format!("{k} {ms:.1}ms/{n}"));
        let mut signals: Vec<&(String, f64, u32)> = rows
            .iter()
            .filter(|r| r.0.starts_with("SIGNAL-SET"))
            .collect();
        signals.sort_by_key(|a| Reverse(a.2));
        let signals = signals
            .iter()
            .take(12)
            .map(|(k, _, n)| format!("{} x{n}", k.trim_start_matches("SIGNAL-SET ")));
        Some(format!(
            "{} || signals set: {}",
            plain.collect::<Vec<_>>().join(" | "),
            signals.collect::<Vec<_>>().join(", ")
        ))
    })
}
