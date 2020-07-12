use crate::acting;
use crate::messages;

pub struct IOActor;

impl IOActor {
    pub const PRINT_HELLO: messages::Atom = messages::Atom(0);
    pub const PRINT_MSG: messages::Atom = messages::Atom(1);
}

impl acting::Actor for IOActor {
    fn handle_message(&mut self, msg: messages::MessageContent, _ctx: acting::Context) {
        match (msg.atom, &*msg.data) {
            (Self::PRINT_HELLO, &[]) => {
                println!("Hello");
            }
            (Self::PRINT_MSG, &[messages::Value::String(ref msg)]) => {
                println!("{}", msg);
            }
            _ => {}
        }
    }

    fn debug_info(&self) -> Box<dyn acting::DebugInfo> {
        Box::new(IODebugInfo)
    }
}

pub struct IODebugInfo;

impl acting::DebugInfo for IODebugInfo {
    fn get_actor_name(&self) -> String {
        "IOActor".to_string()
    }
    fn get_atom_name(&self, atom: messages::Atom) -> Option<String> {
        match atom {
            IOActor::PRINT_HELLO => Some("PRINT_HELLO".to_string()),
            IOActor::PRINT_MSG => Some("PRINT_MSG".to_string()),
            _ => None,
        }
    }
}
