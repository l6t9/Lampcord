use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::Error;
use std::process;
use std::thread;

const JOURNAL: &str = "pending.json";
const HEALTHY: &str = "healthy";
const REPORT: &str = "rolled-back.json";
const LOG: &str = "helper.log";

pub const HELPER_SUFFIX: &str = "-update-helper.exe";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Journal {
    pub version: String,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub target: PathBuf,

    pub old: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub version: String,
    pub reason: String,
}

fn other(msg: impl Into<String>) -> Error {
    Error::other(msg.into())
}

fn files_under(dir: &Path) -> io::Result<Vec<PathBuf>> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_dir() {
                walk(base, &path, out)?;
            } else {
                out.push(path.strip_prefix(base).expect("under base").to_path_buf());
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out)?;
    out.sort();
    Ok(out)
}

fn old_name(target: &Path) -> PathBuf {
    let name = target
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let first = target.with_file_name(format!("{name}.old"));
    if !first.exists() || fs::remove_file(&first).is_ok() {
        return first;
    }
    (1..)
        .map(|n| target.with_file_name(format!("{name}.old{n}")))
        .find(|p| !p.exists() || fs::remove_file(p).is_ok())
        .expect("some name is free")
}

fn move_file(from: &Path, to: &Path) -> io::Result<()> {
    if fs::rename(from, to).is_ok() {
        return Ok(());
    }
    fs::copy(from, to)?;
    let _ = fs::remove_file(from);
    Ok(())
}

pub fn swap(install: &Path, staged: &Path, version: &str) -> io::Result<Journal> {
    let mut journal = Journal {
        version: version.to_string(),
        entries: Vec::new(),
    };
    let result = (|| -> io::Result<()> {
        for rel in files_under(staged)? {
            let target = install.join(&rel);
            if let Some(dir) = target.parent() {
                fs::create_dir_all(dir)?;
            }
            let old = if target.exists() {
                let old = old_name(&target);
                fs::rename(&target, &old)
                    .map_err(|e| other(format!("cannot move {} aside: {e}", rel.display())))?;
                Some(
                    old.strip_prefix(install)
                        .expect("under install")
                        .to_path_buf(),
                )
            } else {
                None
            };

            journal.entries.push(Entry {
                target: rel.clone(),
                old,
            });
            move_file(&staged.join(&rel), &target)
                .map_err(|e| other(format!("cannot place {}: {e}", rel.display())))?;
        }
        Ok(())
    })();
    match result {
        Ok(()) => Ok(journal),
        Err(err) => {
            let _ = rollback(install, &journal);
            Err(err)
        }
    }
}

pub fn rollback(install: &Path, journal: &Journal) -> io::Result<()> {
    let mut first_error = None;
    for entry in journal.entries.iter().rev() {
        let target = install.join(&entry.target);
        let _ = fs::remove_file(&target);
        if let Some(old) = &entry.old {
            if let Err(e) = fs::rename(install.join(old), &target) {
                first_error.get_or_insert_with(|| {
                    other(format!("cannot restore {}: {e}", entry.target.display()))
                });
            }
        }
    }
    first_error.map_or(Ok(()), Err)
}

pub fn commit(install: &Path, journal: &Journal) {
    for entry in &journal.entries {
        if let Some(old) = &entry.old {
            let _ = fs::remove_file(install.join(old));
        }
    }
}

fn save_journal(root: &Path, journal: &Journal) -> io::Result<()> {
    fs::create_dir_all(root)?;
    fs::write(
        root.join(JOURNAL),
        serde_json::to_vec_pretty(journal).expect("journal serializes"),
    )
}

fn load_journal(root: &Path) -> Option<Journal> {
    serde_json::from_slice(&fs::read(root.join(JOURNAL)).ok()?).ok()
}

pub fn commit_pending(install: &Path, root: &Path) {
    if let Some(journal) = load_journal(root) {
        commit(install, &journal);
        let _ = fs::remove_file(root.join(JOURNAL));
    }
}

pub fn mark_healthy(root: &Path) {
    let _ = fs::create_dir_all(root);
    let _ = fs::write(root.join(HEALTHY), b"ok");
}

pub fn take_report(root: &Path) -> Option<Report> {
    let report = serde_json::from_slice(&fs::read(root.join(REPORT)).ok()?).ok();
    let _ = fs::remove_file(root.join(REPORT));
    report
}

pub fn sweep_helpers(install: &Path) {
    let Ok(entries) = fs::read_dir(install) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy().ends_with(HELPER_SUFFIX) {
            let _ = fs::remove_file(entry.path());
        }
    }
}

fn log(root: &Path, line: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join(LOG))
    {
        let _ = writeln!(file, "{line}");
    }
}

fn writable_now(path: &Path) -> bool {
    !path.exists() || OpenOptions::new().write(true).open(path).is_ok()
}

pub fn wait_until_free(path: &Path, timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if writable_now(path) {
            return true;
        }
        thread::sleep(Duration::from_millis(200));
    }
    writable_now(path)
}

#[derive(Debug, Clone)]
pub struct HelperArgs {
    pub install: PathBuf,
    pub staged: PathBuf,
    pub version: String,

    pub exe_name: String,

    pub app_args: Vec<String>,

    pub exit_timeout: Duration,
    pub health_timeout: Duration,
}

impl HelperArgs {
    pub fn new(install: PathBuf, staged: PathBuf, version: String, exe_name: String) -> Self {
        Self {
            install,
            staged,
            version,
            exe_name,
            app_args: vec!["--post-update".into()],
            exit_timeout: Duration::from_secs(60),
            health_timeout: Duration::from_secs(120),
        }
    }

    pub fn root(&self) -> PathBuf {
        self.staged
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.install.join(".update"))
    }
}

fn start(install: &Path, exe_name: &str, args: &[String]) -> io::Result<process::Child> {
    Command::new(install.join(exe_name))
        .args(args)
        .current_dir(install)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}

pub fn run_helper(args: &HelperArgs) -> i32 {
    let root = args.root();
    let exe = args.install.join(&args.exe_name);
    log(&root, &format!("updating to {}", args.version));

    if !wait_until_free(&exe, args.exit_timeout) {
        log(&root, "the old version is still running; giving up");
        return 2;
    }
    let journal = match swap(&args.install, &args.staged, &args.version) {
        Ok(journal) => journal,
        Err(err) => {
            log(&root, &format!("swap failed: {err}"));
            report(
                &root,
                &args.version,
                &format!("could not replace the files: {err}"),
            );
            let _ = start(&args.install, &args.exe_name, &[]);
            return 3;
        }
    };
    let _ = save_journal(&root, &journal);
    let _ = fs::remove_file(root.join(HEALTHY));

    let undo = |reason: String| {
        log(&root, &format!("undoing the update: {reason}"));
        if let Err(e) = rollback(&args.install, &journal) {
            log(&root, &format!("rollback was incomplete: {e}"));
        }
        let _ = fs::remove_file(root.join(JOURNAL));
        report(&root, &args.version, &reason);
        let _ = start(&args.install, &args.exe_name, &[]);
    };

    let mut child = match start(&args.install, &args.exe_name, &args.app_args) {
        Ok(child) => child,
        Err(e) => {
            undo(format!("the new version would not start: {e}"));
            return 4;
        }
    };
    let begun = Instant::now();
    while begun.elapsed() < args.health_timeout {
        if root.join(HEALTHY).exists() {
            commit(&args.install, &journal);
            let _ = fs::remove_file(root.join(JOURNAL));
            log(&root, "the new version is healthy; done");
            return 0;
        }
        if let Ok(Some(status)) = child.try_wait() {
            if root.join(HEALTHY).exists() {
                commit(&args.install, &journal);
                let _ = fs::remove_file(root.join(JOURNAL));
                return 0;
            }
            undo(format!(
                "the new version exited before it was healthy ({status})"
            ));
            return 4;
        }
        thread::sleep(Duration::from_millis(250));
    }

    log(
        &root,
        "no health marker in time, but the new version is running; leaving the journal for the app",
    );
    0
}

fn report(root: &Path, version: &str, reason: &str) {
    let report = Report {
        version: version.to_string(),
        reason: reason.to_string(),
    };
    let _ = fs::write(
        root.join(REPORT),
        serde_json::to_vec_pretty(&report).expect("report serializes"),
    );
}

pub fn launch_helper(
    install: &Path,
    staged: &Path,
    version: &str,
    exe_name: &str,
) -> io::Result<()> {
    let me = env::current_exe()?;
    let helper = install.join(format!(
        "{}{HELPER_SUFFIX}",
        exe_name.trim_end_matches(".exe")
    ));
    sweep_helpers(install);
    fs::copy(&me, &helper)?;
    Command::new(&helper)
        .arg("--apply-update")
        .arg(install)
        .arg(staged)
        .arg(version)
        .arg(exe_name)
        .current_dir(install)
        .spawn()?;
    Ok(())
}

pub fn helper_args_from(args: &[String]) -> Option<HelperArgs> {
    let [install, staged, version, exe_name] = args else {
        return None;
    };
    Some(HelperArgs::new(
        install.into(),
        staged.into(),
        version.clone(),
        exe_name.clone(),
    ))
}
