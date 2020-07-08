use crate::acting;

pub struct IOActor;

impl IOActor {
    pub const PRINT_HELLO: acting::Atom = acting::Atom(0);
    pub const PRINT_MSG: acting::Atom = acting::Atom(1);
}

impl acting::Actor for IOActor {
    fn handle_message(&mut self, msg: acting::MessageContent, _ctx: acting::Context) {
        match (msg.atom, &*msg.data) {
            (Self::PRINT_HELLO, &[]) => {
                println!("Hello");
            }
            (Self::PRINT_MSG, &[acting::Argument::String(ref msg)]) => {
                println!("{}", msg);
            }
            _ => {}
        }
    }
}
