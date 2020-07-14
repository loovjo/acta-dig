use std::collections::HashMap;

use crate::acting::{Actor, Context, DebugInfo};
use crate::messages::{ActorAddr, Atom, Message, MessageContent, Value};

use crate::vm::dactor_base::DactorBase;
use crate::vm::{IOState, VMState, IO};

// TODO: Make atom_name into a reference to the actual actor_base source.
// Currently we clone the atom_name three from adding a handler to getting debug info for the
// corresponding atom, which is not ideal.
pub struct Handler {
    program_idx: usize,
    presets: Vec<(u8, Value)>,
    atom_name: String,
}

impl Handler {
    fn handle<'ctx>(
        &'ctx self,
        base: &'ctx DactorBase,
        msg: MessageContent,
        mut ctx: Option<Context<'ctx>>,
    ) -> HashMap<Atom, Option<Handler>> {
        let mut vm = VMState::new(&base.actor_code, base.programs[self.program_idx]);
        // Place preset
        for (i, arg) in self.presets.iter() {
            vm.registers[*i as usize] = arg.clone();
        }
        // Place message
        for (i, val) in msg.data.into_iter().enumerate() {
            vm.registers[i] = val;
        }

        let mut new_handlers = HashMap::new();
        let io_state = if let Some(ref ctx) = ctx {
            IOState {
                self_addr: ctx.self_addr,
            }
        } else {
            IOState {
                self_addr: ActorAddr { addr: 0 },
            }
        };
        loop {
            match vm.step_once(&io_state) {
                Ok(None) => {}
                Ok(Some(IO::Done)) => break,
                Ok(Some(IO::SendMessage { to, atom, values })) => {
                    if let Some(ref mut ctx) = ctx {
                        ctx.push_msg(Message {
                            to,
                            arrive_after: None,
                            cont: MessageContent { atom, data: values },
                        });
                    }
                }
                Ok(Some(IO::AddHandler {
                    atom,
                    program,
                    atom_name,
                    presets,
                })) => {
                    new_handlers.insert(
                        atom,
                        Some(Handler {
                            program_idx: program as usize,
                            presets,
                            atom_name: atom_name.to_string(),
                        }),
                    );
                }
                Ok(Some(IO::RemoveHandler { atom })) => {
                    new_handlers.insert(atom, None);
                }
                Err(e) => {
                    eprintln!("Handler received error: {:?}", e);
                    break;
                }
            }
        }
        new_handlers
    }
}

pub struct Dactor<'a> {
    base: &'a DactorBase,
    handlers: HashMap<Atom, Handler>,
}

impl<'a> Dactor<'a> {
    pub fn new(base: &'a DactorBase) -> Dactor<'a> {
        let mut this = Dactor {
            base,
            handlers: HashMap::new(),
        };
        let start_handler = Handler {
            program_idx: 0,
            presets: Vec::new(),
            atom_name: "start-all".to_string(),
        };

        let resp = start_handler.handle(
            base,
            MessageContent {
                atom: Atom(0),
                data: Vec::new(),
            },
            None,
        );
        this.handle_handler_response(resp);
        this
    }

    fn handle_handler_response(&mut self, response: HashMap<Atom, Option<Handler>>) {
        for (atom, hand) in response.into_iter() {
            match hand {
                Some(handler) => {
                    self.handlers.insert(atom, handler);
                }
                None => {
                    self.handlers.remove(&atom);
                }
            }
        }
    }
}

impl<'a> Actor for Dactor<'a> {
    fn handle_message<'ctx>(&'ctx mut self, message: MessageContent, ctx: Context<'ctx>) {
        let handler = if let Some(handler) = self.handlers.get(&message.atom) {
            handler
        } else {
            return;
        };

        let handler_response = handler.handle(self.base, message, Some(ctx));
        self.handle_handler_response(handler_response);
    }

    fn debug_info<'di>(&'di self) -> Box<dyn DebugInfo + 'di> {
        Box::new(DactorDebugInfo {
            atom_names: self
                .handlers
                .iter()
                .map(|(&k, h)| (k, h.atom_name.clone()))
                .collect(),
        })
    }
}

pub struct DactorDebugInfo {
    atom_names: HashMap<Atom, String>,
}

impl DebugInfo for DactorDebugInfo {
    fn get_actor_name(&self) -> String {
        "Dactor".to_string()
    }
    fn get_atom_name(&self, atom: Atom) -> Option<String> {
        self.atom_names.get(&atom).cloned()
    }
}
