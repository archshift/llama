use std::collections::HashMap;
use std::hash::Hash;
use std::thread;
use std::sync::mpsc;

pub fn make_idle_task<S, M, FA, FI>(client: Client<M>, state: S, idle_handler: FI) -> thread::JoinHandle<()>
        where M: Send + 'static, S: Send + 'static, FI: Fn(M, &mut S) -> bool + Send + 'static {
    thread::spawn(move || {
        let mut state = state;
        for msg in client.iter() {
            if !idle_handler(msg, &mut state) {
                return
            }
        }
    })
}


pub trait Ident {
    type Identifier: Clone + Eq + Hash + Send;
    fn ident(&self) -> Self::Identifier;
}

pub struct PumpThread {
    thread: thread::JoinHandle<()>,
}

impl PumpThread {
    pub fn join(self) {
        self.thread.join().unwrap()
    }
}

pub struct Pump<M: Ident + Clone + Send + 'static> {
    incoming_txside: mpsc::Sender<M>,
    incoming: mpsc::Receiver<M>,
    outgoing: HashMap<<M as Ident>::Identifier, Vec<mpsc::Sender<M>>>
}

impl<M: Ident + Clone + Send> Pump<M> {
    pub fn new() -> Pump<M> {
        let (mpsc_tx, mpsc_rx) = mpsc::channel();
        Pump {
            incoming_txside: mpsc_tx,
            incoming: mpsc_rx,
            outgoing: HashMap::new(),
        }
    }

    pub fn start(self) -> PumpThread {
        PumpThread {
            thread: thread::Builder::new().name("MessagePump".to_owned())
                                          .stack_size(1<<16).spawn(move || {
                drop(self.incoming_txside);
                let incoming = self.incoming;
                let mut outgoing = self.outgoing;
                for msg in incoming.iter() {
                    outgoing.entry(msg.ident()).or_insert(Vec::new())
                            .retain(|client| client.send(msg.clone()).is_ok())
                }
            }).unwrap()
        }
    }

    pub fn add_client(&mut self, subscriptions: &[<M as Ident>::Identifier]) -> Client<M> {
        let (mpsc_tx, mpsc_rx) = mpsc::channel();
        for subscription in subscriptions {
            self.outgoing.entry(subscription.clone()).or_insert(Vec::new()).push(mpsc_tx.clone());
        }
        Client {
            tx: self.incoming_txside.clone(),
            rx: mpsc_rx,
        }
    }
}

pub struct Client<M> {
    tx: mpsc::Sender<M>,
    rx: mpsc::Receiver<M>
}

impl<M> Client<M> {
    pub fn send(&self, msg: M) -> Result<(), mpsc::SendError<M>> {
        self.tx.send(msg)
    }
    pub fn recv(&self) -> Result<M, mpsc::RecvError> {
        self.rx.recv()
    }
    pub fn try_recv(&self) -> Result<M, mpsc::TryRecvError> {
        self.rx.try_recv()
    }
    pub fn iter(&self) -> mpsc::Iter<M> {
        self.rx.iter()
    }
    pub fn try_iter(&self) -> mpsc::TryIter<M> {
        self.rx.try_iter()
    }
}