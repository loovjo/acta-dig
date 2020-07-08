use crate::acting;

pub struct IOActor;

impl acting::Actor for IOActor {
    fn handle_message(&mut self, msg: acting::MessageContent, _ctx: acting::Context) {
        match (msg.atom, &*msg.data) {
            (acting::Atom::PrintHello, &[]) => {
                println!("Hello");
            }
            (acting::Atom::PrintMsg, &[acting::Argument::String(ref msg)]) => {
                println!("{}", msg);
            }
            _ => {}
        }
    }
}
