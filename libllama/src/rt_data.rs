use std::sync::{self, atomic, mpsc};
use io::hid;

#[derive(Clone)]
pub struct StateFlag(sync::Arc<atomic::AtomicBool>);
impl StateFlag {
    pub fn new(val: bool) -> StateFlag {
        StateFlag(sync::Arc::new(atomic::AtomicBool::new(val)))
    }

    pub fn exchange(&self, val: bool) -> bool {
        self.0.swap(val, atomic::Ordering::SeqCst)
    }

    pub fn get(&self) -> bool {
        self.0.load(atomic::Ordering::SeqCst)
    }
}

pub struct Tx {
    pub key_dmp: StateFlag,
}

pub struct Rx {
    pub key_dmp: StateFlag,
}

pub fn make_channels() -> (Tx, Rx) {
    let key_dmp_state = StateFlag::new(false);

    (Tx {
        key_dmp: key_dmp_state.clone(),
    },
    Rx {
        key_dmp: key_dmp_state
    })
}