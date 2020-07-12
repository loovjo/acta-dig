mod acting;
mod messages;

mod actors;
use actors::{dactor, io_actor};

mod vm;

fn main() {
    let mut worker = acting::Worker::new();

    let io_addr = worker.add_actor(Box::new(io_actor::IOActor));

    let dactor = dactor::Dactor::new(make_base());

    let dactor_addr = worker.add_actor(Box::new(dactor));

    eprintln!("Registered io actor at {:?}", io_addr);
    eprintln!("Registered dyn actor at {:?}", dactor_addr);

    worker
        .msg_queue
        .make_ctx(messages::ActorAddr::random())
        .push_msg(messages::Message {
            to: dactor_addr,
            cont: messages::MessageContent {
                atom: messages::Atom(0x333301), // get-counter
                data: vec![
                    messages::Value::ActorAddr(io_addr),
                    messages::Value::Atom(io_actor::IOActor::PRINT_MSG),
                ],
            },
            arrive_after: None,
        });

    while worker.step_once() {}
}

fn make_base() -> &'static vm::dactor_base::DactorBase {
    let base = vm::dactor_base::DactorBase::parse(include_bytes!("simple_actor.act"))
        .expect("no parse :(");
    Box::leak(Box::new(base)) // not pog
}
