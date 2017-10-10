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
    pub hid_btn: mpsc::Sender<hid::ButtonState>,
    pub key_dmp: StateFlag,
}

pub struct Rx {
    pub hid_btn: mpsc::Receiver<hid::ButtonState>,
    pub key_dmp: StateFlag,
}

pub fn make_channels() -> (Tx, Rx) {
    let hid_btn_chan = mpsc::channel::<hid::ButtonState>();
    let key_dmp_state = StateFlag::new(false);

    (Tx {
        hid_btn: hid_btn_chan.0,
        key_dmp: key_dmp_state.clone(),
    },
    Rx {
        hid_btn: hid_btn_chan.1,
        key_dmp: key_dmp_state
    })
}