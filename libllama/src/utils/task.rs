use std::marker::Send;
use std::sync::{self, atomic};
use std::thread;

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

    pub fn try_join(&mut self) -> Result<bool, ()> {
        if self.running_internal.load(atomic::Ordering::SeqCst) {
            Ok(false)
        } else {
            self.stop().map(|_| true)
        }
    }

    pub fn stop(&mut self) -> Result<bool, ()> {
        if let Some(handle) = self.handle.take() {
            self.running.store(false, atomic::Ordering::SeqCst);
            if handle.join().is_err() {
                Err(())
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        self.stop();
    }
}