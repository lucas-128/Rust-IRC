use server;
use std::net::Shutdown;
use std::sync::mpsc::channel;
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{mpsc::Sender, Arc, Mutex},
    thread, time,
};

pub struct Client {
    pub nickname: Arc<Mutex<String>>,
    pub socket: TcpStream,
}

impl Client {
    pub fn new(address: String, nick: String) -> Client {
        let socket = Self::run(&address).unwrap();
        let nickname = Arc::new(Mutex::new(nick));
        Client { nickname, socket }
    }

    fn run(address: &String) -> std::io::Result<TcpStream> {
        let socket = TcpStream::connect(address)?;
        println!("Cliente conectado a {}", address);
        Ok(socket)
    }

    pub fn send(&self, msg: String) {
        let mut socket = &self.socket;
        let content = msg + "\n";
        println!("Enviando {:?}", content);
        let _ = socket.write(content.as_bytes());
    }

    pub fn receive(&self, tx: Sender<String>) {
        let socket = &self.socket;
        let reader = BufReader::new(socket);
        let mut lines = reader.lines();

        while let Some(Ok(line)) = lines.next() {
            println!("Mensaje recibido: {:?}", line);
            match tx.send(line) {
                Ok(()) => (),
                Err(e) => eprintln!("Error enviando mensaje: {}", e),
            }
        }
    }

    pub fn shutdown(&self) {
        self.socket
            .shutdown(Shutdown::Both)
            .expect("shutdown call failed");
    }

    pub fn get_nickname(&self) -> String {
        self.nickname.lock().unwrap().to_string()
    }
}

#[test]

fn register_four_clients_test() {
    let mut argv = Vec::new();
    argv.push("localhost".to_string());
    argv.push("8081".to_string());
    let host = argv[0].to_owned();
    let port = argv[1].parse::<u16>().expect("Puerto inválido");
    let servername = host.clone() + &port.to_string();

    let mut server = server::server::Server::new();
    server.set_name(servername.clone());
    let server_ref = Arc::new(server);

    let address = host.clone() + ":" + &port.to_string();
    let listener = TcpListener::bind(&address.clone()).unwrap();
    println!("Servidor configurado para escuchar en {}", &address);

    let mut clients_array: Vec<Client> = Vec::new();
    let mut register_array: Vec<String> = Vec::new();

    let client1 = Client::new("localhost:8081".to_string(), "juan".to_string());

    let client2 = Client::new("localhost:8081".to_string(), "martin".to_string());

    let client3 = Client::new("localhost:8081".to_string(), "mateo".to_string());

    let client4 = Client::new("localhost:8081".to_string(), "franco".to_string());

    clients_array.push(client1);
    clients_array.push(client2);
    clients_array.push(client3);
    clients_array.push(client4);

    let msg_pass = "PASS pass".to_string();

    let msg_nick1 = "NICK juan".to_string();
    let msg_nick2 = "NICK martin".to_string();
    let msg_nick3 = "NICK mateo".to_string();
    let msg_nick4 = "NICK franco".to_string();

    let msg_user = "USER user server localhost8080 real".to_string();

    register_array.push(msg_nick1);
    register_array.push(msg_nick2);
    register_array.push(msg_nick3);
    register_array.push(msg_nick4);

    let thread_pool = server::threadpool::ThreadPool::new(4);

    //let client_ref = Client::new("localhost:8080".to_string(), "juan".to_string());
    //let arc_client = Arc::new(client_ref.socket);
    let mut i = 0;

    let mut nick_array: Vec<String> = Vec::new();

    for client_stream in listener.incoming() {
        let arc_server = server_ref.clone();
        let arc_server_ref = arc_server.clone();
        thread_pool.execute(move || {
            let arc_client = Arc::new(client_stream.unwrap());
            server::server::handle_client(arc_server.clone(), arc_client)
                .expect("Error manejando cliente");
        });

        clients_array[i].send(msg_pass.clone());
        clients_array[i].send(register_array[i].clone());
        clients_array[i].send(msg_user.clone());

        let ten_millis = time::Duration::from_millis(100);

        thread::sleep(ten_millis);

        i = i + 1;

        if i == 4 {
            nick_array.push(arc_server_ref.users.lock().unwrap()[0].nickname.clone());
            nick_array.push(arc_server_ref.users.lock().unwrap()[1].nickname.clone());
            nick_array.push(arc_server_ref.users.lock().unwrap()[2].nickname.clone());
            nick_array.push(arc_server_ref.users.lock().unwrap()[3].nickname.clone());
            break;
        }
    }

    assert_eq!(nick_array.contains(&"juan".to_string()), true);
    assert_eq!(nick_array.contains(&"mateo".to_string()), true);
    assert_eq!(nick_array.contains(&"martin".to_string()), true);
    assert_eq!(nick_array.contains(&"franco".to_string()), true);
}

#[test]
fn privmsg_two_users() {
    let mut argv = Vec::new();
    argv.push("localhost".to_string());
    argv.push("8080".to_string());
    let host = argv[0].to_owned();
    let port = argv[1].parse::<u16>().expect("Puerto inválido");
    let servername = host.clone() + &port.to_string();

    let mut server = server::server::Server::new();
    server.set_name(servername.clone());
    let server_ref = Arc::new(server);

    let address = host.clone() + ":" + &port.to_string();
    //let sv_name = host
    //let mut srv = Server::new();
    //srv.name = host + &port.to_string();
    //let server = Arc::new(srv);
    let listener = TcpListener::bind(&address.clone()).unwrap();
    println!("Servidor configurado para escuchar en {}", &address);

    let mut clients_array: Vec<Client> = Vec::new();
    let mut register_array: Vec<String> = Vec::new();

    let client1 = Client::new("localhost:8080".to_string(), "juan".to_string());

    let client2 = Client::new("localhost:8080".to_string(), "martin".to_string());

    clients_array.push(client1);
    clients_array.push(client2);

    let msg_pass = "PASS pass".to_string();

    let msg_nick1 = "NICK juan".to_string();
    let msg_nick2 = "NICK martin".to_string();

    let msg_user = "USER user server localhost8080 real".to_string();

    let priv_msg_juan_martin = "PRIVMSG martin recibido".to_string();

    register_array.push(msg_nick1);
    register_array.push(msg_nick2);

    let thread_pool = server::threadpool::ThreadPool::new(4);

    //let client_ref = Client::new("localhost:8080".to_string(), "juan".to_string());
    //let arc_client = Arc::new(client_ref.socket);
    let mut i = 0;

    let mut msg_recibido = "".to_string();
    for client_stream in listener.incoming() {
        let arc_server = server_ref.clone();
        thread_pool.execute(move || {
            let arc_client = Arc::new(client_stream.unwrap());
            server::server::handle_client(arc_server.clone(), arc_client)
                .expect("Error manejando cliente");
        });

        clients_array[i].send(msg_pass.clone());
        clients_array[i].send(register_array[i].clone());
        clients_array[i].send(msg_user.clone());

        let ten_millis = time::Duration::from_millis(100);

        thread::sleep(ten_millis);

        i = i + 1;

        if i == 2 {
            clients_array[0].send(priv_msg_juan_martin.clone());

            let ten_millis = time::Duration::from_millis(1000);

            thread::sleep(ten_millis);

            let (sender, receiver) = channel();

            thread::spawn(move || {
                clients_array[1].receive(sender);
            });

            let ten_millis = time::Duration::from_millis(100);

            thread::sleep(ten_millis);

            let _ = receiver.recv().unwrap();
            msg_recibido = receiver.recv().unwrap();
            //println!("recibidoo {}", recibido);

            break;
        }
    }

    assert_eq!(msg_recibido, priv_msg_juan_martin);
}

#[test]
fn client_is_oper() {
    let mut argv = Vec::new();
    argv.push("localhost".to_string());
    argv.push("8082".to_string());
    let host = argv[0].to_owned();
    let port = argv[1].parse::<u16>().expect("Puerto inválido");
    let servername = host.clone() + &port.to_string();

    let mut server = server::server::Server::new();
    server.set_name(servername.clone());
    let server_ref = Arc::new(server);

    let address = host.clone() + ":" + &port.to_string();
    //let sv_name = host
    //let mut srv = Server::new();
    //srv.name = host + &port.to_string();
    //let server = Arc::new(srv);
    let listener = TcpListener::bind(&address.clone()).unwrap();
    println!("Servidor configurado para escuchar en {}", &address);

    let mut clients_array: Vec<Client> = Vec::new();
    let mut register_array: Vec<String> = Vec::new();

    let client1 = Client::new("localhost:8082".to_string(), "juan".to_string());

    clients_array.push(client1);

    let msg_pass = "PASS 1234".to_string();

    let msg_nick1 = "NICK juan".to_string();

    let msg_user = "USER admin server localhost8080 real".to_string();

    let msg_oper = "OPER admin 1234".to_string();

    register_array.push(msg_nick1);

    let thread_pool = server::threadpool::ThreadPool::new(4);

    //let client_ref = Client::new("localhost:8080".to_string(), "juan".to_string());
    //let arc_client = Arc::new(client_ref.socket);
    let mut i = 0;

    let mut msg_recibido = false;
    for client_stream in listener.incoming() {
        let arc_server = server_ref.clone();
        thread_pool.execute(move || {
            let arc_client = Arc::new(client_stream.unwrap());
            server::server::handle_client(arc_server.clone(), arc_client)
                .expect("Error manejando cliente");
        });

        clients_array[i].send(msg_pass.clone());
        clients_array[i].send(register_array[i].clone());
        clients_array[i].send(msg_user.clone());

        let ten_millis = time::Duration::from_millis(100);

        thread::sleep(ten_millis);

        i = i + 1;

        if i == 1 {
            clients_array[0].send(msg_oper.clone());

            let ten_millis = time::Duration::from_millis(1000);

            thread::sleep(ten_millis);

            msg_recibido = server_ref.clone().users.lock().unwrap()[0].is_admin;
            //println!("recibidoo {}", recibido);

            break;
        }
    }

    assert_eq!(msg_recibido, true);
}

#[test]
fn two_users_join_channel() {
    let mut argv = Vec::new();
    argv.push("localhost".to_string());
    argv.push("8083".to_string());
    let host = argv[0].to_owned();
    let port = argv[1].parse::<u16>().expect("Puerto inválido");
    let servername = host.clone() + &port.to_string();

    let mut server = server::server::Server::new();
    server.set_name(servername.clone());
    let server_ref = Arc::new(server);

    let address = host.clone() + ":" + &port.to_string();
    //let sv_name = host
    //let mut srv = Server::new();
    //srv.name = host + &port.to_string();
    //let server = Arc::new(srv);
    let listener = TcpListener::bind(&address.clone()).unwrap();
    println!("Servidor configurado para escuchar en {}", &address);

    let mut clients_array: Vec<Client> = Vec::new();
    let mut register_array: Vec<String> = Vec::new();

    let client1 = Client::new("localhost:8083".to_string(), "juan".to_string());
    let client2 = Client::new("localhost:8083".to_string(), "martin".to_string());

    clients_array.push(client1);

    clients_array.push(client2);

    let msg_pass = "PASS 1234".to_string();

    let msg_nick1 = "NICK juan".to_string();

    let msg_nick2 = "NICK martin".to_string();

    let msg_user = "USER admin server localhost8080 real".to_string();

    let msg_join = "JOIN #canal1".to_string();

    register_array.push(msg_nick1);

    register_array.push(msg_nick2);

    let thread_pool = server::threadpool::ThreadPool::new(4);

    let mut i = 0;

    let mut cliente_en_canal1 = "".to_string();
    let mut cliente_en_canal2 = "".to_string();

    for client_stream in listener.incoming() {
        let arc_server = server_ref.clone();
        thread_pool.execute(move || {
            let arc_client = Arc::new(client_stream.unwrap());
            server::server::handle_client(arc_server.clone(), arc_client)
                .expect("Error manejando cliente");
        });

        clients_array[i].send(msg_pass.clone());
        clients_array[i].send(register_array[i].clone());
        clients_array[i].send(msg_user.clone());

        let ten_millis = time::Duration::from_millis(100);

        thread::sleep(ten_millis);

        i = i + 1;

        if i == 2 {
            clients_array[0].send(msg_join.clone());

            clients_array[1].send(msg_join.clone());

            let ten_millis = time::Duration::from_millis(1000);

            thread::sleep(ten_millis);

            cliente_en_canal1 = server_ref.clone().channels.lock().unwrap()[0].users[0].clone();
            cliente_en_canal2 = server_ref.clone().channels.lock().unwrap()[0].users[1].clone();

            break;
        }
    }

    assert_eq!(cliente_en_canal1, "juan".to_string());
    assert_eq!(cliente_en_canal2, "martin".to_string());
}
