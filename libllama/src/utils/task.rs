use std::marker::Send;
use std::sync::{self, atomic};
use std::thread;

#[derive(Eq, PartialEq)]
pub enum JoinStatus {
    NotReady,
    Joined
}

#[derive(Eq, PartialEq)]
pub enum StopStatus {
    Stopped,
    NotRunning,
}

pub trait TaskMgmt {
    fn try_join(&mut self) -> Result<JoinStatus, ()>;
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

pub struct Task {
    running: sync::Arc<atomic::AtomicBool>,
    running_internal: sync::Arc<atomic::AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Task {
    pub fn spawn<F>(f: F) -> Task
        where F: FnOnce(sync::Arc<atomic::AtomicBool>), F: Send + 'static {

        let running = sync::Arc::new(atomic::AtomicBool::new(true));
        let running_t = running.clone();
        let running_internal = sync::Arc::new(atomic::AtomicBool::new(false));
        let running_internal_t = running_internal.clone();

        let marker = RunningMarker::mark(running_internal_t);

        let handle = thread::spawn(move || {
            let running_marker = marker;
            f(running_t);
        });

        Task {
            running: running,
            running_internal: running_internal,
            handle: Some(handle),
        }
    }
}

impl TaskMgmt for Task {
    fn try_join(&mut self) -> Result<JoinStatus, ()> {
        if self.running_internal.load(atomic::Ordering::SeqCst) {
            Ok(JoinStatus::NotReady)
        } else {
            self.stop().map(|_| JoinStatus::Joined)
        }
    }

    fn stop(&mut self) -> Result<StopStatus, ()> {
        if let Some(handle) = self.handle.take() {
            self.running.store(false, atomic::Ordering::SeqCst);
            if handle.join().is_err() {
                Err(())
            } else {
                Ok(StopStatus::Stopped)
            }
        } else {
            Ok(StopStatus::NotRunning)
        }
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        self.stop();
    }
}

#[derive(Eq, PartialEq)]
pub enum EndBehavior {
    StopAll,
    Ignore
}

pub struct TaskUnion<'a> (
    pub &'a mut [(&'a mut Option<Task>, EndBehavior)]
);

impl<'a> TaskMgmt for TaskUnion<'a> {
    fn try_join(&mut self) -> Result<JoinStatus, ()> {
        let make_has_stopped = |tu: &mut (&'a mut Option<Task>, EndBehavior)| {
            match tu.0.as_mut() {
                Some(ref mut task) => Ok(task.try_join()? == JoinStatus::Joined),
                None => Ok(true)
            }
        };

        let should_stop = self.0.iter_mut()
                                .filter(|ref tu| tu.1 == EndBehavior::StopAll)
                                .map(make_has_stopped)
                                .fold(Ok(true), |acc, b| Ok(acc? && b?));
        match should_stop {
            Ok(true) | Err(_) => should_stop.and(self.stop())
                                            .map(|_| JoinStatus::Joined),
            Ok(false) => Ok(JoinStatus::NotReady)
        }
    }

    fn stop(&mut self) -> Result<StopStatus, ()> {
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