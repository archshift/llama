use std::cell::Cell;

use indextree::{Arena, NodeId};

use cpu::BreakReason;
use cpu::dummy11::Dummy11HW;

#[macro_export]
macro_rules! dmnode_inner {
    ($arena:expr, $base_node:expr, $op:expr, $dbg:expr) => ({
        let node = $arena.new_node(ProgramNodeInner::new($op, $dbg));
        $base_node.append(node, $arena);
        node
    });
}

#[macro_export]
macro_rules! dmnode {
    ( in $arena:expr, $base_node:expr; do $body:expr ) =>
        ( dmnode_inner!($arena, $base_node, ProgramOp::Stmt($body), stringify!($body)); );
    ( in $arena:expr, $base_node:expr; if $body:expr ) =>
        ( dmnode_inner!($arena, $base_node, ProgramOp::If($body), concat!("if ", stringify!($body))); );
    ( in $arena:expr, $base_node:expr; while $body:expr ) =>
        ( dmnode_inner!($arena, $base_node, ProgramOp::While($body), concat!("while ", stringify!($body))); );
    ( in $arena:expr, $base_node:expr; break $body:expr ) =>
        ( dmnode_inner!($arena, $base_node, ProgramOp::Break($body), concat!("break ", stringify!($body))); );
    ( in $arena:expr, $base_node:expr; nop ) =>
        ( dmnode_inner!($arena, $base_node, ProgramOp::Nop, "nop"); )
}

#[macro_export]
macro_rules! dmerr {
    (+) => {Ok};
    (-) => {Err};
    (=) => {{
        fn identity<T>(t: T) -> T { t }
        identity
    }}
}

#[macro_export]
macro_rules! dmprog {
    // DESUGAR PROGRAM
    (
        in $program:expr;
        with $state:tt, $hw:tt;
        $($rest:tt)*
    ) => {{
        let (a, bn) = (&mut $program.arena, $program.base_node);
        dmprog!(in a, bn; with $state, $hw; $($rest)*);
    }};

    // BASE CASE
    (
        in $arena:expr, $base_node:expr;
        with $state:tt, $hw:tt;
    ) => {};

    // EXPRESSION
    (
        in $arena:expr, $base_node:expr;
        with $state:tt, $hw:tt;
        $e:tt{ $($code:tt)* };
        $($rest:tt)*
    ) => {
        #[allow(unused_variables)] {
            dmnode!(in $arena, $base_node; do |$state, $hw| dmerr!($e)({ $($code)* }) );
            dmprog!(in $arena, $base_node; with $state, $hw; $($rest)*);
        }
    };

    // WHILE STATEMENT
    (
        in $arena:expr, $base_node:expr;
        with $state:tt, $hw:tt;
        while $e:tt{ $($cond:tt)* } {
            $($body:tt)*
        }
        $($rest:tt)*
    ) => {
        #[allow(unused_variables)] {
            {
                let while_node = dmnode!(in $arena, $base_node; while |$state, $hw| dmerr!($e)({ $($cond)* }) );
                dmprog!(in $arena, while_node; with $state, $hw; $($body)*);
            }
            dmprog!(in $arena, $base_node; with $state, $hw; $($rest)*);
        }
    };
}


pub struct BoxedSteppable(pub(crate) Box<Steppable + Send>);

pub(crate) type OpFn<State> = fn(&mut State, &mut Dummy11HW) -> Result<(), ()>;
pub(crate) type CondFn<State> = fn(&State, &Dummy11HW) -> Result<bool, ()>;

#[allow(dead_code)] 
pub(crate) enum ProgramOp<State> {
    Nop,
    Block,
    Stmt(OpFn<State>),
    If(CondFn<State>),
    While(CondFn<State>),
    Break(BreakReason)
}

pub(crate) struct Program<State> {
    pub(crate) arena: Arena<ProgramNodeInner<State>>,
    pub(crate) base_node: NodeId,
    state: State,
    active_node: NodeId,
}

impl<State> Program<State>
    where State: Send + 'static {

    pub(crate) fn new(state: State) -> Program<State> {
        let mut arena = Arena::new();
        let base_node = arena.new_node(ProgramNodeInner::new(ProgramOp::Block, "{}"));
        Program {
            arena: arena,
            base_node: base_node,
            state: state,
            active_node: base_node
        }
    }

    fn add_idle_loop(&mut self) {
        let (a, bn) = (&mut self.arena, self.base_node);
        let while_node = dmnode!(in a, bn; while |_, _| Ok(true));
        {
            dmnode!(in a, while_node; break BreakReason::WFI);
        }
    }

    pub(crate) fn build(mut self) -> BoxedSteppable {
        self.add_idle_loop();
        BoxedSteppable(Box::new(self))
    }
}

pub(crate) trait Steppable {
    fn step(&mut self, hardware: &mut Dummy11HW) -> BreakReason;
}

impl<State> Steppable for Program<State> {
    fn step(&mut self, hardware: &mut Dummy11HW) -> BreakReason {
        let ret = run_node(&mut self.state, hardware, &mut self.arena, self.active_node).unwrap();
        if let Some(n) = next_node(&mut self.arena, self.active_node) {
            self.active_node = n;
        } else {
            panic!("Reached end of Dummy11 program!");
        }
        ret
    }
}

pub(crate) struct ProgramNodeInner<State> {
    op: ProgramOp<State>,
    enter_body: Cell<bool>,
    run: Cell<bool>,
    sticky: bool,
    _dbg_string: String,
}

impl<State> ProgramNodeInner<State> {
    pub(crate) fn new(op: ProgramOp<State>, _dbg_string: &str) -> ProgramNodeInner<State> {
        let (enter_body, sticky) = match op {
            ProgramOp::Nop
            | ProgramOp::Stmt(_)
            | ProgramOp::If(_)
            | ProgramOp::Break(_) => (false, false),
            ProgramOp::Block => (true, false),
            ProgramOp::While(_) => (false, true),
        };
        ProgramNodeInner {
            op: op,
            enter_body: Cell::new(enter_body),
            run: Cell::new(true),
            sticky: sticky,
            _dbg_string: _dbg_string.to_owned()
        }
    }
}

fn run_node<State>(state: &mut State, hw: &mut Dummy11HW,
                   arena: &mut Arena<ProgramNodeInner<State>>,
                   node_id: NodeId) -> Result<BreakReason, ()> {
    let data = &mut arena[node_id].data;
    if !data.run.get() {
        return Ok(BreakReason::LimitReached)
    }
    trace!("Running Dummy11 node: {}", data._dbg_string);
    match data.op {
        ProgramOp::Stmt(f) => f(state, hw)?,
        ProgramOp::If(c) | ProgramOp::While(c) => { data.enter_body.replace(c(state, hw)?); },
        ProgramOp::Break(ref r) => return Ok(r.clone()),
        _ => {}
    }
    Ok(BreakReason::LimitReached)
}

fn next_node<State>(arena: &Arena<ProgramNodeInner<State>>, node_id: NodeId) -> Option<NodeId> {
    let node = &arena[node_id];

    if node.data.enter_body.get() {
        if !node.data.sticky {
            node.data.enter_body.replace(false);
            node.data.run.replace(false);
        }
        if let Some(child) = node.first_child() {
            Some(child)
        } else {
            panic!("Attempted to enter body of childless node!");
        }
    } else if let Some(parent) = node.parent() {
        if let Some(next_sibling) = node.next_sibling() {
            Some(next_sibling)
        } else {
            Some(parent)
        }
    } else {
        None
    }
}
