use std::{
    net::Shutdown,
    sync::{Arc, Mutex},
};

use crate::channel::Channel;
use crate::{
    message::Message,
    replies::err_no_privileges,
    server::{show_spanning_tree, Server},
    server_errors::ServerError,
    user::User,
};

pub fn server_msg(msg: Message, server: Arc<Server>) -> Result<Vec<Message>, ServerError> {
    let response_vec = Vec::new();

    match msg.parameters[0].as_str() {
        "USER_LIST_UPDATE" => {
            process_users_list_notification(&server, &msg);
        }
        "CHANNEL_LIST_UPDATE" => {
            process_channel_list_notification(&server, &msg);
        }
        _ => process_server_notification(msg, server),
    }
    Ok(response_vec)
}

fn process_server_notification(msg: Message, server: Arc<Server>) {
    println!("Agregando nuevo servidor al modelo");
    let parameters = msg.parameters;
    let mut hopcount: usize = parameters[1].clone().parse().unwrap();
    let connected_servers = server.get_connected_servers_from(msg.prefix.clone().unwrap());

    if let Some(connected_servers) = connected_servers {
        let mut new_server_node = Server::new();
        new_server_node.set_name(parameters[0].clone());
        connected_servers.lock().unwrap().push(new_server_node);

        let connected_servers_lock = server.connected_servers.lock().unwrap();

        for connected_server in connected_servers_lock.iter() {
            if !connected_server.is_connected_to(&parameters[0].clone()) {
                //No devuelvo el mensaje a quien me lo mandó
                hopcount += 1;
                let new_msg = format!(
                    ":{} SERVER {} {}",
                    msg.prefix.clone().unwrap(),
                    parameters[0].clone(),
                    hopcount
                );
                let _ = connected_server.send_message(new_msg);
            }
        }
    }
    println!("Servidor {} exitosamente añadido al modelo", parameters[0]);

    // Debugging purposes:
    show_spanning_tree(&server, 0);
}

fn process_users_list_notification(server: &Arc<Server>, msg: &Message) {
    println!("Actualizando lista de usuarios por movimiento en otro servidor");
    let mut i: usize = 1;
    let mut current_sv_users = server.users.lock().unwrap();
    let current_sv_name = server.name.clone();
    current_sv_users.retain(|user| user.server == current_sv_name);
    while i < msg.parameters.len() - 1 {
        let new_user_server = msg.parameters[i + 1].clone();

        if new_user_server != current_sv_name {
            let mut new_user = User::new(None);
            new_user.set_nickname(msg.parameters[i].clone());
            new_user.set_server(new_user_server);
            new_user.set_host(msg.parameters[i + 3].clone());
            new_user.set_username(msg.parameters[i + 2].clone());
            new_user.set_realname(msg.parameters[i + 4].clone());
            current_sv_users.push(new_user);
        }

        i += 5;
    }
    let mut server_users = String::new();
    for u in current_sv_users.iter() {
        server_users = server_users + &u.nickname + " ";
    }
    server_users.pop();
    for u in current_sv_users.iter() {
        let _ = u.update_server_users(server_users.clone());
    }
    let connected_servers_lock = server.connected_servers.lock().unwrap();
    for connected_server in connected_servers_lock.iter() {
        if connected_server.name != msg.prefix.clone().unwrap() {
            println!("{}", connected_server.name);
            println!("{}", msg.prefix.clone().unwrap());
            let mut mew_msg = msg.clone();
            mew_msg.prefix = Some(server.name.clone());
            let _ = connected_server.send_message(String::from(mew_msg));
        }
    }
    println!("Lista de usuarios actualizada exitosamente por movimiento en otro servidor");
}
fn process_channel_list_notification(server: &Arc<Server>, msg: &Message) {
    println!("Actualizando lista de canales por movimiento en otro servidor");
    let mut i: usize = 1;
    let mut current_sv_channels = server.channels.lock().unwrap();
    let current_sv_name = server.name.clone();
    while i < msg.parameters.len() - 1 {
        let new_channel_server = msg.parameters[i + 1].clone();

        if new_channel_server != current_sv_name {
            let mut new_channel = Channel::new(&msg.parameters[i].clone());
            new_channel.set_users(msg.parameters[i + 1].clone());
            new_channel.set_topic(msg.parameters[i + 2].clone());
            new_channel.set_admins(msg.parameters[i + 3].clone());
            new_channel.set_limit(msg.parameters[i + 4].clone());
            new_channel.set_ban_list(msg.parameters[i + 5].clone());
            new_channel.set_speak_users(msg.parameters[i + 6].clone());
            new_channel.set_password(msg.parameters[i + 7].clone());

            current_sv_channels.push(new_channel);
        }

        i += 8;
    }
    // let mut server_users = String::new();
    // for u in current_sv_users.iter() {
    //     server_users = server_users + &u.nickname + " ";
    // }
    // server_users.pop();
    // for u in current_sv_users.iter() {
    //     let _ = u.update_server_users(server_users.clone());
    // }
    let connected_servers_lock = server.connected_servers.lock().unwrap();
    for connected_server in connected_servers_lock.iter() {
        if connected_server.name != msg.prefix.clone().unwrap() {
            println!("{}", connected_server.name);
            println!("{}", msg.prefix.clone().unwrap());
            let mut mew_msg = msg.clone();
            mew_msg.prefix = Some(server.name.clone());
            let _ = connected_server.send_message(String::from(mew_msg));
        }
    }
    println!("Lista de canales actualizada exitosamente por movimiento en otro servidor");
}
/// Se encarga de interpretar el mensaje de SQUIT enviado a un servidor. Desconecta al servidor
/// del que se acaba de ir de la red y además distribuye esta información al resto de los servidores
/// conectados.
pub fn squit_msg(
    msg: Message,
    server: Arc<Server>,
    user_nickname: &String,
) -> Result<Vec<Message>, ServerError> {
    let mut response_vec = Vec::new();
    if msg.prefix.is_none() {
        // Sin prefijo, es un operador intentando desconectar un servidor
        println!(
            "Operador solicitando baja de servidor {}",
            msg.parameters[0]
        );
        process_squit_msg_from_oper(server, user_nickname, &mut response_vec, &msg)?;
    } else {
        // Con prefijo, es un servidor informando la desconexión de un servidor al resto de la red
        println!("Servidor informando baja de servidor {}", msg.parameters[0]);
        process_squit_msg_from_server(&msg, server);
    }

    Ok(response_vec)
}

fn process_squit_msg_from_oper(
    server: Arc<Server>,
    user_nickname: &String,
    response_vec: &mut Vec<Message>,
    msg: &Message,
) -> Result<(), ServerError> {
    let mut lock_users = server
        .users
        .lock()
        .map_err(|_e| ServerError::new("Cannot obtain users list"))?;
    let current_user = lock_users
        .iter_mut()
        .find(|user| user.nickname == *user_nickname)
        .ok_or_else(|| ServerError::new("Cannot obtain current user"))?;
    if !current_user.is_admin {
        let err_no_privileges = err_no_privileges();
        response_vec.push(err_no_privileges)
    } else {
        drop(lock_users);
        let mut msg = msg.clone();
        msg.prefix = Some("".to_string()); // Prefijo vacío para reenviar a todos los servidores
        process_squit_msg_from_server(&msg, server);
    };
    Ok(())
}

fn process_squit_msg_from_server(msg: &Message, server: Arc<Server>) {
    forward_squit_msg(msg, server.clone());
    if msg.parameters[0] == server.name {
        // Si el server a desconectar es este
        println!("Solicitud de baja de este servidor recibida");
        disconnect_users_from_server_shutting_down(server.clone());
        server.shutdown();
    } else {
        println!("Baja de otro servidor de la red informada");
        let removed_server =
            remove_server_from_network(server.connected_servers.clone(), &msg.parameters[0]);
        remove_disconnected_server_users(server.clone(), removed_server.unwrap());
        notify_disconnected_users_to_clients(server);
    }
}

fn notify_disconnected_users_to_clients(server: Arc<Server>) {
    let users_lock = server.users.lock().unwrap();
    let mut server_users = String::new();
    for u in users_lock.iter() {
        server_users = server_users + &u.nickname + " ";
    }
    server_users.pop();
    for user in users_lock.iter() {
        let _ = user.update_server_users(server_users.clone());
    }
    println!("Current clients notified of disconnected users");
}

fn remove_disconnected_server_users(server: Arc<Server>, disconnected_server: Server) {
    let mut inner_servers: Vec<String> = Vec::new();
    retrieve_inner_servers(&disconnected_server, &mut inner_servers);

    let mut users_lock = server.users.lock().unwrap();
    // Retener a aquellos usuarios que no estén en el servidor desconectado ni en uno de sus nodos subyacentes
    users_lock.retain(|user| {
        user.server != disconnected_server.name && !inner_servers.contains(&user.server)
    });
}

fn retrieve_inner_servers(root_server: &Server, vec_inner_servers: &mut Vec<String>) {
    let inner_servers_lock = root_server.connected_servers.lock().unwrap();
    for srv in inner_servers_lock.iter() {
        println!(
            "Se removerán también usuarios del servidor {} por desconexión del servidor {}",
            srv.name, root_server.name
        );
        vec_inner_servers.push(srv.name.clone());
        retrieve_inner_servers(srv, vec_inner_servers);
    }
}

fn disconnect_users_from_server_shutting_down(server: Arc<Server>) {
    let users_lock = server.users.lock().unwrap();
    for user in users_lock.iter() {
        if user.socket.is_some() {
            println!(
                "Desconectando usuario {} por cierre de servidor",
                user.nickname
            );
            let usr_quit_msg = Message::from("QUIT :Server shutting down".to_string());
            let _ = user.send_private_message(usr_quit_msg);
            user.socket
                .as_ref()
                .unwrap()
                .shutdown(Shutdown::Both)
                .expect("client shutdown call failed");
        }
    }
}

fn forward_squit_msg(msg: &Message, server: Arc<Server>) {
    let mut squit_msg = msg.clone();
    squit_msg.prefix = Some(server.name.clone());
    let connected_servers_lock = server.connected_servers.lock().unwrap();
    for connected_server in connected_servers_lock.iter() {
        if connected_server.name.clone() != msg.prefix.clone().unwrap() {
            // No le devuelvo el mensaje a quien me lo mandó
            let _ = connected_server.send_message(squit_msg.clone().into());
        }
    }
}
pub fn forward_invite_msg(msg: Message, connected_servers: Arc<Mutex<Vec<Server>>>) {
    let connected_servers_lock = connected_servers.lock().unwrap();
    if msg.prefix.is_some() {
        for connected_server in connected_servers_lock.iter() {
            if connected_server.name.clone() != msg.prefix.clone().unwrap() {
                // No le devuelvo el mensaje a quien me lo mandó
                let _ = connected_server.send_message(msg.clone().into());
            }
        }
    }
}

// Se recorre recursivamente el árbol para desconectar los servidores o la red de servidores que esté detrás del nodo que se intenta desconectar
fn remove_server_from_network(
    connected_servers: Arc<Mutex<Vec<Server>>>,
    disconnected_server_name: &String,
) -> Option<Server> {
    let mut connected_servers_lock = connected_servers.lock().unwrap();
    for (pos, connected_server) in connected_servers_lock.iter().enumerate() {
        if &connected_server.name == disconnected_server_name {
            return Some(connected_servers_lock.remove(pos));
        }
        let removed_server = remove_server_from_network(
            connected_server.connected_servers.clone(),
            disconnected_server_name,
        );
        if removed_server.is_some() {
            return removed_server;
        }
    }
    None
}
