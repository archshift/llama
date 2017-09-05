use std::sync::mpsc;
use io::hid;

pub struct Tx {
    pub hid_btn: mpsc::Sender<hid::ButtonState>,
}

pub struct Rx {
    pub hid_btn: mpsc::Receiver<hid::ButtonState>,
}

pub fn make_channels() -> (Tx, Rx) {
    let hid_btn_chan = mpsc::channel::<hid::ButtonState>();

    (Tx {
        hid_btn: hid_btn_chan.0,
    },
    Rx {
        hid_btn: hid_btn_chan.1,
    })
}