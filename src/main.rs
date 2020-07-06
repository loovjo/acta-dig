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
                atom: acting::Atom {
                    name: io_actor::ATOM_HELLO.to_string(),
                },
                data: vec![],
            },
        })
        .expect("Could not send");

    while worker.step_once() {}

    println!("IOActor at {:?}", io_addr);
}
