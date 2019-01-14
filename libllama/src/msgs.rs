use std::collections::HashMap;
use std::hash::Hash;
use std::thread;
use std::sync::mpsc;

pub fn make_idle_task<S, M, FI>(client: Client<M>, state: S, idle_handler: FI) -> thread::JoinHandle<()>
        where M: Ident + Send + Clone + 'static, S: Send + 'static, FI: Fn(M, &mut S) -> bool + Send + 'static {
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

type ClientId = &'static str;
type TxMsgs<M> = &'static [<M as Ident>::Identifier];
type RxMsgs<M> = &'static [<M as Ident>::Identifier];

type Route<X> = mpsc::Sender<X>;
type RouteSink<X> = mpsc::Receiver<X>;

type GraphSpec<M> = &'static [(
    ClientId,
    TxMsgs<M>,
    RxMsgs<M>
)];


type ClientRoutes<M> = HashMap<ClientId, HashMap<<M as Ident>::Identifier, Vec<Route<M>>>>;
type ClientMap<M> = HashMap<ClientId, (Route<M>, RouteSink<M>)>;
type MsgDests<M> = HashMap<<M as Ident>::Identifier, Vec<ClientId>>;

pub struct MsgGraph<M: Ident + Clone + Send + 'static> {
    client_routes: ClientRoutes<M>,
    clients: ClientMap<M>
}

impl<M: Ident + Clone + Send> MsgGraph<M> {
    pub fn new(graph_spec: GraphSpec<M>) -> MsgGraph<M> {
        let mut client_routes: ClientRoutes<M> = HashMap::new();
        let mut clients: ClientMap<M> = HashMap::new();
        let mut msg_dests: MsgDests<M> = HashMap::new();
        
        for (client, _, rx_msgs) in graph_spec {
            clients.insert(*client, mpsc::channel());
            for rx_msg in rx_msgs.iter() {
                msg_dests.entry(rx_msg.clone()).or_insert(Vec::new())
                    .push(*client);
            }
        }
        
        for (client, tx_msgs, _) in graph_spec {
            let msg_routes = client_routes.entry(client).or_insert(HashMap::new());

            for tx_msg in tx_msgs.iter() {
                let routes = msg_routes.entry(tx_msg.clone()).or_insert(Vec::new());

                if let Some(dst_clients) = msg_dests.get(tx_msg) {
                    for dst_client in dst_clients {
                        let route = clients[dst_client].0.clone();
                        routes.push(route);
                    }
                }
            }
        }
        
        MsgGraph {
            client_routes,
            clients
        }
    }
    
    pub fn client(&mut self, client: ClientId) -> Option<Client<M>> {
        let mut tx_map = HashMap::new();
        
        for (msg, tx_routes) in self.client_routes.remove(client)? {
            let routes = tx_map.entry(msg.clone()).or_insert(Vec::new());
            for tx in tx_routes {
                routes.push(tx);
            }
        }
        
        Some(Client {
            tx: tx_map,
            rx: self.clients.remove(client)?.1
        })
    }
}

pub struct Client<M: Ident + Clone> {
    tx: HashMap<<M as Ident>::Identifier, Vec<Route<M>>>,
    rx: RouteSink<M>
}

impl<M: Ident + Clone> Client<M> {
    pub fn send(&self, msg: M) {
        let vec = self.tx.get(&msg.ident())
            .expect("Attempted to send unauthorized message!");
        
        for dst in vec {
            let _ = dst.send(msg.clone());
        }
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
