use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub const LOW_MB: u64 = 2048;
pub const RESUME_MB: u64 = 3072;
pub const WAIT_LIMIT: Duration = Duration::from_secs(15 * 60);
pub const TICK: Duration = Duration::from_secs(1);
const REPORT_EVERY: u32 = 5;

static RESUME_NOW: AtomicBool = AtomicBool::new(false);

pub fn resume_now() {
    RESUME_NOW.store(true, Ordering::SeqCst);
}

fn take_resume() -> bool {
    RESUME_NOW.swap(false, Ordering::SeqCst)
}

#[cfg(windows)]
pub fn free_mb() -> Option<u64> {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut status = MEMORYSTATUSEX { dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32, dwMemoryLoad: 0, ullTotalPhys: 0, ullAvailPhys: 0, ullTotalPageFile: 0, ullAvailPageFile: 0, ullTotalVirtual: 0, ullAvailVirtual: 0, ullAvailExtendedVirtual: 0 };
    let ok = unsafe { GlobalMemoryStatusEx(&mut status) } != 0;
    ok.then(|| status.ullAvailPhys.min(status.ullAvailPageFile) / (1024 * 1024))
}

#[cfg(not(windows))]
pub fn free_mb() -> Option<u64> {
    None
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Wait {
    NotNeeded,
    Freed,
    Resumed,
    TimedOut,
}

pub struct Probe<'a> {
    pub read: &'a dyn Fn() -> Option<u64>,
    pub sleep: &'a dyn Fn(Duration),
    pub resume: &'a dyn Fn() -> bool,
}

pub fn system_probe() -> Probe<'static> {
    Probe { read: &free_mb, sleep: &std::thread::sleep, resume: &take_resume }
}

pub fn low(probe: &Probe) -> Option<u64> {
    (probe.read)().filter(|free| *free < LOW_MB)
}

pub fn wait(probe: &Probe, on_wait: &dyn Fn(u64)) -> Wait {
    let Some(first) = low(probe) else { return Wait::NotNeeded };
    let _ = (probe.resume)();
    log::warn!("only {first} MB free; waiting for {RESUME_MB} MB");
    on_wait(first);
    let mut waited = Duration::ZERO;
    let mut ticks = 0;
    while waited < WAIT_LIMIT {
        (probe.sleep)(TICK);
        waited += TICK;
        ticks += 1;
        if (probe.resume)() {
            log::info!("resumed by hand after {} s", waited.as_secs());
            return Wait::Resumed;
        }
        let free = (probe.read)().unwrap_or(RESUME_MB);
        if free >= RESUME_MB {
            log::info!("{free} MB free after {} s; going on", waited.as_secs());
            return Wait::Freed;
        }
        if ticks % REPORT_EVERY == 0 {
            on_wait(free);
        }
    }
    log::error!("still short of memory after {} s", waited.as_secs());
    Wait::TimedOut
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use super::*;

    fn run(readings: Vec<u64>, resume_at: Option<usize>) -> (Wait, Vec<u64>, usize) {
        let calls = Cell::new(0usize);
        let readings = RefCell::new(readings.into_iter());
        let last = Cell::new(0);
        let read = || {
            calls.set(calls.get() + 1);
            if let Some(next) = readings.borrow_mut().next() {
                last.set(next);
            }
            Some(last.get())
        };
        let sleeps = Cell::new(0usize);
        let sleep = |_| sleeps.set(sleeps.get() + 1);
        let resume = || resume_at.is_some_and(|at| sleeps.get() == at);
        let shown = RefCell::new(Vec::new());
        let outcome = wait(&Probe { read: &read, sleep: &sleep, resume: &resume }, &|free| shown.borrow_mut().push(free));
        (outcome, shown.into_inner(), sleeps.get())
    }

    #[test]
    fn enough_memory_never_waits() {
        assert_eq!(run(vec![5000], None), (Wait::NotNeeded, vec![], 0));
    }

    #[test]
    fn waits_until_the_resume_level_and_reports_every_few_seconds() {
        let mut readings = vec![1500; 12];
        readings.push(3100);
        let (outcome, shown, sleeps) = run(readings, None);
        assert_eq!(outcome, Wait::Freed);
        assert_eq!(sleeps, 12);
        assert_eq!(shown, [1500, 1500, 1500]);
    }

    #[test]
    fn between_low_and_resume_it_keeps_waiting() {
        let (outcome, _, sleeps) = run(vec![1500, 2500, 2500, 3072], None);
        assert_eq!((outcome, sleeps), (Wait::Freed, 3));
    }

    #[test]
    fn the_button_resumes_at_once() {
        let (outcome, _, sleeps) = run(vec![1000], Some(4));
        assert_eq!((outcome, sleeps), (Wait::Resumed, 4));
    }

    #[test]
    fn gives_up_after_the_limit() {
        let (outcome, shown, sleeps) = run(vec![1000], None);
        assert_eq!(outcome, Wait::TimedOut);
        assert_eq!(sleeps as u64, WAIT_LIMIT.as_secs());
        assert_eq!(shown.len() as u64, 1 + WAIT_LIMIT.as_secs() / 5);
    }
}
