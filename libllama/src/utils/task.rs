use std::marker::Send;
use std::mem;
use std::sync::{self, atomic};
use std::thread;

#[derive(Eq, PartialEq)]
pub enum StopStatus {
    Stopped,
    NotRunning,
}

pub trait TaskMgmt {
    fn running(&self) -> bool;
    fn stop(&mut self) -> Result<StopStatus, ()>;
}

struct RunningMarker(sync::Arc<atomic::AtomicBool>);
impl RunningMarker {
    fn mark(atomic: sync::Arc<atomic::AtomicBool>) -> RunningMarker {
        atomic.store(true, atomic::Ordering::SeqCst);
        RunningMarker(atomic)
    }
}
impl Drop for RunningMarker {
    fn drop(&mut self) {
        let &mut RunningMarker(ref mut atomic) = self;
        atomic.store(false, atomic::Ordering::SeqCst);
    }
}

pub enum ReturnVal<T> {
    Waiting(thread::JoinHandle<T>),
    Ready(T),
    None
}

impl<T> ReturnVal<T> {
    pub fn take(&mut self) -> ReturnVal<T> {
        mem::replace(self, ReturnVal::None)
    }
}

pub struct Task<T> {
    should_run: sync::Arc<atomic::AtomicBool>,
    running_internal: sync::Arc<atomic::AtomicBool>,
    pub ret: ReturnVal<T>,
}

impl<T> Task<T> {
    pub fn spawn<F>(f: F) -> Task<T>
        where T: Send + 'static, F: FnOnce(sync::Arc<atomic::AtomicBool>) -> T, F: Send + 'static {

        let should_run = sync::Arc::new(atomic::AtomicBool::new(true));
        let should_run_t = should_run.clone();
        let running_internal = sync::Arc::new(atomic::AtomicBool::new(false));
        let running_internal_t = running_internal.clone();

        let marker = RunningMarker::mark(running_internal_t);

        let handle = thread::spawn(move || {
            let _running_marker = marker;
            f(should_run_t)
        });

        Task {
            should_run: should_run,
            running_internal: running_internal,
            ret: ReturnVal::Waiting(handle),
        }
    }
}

impl<T> TaskMgmt for Task<T> {
    fn running(&self) -> bool {
        self.running_internal.load(atomic::Ordering::SeqCst)
    }

    fn stop(&mut self) -> Result<StopStatus, ()> {
        if let ReturnVal::Waiting(handle) = self.ret.take() {
            self.should_run.store(false, atomic::Ordering::SeqCst);
            if let Ok(x) = handle.join() {
                self.ret = ReturnVal::Ready(x);
                Ok(StopStatus::Stopped)
            } else {
                Err(())
            }
        } else {
            Ok(StopStatus::NotRunning)
        }
    }
}

impl<T> Drop for Task<T> {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[derive(Eq, PartialEq)]
pub enum EndBehavior {
    StopAll,
    Ignore
}

pub struct TaskUnion<'a> (
    pub &'a mut [(Option<&'a mut TaskMgmt>, EndBehavior)]
);

impl<'a> TaskUnion<'a> {
    pub fn should_stop(&self) -> bool {
        let make_has_stopped = |tu: &(Option<&'a mut TaskMgmt>, EndBehavior)| {
            if let Some(ref task) = tu.0 { !task.running() }
            else { true }
        };

        self.0.iter()
              .filter(|ref tu| tu.1 == EndBehavior::StopAll)
              .map(make_has_stopped)
              .fold(false, |acc, b| acc || b)
    }

    pub fn stop(&mut self) -> Result<StopStatus, ()> {
        let status_folder = |acc, stop_res| {
            match (acc, stop_res) {
                (Ok(StopStatus::Stopped), Ok(_)) | (Ok(_), Ok(StopStatus::Stopped)) => Ok(StopStatus::Stopped),
                (Ok(_), Ok(_)) => Ok(StopStatus::NotRunning),
                _ => Err(())
            }
        };

        self.0.iter_mut()
              .filter_map(|tu| tu.0.as_mut())
              .map(|t| t.stop())
              .fold(Ok(StopStatus::NotRunning), status_folder)
    }
}