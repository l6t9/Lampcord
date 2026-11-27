use std::sync::{Condvar, Mutex, OnceLock};
use std::thread::Builder;
use std::thread;

type Job = Box<dyn FnOnce() + Send>;

struct Pool {
    jobs: Mutex<Vec<Job>>,
    wake: Condvar,
}

fn pool() -> &'static Pool {
    static POOL: OnceLock<&'static Pool> = OnceLock::new();
    POOL.get_or_init(|| {
        let pool: &'static Pool = Box::leak(Box::new(Pool {
            jobs: Mutex::new(Vec::new()),
            wake: Condvar::new(),
        }));
        let workers = (thread::available_parallelism().map_or(4, |n| n.get()) / 2).clamp(2, 4);
        for index in 0..workers {
            let spawned = Builder::new()
                .name(format!("decode-{index}"))
                .spawn(move || {
                    loop {
                        let job = {
                            let mut jobs = pool.jobs.lock().expect("decode jobs poisoned");
                            loop {
                                if let Some(job) = jobs.pop() {
                                    break job;
                                }
                                jobs = pool.wake.wait(jobs).expect("decode jobs poisoned");
                            }
                        };
                        job();
                    }
                });
            if let Err(err) = spawned {
                log::warn!("could not start a decode worker: {err}");
            }
        }
        pool
    })
}

pub fn spawn(job: impl FnOnce() + Send + 'static) {
    let pool = pool();
    pool.jobs
        .lock()
        .expect("decode jobs poisoned")
        .push(Box::new(job));
    pool.wake.notify_one();
}
