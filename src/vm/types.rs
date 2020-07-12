use crate::messages::{ActorAddr, Atom, Value};

#[derive(Debug, PartialEq)]
pub enum IO {
    SendMessage {
        to: ActorAddr,
        // TODO
        // delay: Duration
        atom: Atom,
        values: Vec<Value>,
    },
    AddHandler {
        atom: Atom,
        program: u64,
        presets: Vec<(u8, Value)>,
    },
    RemoveHandler {
        atom: Atom,
    },
    Exit,
}

#[derive(Debug, PartialEq)]
pub enum VMError {
    OutOfBounds,
    NoSuchInstruction(u8),
    WrongValueType(Value),
}

#[derive(Debug, Eq, PartialEq)]
pub struct IOState {
    pub self_addr: ActorAddr,
    // TODO
    // rng
}
