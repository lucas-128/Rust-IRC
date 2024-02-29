mod channel;
mod channel_modes;
mod interpreter;
mod message;
mod registration;
mod replies;
pub mod server;
mod server_errors;
mod server_messages_interpreter;
mod threadpool;
mod user;

use std::env::args;
use std::io::{stdin, BufRead, BufReader};
use std::sync::Arc;
use std::thread;

use ::server::server::{attempt_server_conection, run, show_spanning_tree, Server};

static SERVER_ARGS: usize = 3;

fn main() {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != SERVER_ARGS {
        panic!("Cantidad de argumentos inválida");
    }
    let host = argv[1].to_owned();
    let port = argv[2].parse::<u16>().expect("Puerto inválido");
    let servername = host.clone() + &port.to_string();

    let stream = stdin();
    let reader = BufReader::new(stream);

    let mut server = Server::new();
    server.set_name(servername.clone());
    let server_ref = Arc::new(server);
    let sv_thread_ref = server_ref.clone();

    let _ = thread::spawn(move || {
        for line in reader.lines().flatten() {
            match line.as_str() {
                "show_net()" => show_spanning_tree(&sv_thread_ref, 0),
                _ => attempt_server_conection(sv_thread_ref.clone(), line, servername.clone()),
            };
        }
    });

    run(server_ref, host, port).expect("No se pudo inicializar el servidor");
}
