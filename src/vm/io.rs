use crate::messages::{ActorAddr, Atom, Value};

#[derive(Debug, PartialEq)]
pub enum IO<'a> {
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
        atom_name: &'a str,
        presets: Vec<(u8, Value)>,
    },
    RemoveHandler {
        atom: Atom,
    },
    Done,
}

#[derive(Debug, PartialEq)]
pub enum VMError {
    OutOfBounds,
    NoSuchInstruction(u8),
    WrongValueType(Value),
    InvalidUTF8Error(Vec<u8>),
}

#[derive(Debug, Eq, PartialEq)]
pub struct IOState {
    pub self_addr: ActorAddr,
    // TODO
    // rng
}
