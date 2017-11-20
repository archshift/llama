use std::cell::Cell;

use indextree::{Arena, NodeId};

use cpu::BreakReason;
use mem;

pub struct BoxedSteppable(Box<Steppable + Send + Sync>);

macro_rules! dmnode_inner {
    ($arena:expr, $base_node:expr, $op:expr, $dbg:expr) => ({
        let node = $arena.new_node(ProgramNodeInner::new($op, $dbg));
        $base_node.append(node, $arena);
        node
    });
}

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

pub mod modes {
    use std::time;
    use std::thread;

    use super::*;

    pub fn idle() -> BoxedSteppable {
        let mut program = Program::<()>::new(());
        program.build()
    }

    pub fn boot() -> BoxedSteppable {
        let mut program = Program::<()>::new(());
        {
            let (a, bn) = (&mut program.arena, program.base_node);

            const PXI_SYNC_ADDR: u32 = 0x10163000;
            fn pxisync_read(hw: &Dummy11HW) -> u8 {
                (hw.memory.read::<u32>(PXI_SYNC_ADDR) >> 8) as u8
            }
            fn pxisync_write(hw: &mut Dummy11HW, val: u8) {
                let current = hw.memory.read::<u32>(PXI_SYNC_ADDR);
                let new = current & 0xFFFFFF00 | val as u32;
                hw.memory.write::<u32>(PXI_SYNC_ADDR, new);
            }

            let while_node = dmnode!(in a, bn; while |_, hw| Ok(pxisync_read(hw) != 9));
            {
                dmnode!(in a, while_node; do |_, _| Ok(thread::yield_now()));
            }

            dmnode!(in a, bn; do |_, hw| Ok(pxisync_write(hw, 11)));
        }
        program.build()
    }

    pub fn kernel() -> BoxedSteppable {
        let mut program = Program::<()>::new(());
        {
            let (a, bn) = (&mut program.arena, program.base_node);

            const SYNC_ADDR: u32 = 0x1FFFFFF0;

            dmnode!(in a, bn; do |_, hw| Ok(hw.memory.write::<u8>(SYNC_ADDR, 1)));

            let while_node = dmnode!(in a, bn; while |_, hw|
                Ok(hw.memory.read::<u8>(SYNC_ADDR) != 2
            ));
            {
                dmnode!(in a, while_node; do |_, _| Ok(thread::yield_now()));
            }

            dmnode!(in a, bn; do |_, hw| Ok(hw.memory.write::<u8>(SYNC_ADDR, 3)));
        }
        program.build()
    }
}

struct Dummy11HW {
    memory: mem::MemController
}

pub struct Dummy11 {
    hw: Dummy11HW,
    program: BoxedSteppable
}

impl Dummy11 {
    pub fn new(memory: mem::MemController, program: BoxedSteppable) -> Dummy11 {
        Dummy11 {
            hw: Dummy11HW {
                memory: memory
            },
            program: program
        }
    }

    pub fn step(&mut self) -> BreakReason {
        self.program.0.step(&mut self.hw)
    }
}

type OpFn<State> = fn(&mut State, &mut Dummy11HW) -> Result<(), ()>;
type CondFn<State> = fn(&State, &Dummy11HW) -> Result<bool, ()>;

enum ProgramOp<State> {
    Nop,
    Block,
    Stmt(OpFn<State>),
    If(CondFn<State>),
    While(CondFn<State>),
    Break(BreakReason)
}

struct Program<State> {
    arena: Arena<ProgramNodeInner<State>>,
    base_node: NodeId,
    state: State,
    active_node: NodeId,
}

impl<State> Program<State>
    where State: Sync + Send + 'static {

    fn new(state: State) -> Program<State> {
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

    fn build(mut self) -> BoxedSteppable {
        self.add_idle_loop();
        BoxedSteppable(Box::new(self))
    }
}

trait Steppable {
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

struct ProgramNodeInner<State> {
    op: ProgramOp<State>,
    enter_body: Cell<bool>,
    run: Cell<bool>,
    sticky: bool,
    _dbg_string: String,
}

unsafe impl<State> Sync for ProgramNodeInner<State> {} // TODO: not good!

impl<State> ProgramNodeInner<State> {
    fn new(op: ProgramOp<State>, _dbg_string: &str) -> ProgramNodeInner<State> {
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