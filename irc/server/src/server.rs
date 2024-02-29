use crate::interpreter::process_message;
use crate::message::Message;
use crate::registration::validate_connection;
use crate::replies::err_already_registred;
use crate::threadpool::ThreadPool;
use crate::user::User;

use crate::channel::Channel;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::{
    io::{BufRead, BufReader},
    net::TcpListener,
};
use std::{process, thread};
///Es la representación de un servidor de un sistema de Internet Relay Chat.
/// Permite alojar usuarios y canales y además es posible conectarse a otros servidores,
/// generando una red con topología spanning tree.
pub struct Server {
    pub name: String,
    pub users: Arc<Mutex<Vec<User>>>,
    pub channels: Arc<Mutex<Vec<Channel>>>,
    pub connected_servers: Arc<Mutex<Vec<Server>>>,
    pub socket: Option<Arc<TcpStream>>,
}
impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}
impl Server {
    ///Crea un nuevo servidor, inicializando sus atributos.
    pub fn new() -> Server {
        let name = String::new();
        let users = Arc::new(Mutex::new(Vec::new()));
        let channels = Arc::new(Mutex::new(Vec::new()));
        let connected_servers = Arc::new(Mutex::new(Vec::new()));
        let socket = None;
        Server {
            name,
            users,
            channels,
            connected_servers,
            socket,
        }
    }

    /// Evalúa si el servidor ya ha pasado por la etapa de registración.
    pub fn is_registered(&self) -> bool {
        !self.name.is_empty() && self.socket.is_some()
    }
    /// Evalúa si existe un camino entre el servidor y otro cuyo nombre es pasado por parámetro.
    pub fn is_connected_to(&self, servername: &String) -> bool {
        let connected_servers_lock = self.connected_servers.lock().unwrap();
        for connected_server in connected_servers_lock.iter() {
            if &connected_server.name == servername {
                return true;
            }

            if connected_server.is_connected_to(servername) {
                return true;
            }
        }

        false
    }
    /// Obtiene todos los servidores a los que se puede acceder desde el servidor dado.
    pub fn get_connected_servers_from(
        &self,
        servername: String,
    ) -> Option<Arc<Mutex<Vec<Server>>>> {
        let servers_lock = self.connected_servers.lock().unwrap();
        let opt_server = servers_lock
            .iter()
            .find(|con_ser| con_ser.name == servername);
        if let Some(found_server) = opt_server {
            Some(found_server.connected_servers.clone())
        } else {
            for server in servers_lock.iter() {
                let servers = server.get_connected_servers_from(servername.clone());
                if servers.is_some() {
                    return servers;
                }
            }
            None
        }
    }

    ///Se utiliza para enviarle un mensaje en forma de String a un servidor.
    pub fn send_message(&self, msg: String) -> Result<usize, std::io::Error> {
        if let Some(s) = &self.socket {
            let content: String = msg;
            s.as_ref().write((content + "\n").as_bytes())
        } else {
            Ok(0)
        }
    }
    ///Configura el socket del servidor.
    pub fn set_socket(&mut self, socket: Arc<TcpStream>) {
        self.socket = Some(socket);
    }
    ///Configura el nombre del servidor.
    pub fn set_name(&mut self, sv_name: String) {
        self.name = sv_name;
    }

    pub fn shutdown(&self) {
        println!("Server shutting down...");
        process::exit(1);
    }
}

///Atiende los mensajes de las conexiones recibidas por un servidor.
pub fn handle_client(server: Arc<Server>, socket: Arc<TcpStream>) -> std::io::Result<()> {
    let reader = BufReader::new(socket.as_ref());
    let lines = reader.lines();

    let user_nickname = register_connection(socket.clone(), server.clone());
    if let Some(aux_nickname) = user_nickname {
        process_client_messages(lines, server, aux_nickname, socket.clone());
    } else {
        process_server_messages(lines, server);
    }

    println!("Connection closed");
    Ok(())
}
///Procesa los mensajes emitidos por un cliente al servidor.
fn process_client_messages(
    mut lines: std::io::Lines<BufReader<&TcpStream>>,
    server: Arc<Server>,
    mut aux_nickname: String,
    socket: Arc<TcpStream>,
) {
    while let Some(Ok(line)) = lines.next() {
        let message = Message::from(line);
        let response = process_message(message, server.clone(), &mut aux_nickname);
        if let Ok(..) = response {
            for response_msg in response.unwrap().into_iter() {
                let line: String = response_msg.into();
                let _ = socket.as_ref().write((line + "\n").as_bytes());
            }
        }
    }
}
///Procesa los mensajes emitidos por otro servidor al servidor.
fn process_server_messages(mut lines: std::io::Lines<BufReader<&TcpStream>>, server: Arc<Server>) {
    while let Some(Ok(line)) = lines.next() {
        let message = Message::from(line);
        let mut sender_nick = "".to_string();
        let _ = process_message(message, server.clone(), &mut sender_nick);
    }
}
///Se encarga del proceso de registración, ya sea de un nuevo cliente o de un nuevo servidor que se
/// quiera conectar.
fn register_connection(socket: Arc<TcpStream>, current_server: Arc<Server>) -> Option<String> {
    println!("Nueva conexión entrante");

    let users = current_server.users.clone();
    let connected_servers = current_server.connected_servers.clone();

    let (user, server) = validate_connection(socket).expect("No se pudo registrar el usuario");

    if user.is_registered() {
        println!("Nuevo usuario registrado");
        let user_nickname = add_user_to_net(user, users, &connected_servers, current_server);
        println!("Usuario {} exitosamente agregado a la red", user_nickname);
        Some(user_nickname)
    } else {
        if !current_server.is_connected_to(&server.name) {
            // Se envía como respuesta exitosa de la conexión
            let msg = ":".to_string() + &current_server.name + " SERVER " + &current_server.name;
            let _ = server.send_message(msg);

            add_server_to_net(server, current_server);
        } else {
            let _ = server.send_message(err_already_registred().into());
        }
        None
    }
}

fn add_user_to_net(
    user: User,
    users: Arc<Mutex<Vec<User>>>,
    connected_servers: &Arc<Mutex<Vec<Server>>>,
    current_server: Arc<Server>,
) -> String {
    let current_server_name = current_server.name.clone();
    let user_nickname = user.nickname.to_owned();
    let mut mutex = users.lock().expect("No se pudo registrar el usuario");
    mutex.push(user);
    println!("Usuario agregado a la lista de usuarios online");
    let mut server_users = String::new();
    for u in mutex.iter() {
        server_users = server_users + &u.nickname + " ";
    }
    server_users.pop();
    println!("Users actuales: {}", server_users);
    let mut user_update_msg = format!(":{current_server_name} SERVER USER_LIST_UPDATE ");
    for user in mutex.iter() {
        let _ = user.update_server_users(server_users.clone());
        user_update_msg = user_update_msg
            + &user.nickname
            + " "
            + &user.server
            + " "
            + &user.username
            + " "
            + &user.hostname
            + " "
            + &user.realname
            + " ";
    }
    for connected_server in connected_servers.lock().unwrap().iter() {
        let _ = connected_server.send_message(user_update_msg.clone() + "\n");
        println!("Informando nuevo usuario a {}", connected_server.name);
    }
    user_nickname
}

fn add_server_to_net(new_server: Server, current_server: Arc<Server>) {
    let sv_new_name = new_server.name.clone();

    println!("Agregando servididor {} a la red", &sv_new_name);
    let mut connected_servers_lock = current_server.connected_servers.lock().unwrap();

    notify_new_server_to_net(
        &new_server.name,
        &connected_servers_lock,
        &current_server.name,
    );
    exchange_servers_list(
        &current_server.name,
        &new_server,
        &connected_servers_lock,
        1,
    );
    exchange_users_list(current_server.clone(), &new_server);

    exchange_channel_list(current_server.clone(), &new_server);

    connected_servers_lock.push(new_server);
    println!("Servidor {} agregado a la red", &sv_new_name);

    // Debugging purposes:
    drop(connected_servers_lock);
    show_spanning_tree(&current_server, 0);
}

// Recorre cada rama del spanning tree y le informa los servidores conectados a la red al servidor que acaba de iniciar la conexión
fn exchange_servers_list(
    root_server_name: &String,
    new_server: &Server,
    connected_servers_lock: &std::sync::MutexGuard<Vec<Server>>,
    hopcount: usize,
) {
    for connected_server in connected_servers_lock.iter() {
        println!(
            "Enviando información del servidor {} con nodo raíz {} (hopcount: {}) al nuevo servidor {}",
            connected_server.name, root_server_name, hopcount, new_server.name
        );

        let server_name = connected_server.name.clone();
        let server_connection_msg = format!(":{root_server_name} SERVER {server_name} {hopcount}");
        let _ = new_server.send_message(server_connection_msg);
        let connected_server_childs_lock = connected_server.connected_servers.lock().unwrap();
        exchange_servers_list(
            &connected_server.name,
            new_server,
            &connected_server_childs_lock,
            hopcount + 1,
        );
    }
}

fn notify_new_server_to_net(
    new_server_name: &String,
    connected_servers_lock: &std::sync::MutexGuard<Vec<Server>>,
    current_server_name: &String,
) {
    println!("Notificando red sobre nueva conexión");
    let server_connection_msg = format!(":{current_server_name} SERVER {new_server_name} 2");
    for connected_server in connected_servers_lock.iter() {
        let _ = connected_server.send_message(server_connection_msg.clone());
    }
}

fn exchange_users_list(current_server: Arc<Server>, server: &Server) {
    println!("Enviando lista de usuarios a server nuevo");
    let current_server_name = current_server.name.clone();
    let mut user_update_msg = format!(":{current_server_name} SERVER USER_LIST_UPDATE ");
    let users_lock = current_server.users.lock().unwrap();

    for user in users_lock.iter() {
        user_update_msg = user_update_msg
            + &user.nickname
            + " "
            + &user.server
            + " "
            + &user.username
            + " "
            + &user.hostname
            + " "
            + &user.realname
            + " ";
    }
    let _ = server.send_message(user_update_msg);
    println!("Lista de usuarios enviada");
}
fn exchange_channel_list(current_server: Arc<Server>, server: &Server) {
    println!("Enviando lista de canales a server nuevo");
    let current_server_name = current_server.name.clone();
    let mut channel_update_msg = format!(":{current_server_name} SERVER CHANNEL_LIST_UPDATE ");
    let channels_lock = current_server.channels.lock().unwrap();

    for channel in channels_lock.iter() {
        if channel.name.starts_with('&') {
            channel_update_msg = channel_update_msg
                + &channel.name
                + " "
                + &channel.get_users_list()
                + " "
                + &channel.get_topic_option()
                + " "
                + &channel.get_admins_list()
                + " "
                + &channel.get_limit_option()
                + " "
                + &channel.get_ban_list()
                + " "
                + &channel.get_can_speak_users_list()
                + " "
                + &channel.get_password_option()
                + " ";
        }
    }
    println!("{}", channel_update_msg);
    let _ = server.send_message(channel_update_msg);
    println!("Lista de canales enviada");
}

// Este método tiene como fin debuggear la estructura
pub fn show_spanning_tree(server: &Server, level: usize) {
    let spaces = " ".repeat(level);
    println!("{}>{}", spaces, server.name);
    let inner_servers_lock = server.connected_servers.lock().unwrap();
    for inner_server in inner_servers_lock.iter() {
        //println!("{}Servidor {} anidado bajo servidor {}", spaces, inner_server.name, server.name);
        show_spanning_tree(inner_server, level + 2);
    }
    //println!("{}{} --------------------------------", spaces, server.name);
}

pub fn attempt_server_conection(server: Arc<Server>, input: String, servername: String) {
    let lines: Vec<_> = input.split(' ').collect();

    if lines.len() == 2 {
        let address: String = lines[0].to_string() + ":" + lines[1];
        println!("Conectando a servidor {}", address);

        let socket = TcpStream::connect(address.clone()).unwrap();
        let socket_ref = Arc::new(socket);
        let reader = BufReader::new(socket_ref.as_ref());
        let content = "SERVER ".to_string() + &servername + " 1\n";
        let _ = socket_ref.as_ref().write(content.as_bytes());

        let new_server_name = lines[0].to_string() + lines[1];

        let mut lines = reader.lines();
        if let Some(Ok(line)) = lines.next() {
            let msg = Message::from(line);

            if msg.command.as_str() == "SERVER" {
                println!("Conexion nueva aceptada");

                let mut new_server = Server::new();
                new_server.set_socket(socket_ref.clone());
                new_server.set_name(new_server_name);

                add_server_to_net(new_server, server.clone());

                let thread_sv_ref = server;

                let _ = thread::spawn(move || {
                    println!("Listo para escuchar mensajes del nuevo servidor");

                    let socket_thread_ref = socket_ref.clone();
                    let reader = BufReader::new(socket_thread_ref.as_ref());
                    let mut lines = reader.lines();

                    while let Some(Ok(line)) = lines.next() {
                        let message = Message::from(line);
                        let mut sender_nick = "".to_string();
                        let _ = process_message(message, thread_sv_ref.clone(), &mut sender_nick);
                    }
                });

                println!("Conexión exitosa con servidor {}", address);
            } else {
                println!("Conexión con servidor {} fallida", address);
            }
        }
    }
}
///Arranca la ejecución de un servidor, permitiéndole recibir nuevas conexiones y mensajes.
pub fn run(server: Arc<Server>, host: String, port: u16) -> std::io::Result<()> {
    let address = host + ":" + &port.to_string();

    let listener = TcpListener::bind(&address)?;
    println!("Servidor configurado para escuchar en {}", &address);

    let thread_pool = ThreadPool::new(4);
    for client_stream in listener.incoming() {
        let client_ref = Arc::new(client_stream?);
        let server_ref = server.clone();
        thread_pool.execute(move || {
            handle_client(server_ref.clone(), client_ref).expect("Error manejando cliente");
        });
    }
    Ok(())
}
