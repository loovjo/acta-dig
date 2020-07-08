use std::collections::HashMap;

use crate::acting;

pub struct DynActor {
    pub actor_name: String,
    pub actor_fns:
        HashMap<acting::Atom, (String, Box<dyn Fn(acting::MessageContent, acting::Context)>)>,
}

impl acting::Actor for DynActor {
    fn handle_message(&mut self, msg: acting::MessageContent, ctx: acting::Context) {
        let actor_fn = if let Some((_name, af)) = self.actor_fns.get(&msg.atom) {
            af
        } else {
            return;
        };

        actor_fn(msg, ctx);
    }
    fn debug_info(&self) -> Box<dyn acting::DebugInfo> {
        let mut fn_names = HashMap::new();
        for (&atom, (name, _fn)) in self.actor_fns.iter() {
            fn_names.insert(atom, name.clone());
        }
        Box::new(DynActorDebugInfo {
            actor_name: self.actor_name.clone(),
            fn_names,
        })
    }
}

pub struct DynActorDebugInfo {
    actor_name: String,
    fn_names: HashMap<acting::Atom, String>,
}

impl acting::DebugInfo for DynActorDebugInfo {
    fn get_actor_name(&self) -> String {
        self.actor_name.clone()
    }
    fn get_atom_name(&self, atom: acting::Atom) -> Option<String> {
        self.fn_names.get(&atom).cloned()
    }
}
