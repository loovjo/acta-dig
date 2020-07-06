mod acting;
mod io_actor;

fn main() {
    let mut worker = acting::Worker::new();

    let io_addr = worker.add_actor(Box::new(io_actor::IOActor));

    worker
        .msg_send
        .send(acting::Message {
            to: io_addr,
            cont: acting::MessageContent {
                atom: acting::Atom::PrintMsg,
                data: vec![acting::Argument::String("Hello, world".to_string())],
            },
        })
        .expect("Could not send");

    while worker.step_once() {}

    println!("IOActor at {:?}", io_addr);
}
