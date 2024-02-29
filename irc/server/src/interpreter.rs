use std::io::Write;
use std::sync::{Arc, Mutex};
use std::vec;

use crate::server::attempt_server_conection;
use crate::server_messages_interpreter::{server_msg, squit_msg};

use crate::channel::{
    invite_msg, join_msg, list_msg, mode_msg, names_msg, oper_ch_msg, part_msg, topic_msg, Channel,
};
use crate::{message::Message, server::Server, user::User};

use crate::replies::{
    err_already_registred, err_can_not_send_to_chan, err_chan_opriv_is_needed,
    err_need_more_params, err_nickname_in_use, err_no_nickname_given, err_no_oper_host,
    err_no_privileges, err_no_recpient, err_no_such_channel, err_no_such_nick, err_no_text_tosend,
    error_no_such_nick, rpl_away, rpl_unaway, rpl_who_reply, rpl_whoisuser, rpl_you_are_oper,
};
use crate::server_errors::ServerError;
///Recibe el mensaje que fue emitido a un servidor y deriva su
/// procesamiento a la función correspondiente.
pub fn process_message(
    msg: Message,
    server: Arc<Server>,
    user_nickname: &mut String,
) -> Result<Vec<Message>, ServerError> {
    match msg.command.as_str() {
        "PASS" => password_msg(msg),
        "USER" => user_msg(msg),
        "NICK" => nick_msg(msg, server.users.clone(), user_nickname),

        "PRIVMSG" => priv_msg(msg, server, user_nickname.to_string()),
        "QUIT" => quit_msg(msg, user_nickname, server.users.clone()),
        "NOTICE" => notice_msg(
            msg,
            server.users.clone(),
            user_nickname.to_string(),
            server.channels.clone(),
        ),
        "JOIN" => join_msg(
            msg,
            user_nickname.to_string(),
            server.users.clone(),
            server.channels.clone(),
            server.connected_servers.clone(),
            server.name.clone(),
        ),
        "PART" => part_msg(
            msg,
            user_nickname.to_string(),
            server.users.clone(),
            server.channels.clone(),
        ),
        "OPER" => oper_msg(msg, server.users.clone(), &user_nickname.to_string()),
        "KICK" => kick_msg(
            msg,
            server.users.clone(),
            server.channels.clone(),
            user_nickname,
        ),
        "OPERCH" => oper_ch_msg(
            msg,
            server.users.clone(),
            server.channels.clone(),
            user_nickname.to_string(),
        ),
        "INVITE" => invite_msg(
            msg,
            server.users.clone(),
            user_nickname.to_string(),
            server.channels.clone(),
        ),
        "NAMES" => names_msg(
            msg,
            server.users.clone(),
            user_nickname.to_string(),
            server.channels.clone(),
        ),
        "LIST" => list_msg(
            msg,
            user_nickname.to_string(),
            server.users.clone(),
            server.channels.clone(),
        ),
        "WHOIS" => whois_msg(msg, server.users.clone()),
        "WHO" => who_msg(msg, server.users.clone()),
        "TOPIC" => topic_msg(msg, user_nickname.to_string(), server.channels.clone()),
        "AWAY" => away_msg(msg, server.users.clone()),
        "MODE" => mode_msg(msg, user_nickname.to_string(), server.channels.clone()),
        "SERVER" => server_msg(msg, server),
        "SQUIT" => squit_msg(msg, server, &user_nickname.to_string()),
        "SERVER_CONNECT" => attempt_sv_connection(msg, server, user_nickname.to_string()),
        _ => {
            println!("Comando inválido");
            Err(ServerError::new("Comando invalido"))
        }
    }
}
/// Se encarga de interpretar el mensaje de KICK enviado a un servidor
/// y en caso de éxito quitar al usuario del canal.
pub fn kick_msg(
    msg: Message,
    users: Arc<Mutex<Vec<User>>>,
    channels: Arc<Mutex<Vec<Channel>>>,
    nick: &mut String,
) -> Result<Vec<Message>, ServerError> {
    let mut response_vector = Vec::new();
    let mut lock = channels
        .lock()
        .map_err(|_e| ServerError::new("Cannot obtain channels list"))?;
    let mut lock_users = users
        .lock()
        .map_err(|_e| ServerError::new("Cannot obtain users list"))?;
    if msg.parameters.len() < 2 {
        let need_more_params_message = err_need_more_params(msg.command.clone());
        response_vector.push(need_more_params_message);
    }
    let current_user = lock_users
        .iter_mut()
        .find(|user| user.nickname == *nick)
        .ok_or_else(|| ServerError::new("Cannot obtain curren user"))?;
    match lock
        .iter_mut()
        .find(|channel| channel.name == msg.parameters[0])
        .ok_or(())
    {
        Ok(current_channel) => {
            if current_channel.is_admin(&current_user.nickname) {
                current_channel.remove_user(&msg.parameters[1]);
            } else {
                let chanopriv = err_chan_opriv_is_needed(current_channel.name.clone());
                response_vector.push(chanopriv);
            }
        }
        Err(()) => {
            let no_channel = err_no_such_channel(msg.parameters[0].clone());
            response_vector.push(no_channel);
        }
    }

    Ok(response_vector)
}
/// Se encarga de interpretar el mensaje de PASS enviado a un servidor
/// y si se quiere cambiar la contraseña, notifica que el usuario ya está registrado.
pub fn password_msg(msg: Message) -> Result<Vec<Message>, ServerError> {
    let mut response_vector = Vec::new();
    if msg.parameters.len() != 1 {
        let need_more_params_message = err_need_more_params(msg.command);
        response_vector.push(need_more_params_message);
    } else {
        let already_registered = err_already_registred();
        response_vector.push(already_registered)
    }
    Ok(response_vector)
}
/// Se encarga de interpretar el mensaje de NICK enviado a un servidor
/// y en caso de éxito permite que un usuario se cambie el nickname.
pub fn nick_msg(
    msg: Message,
    users: Arc<Mutex<Vec<User>>>,
    nick: &mut String,
) -> Result<Vec<Message>, ServerError> {
    let mut response_vector = Vec::new();
    let mut lock = users.lock().unwrap();
    let aux_lock = &lock.clone();
    if msg.parameters.is_empty() {
        let no_nick = err_no_nickname_given();
        response_vector.push(no_nick);
        return Ok(response_vector);
    }
    let mut nick_in_use = false;
    for user in lock.iter() {
        if user.nickname == msg.parameters[0] {
            nick_in_use = true;
            break;
        }
    }

    let mut current_user = lock.iter_mut().find(|user| user.nickname == *nick).unwrap();

    match nick_in_use {
        true => {
            let nick_repetido = err_nickname_in_use(msg.parameters[0].to_string());
            response_vector.push(nick_repetido);
        }
        false => {
            *nick = msg.parameters[0].to_string();
            let mut server_users = String::new();
            for u in aux_lock.iter() {
                if u.nickname == current_user.nickname {
                    server_users = server_users + &msg.parameters[0].to_string() + " ";
                } else {
                    server_users = server_users + &u.nickname + " ";
                }
            }
            for u in aux_lock.iter() {
                let _ = u.update_server_users(server_users.clone());
            }
            current_user.nickname = msg.parameters[0].to_string();
            println!("Nickname changed to {}", msg.parameters[0]);
        }
    }
    Ok(response_vector)
}
/// Se encarga de interpretar el mensaje de USER enviado a un servidor
/// y si se quiere cambiar el username, notifica que el usuario ya está registrado.
pub fn user_msg(msg: Message) -> Result<Vec<Message>, ServerError> {
    let mut response_vector = Vec::new();
    if msg.parameters.len() != 4 {
        let need_more_params_message = err_need_more_params(msg.command);
        response_vector.push(need_more_params_message);
    } else {
        let already_registered = err_already_registred();
        response_vector.push(already_registered)
    }
    Ok(response_vector)
}
/// Se encarga de interpretar el mensaje de QUIT enviado a un servidor
/// y en caso de éxito desconecta al usuario del servidor.
pub fn quit_msg(
    msg: Message,
    nickname: &String,
    users: Arc<Mutex<Vec<User>>>,
) -> Result<Vec<Message>, ServerError> {
    let mut users_list = users
        .lock()
        .map_err(|_e| ServerError::new("Cannot obtain users list"))?;
    let index = users_list
        .iter()
        .position(|x| &x.nickname == nickname)
        .unwrap();
    users_list.remove(index);
    // Al borrarlo de la lista, se pierde el ownership y se dropea el usuario
    println!("Usuario desconectado!: {:?}", msg.parameters[0]);

    let mut server_users = String::new();

    for u in users_list.iter() {
        server_users = server_users + &u.nickname + " ";
    }
    server_users.pop();

    for u in users_list.iter() {
        let _ = u.update_server_users(server_users.clone());
    }

    Ok(Vec::new())
}
/// Se encarga de interpretar el mensaje de PRIVMSG enviado a un servidor
/// y en caso de éxito enviar un mensaje privado a un canal o a un usuario
pub fn priv_msg(
    msg: Message,
    server: Arc<Server>,
    nick: String,
) -> Result<Vec<Message>, ServerError> {
    let mut response_vector = Vec::new();

    let users = server.users.clone();
    let channels = server.channels.clone();

    let users_lock = users.lock().unwrap();

    for u in users_lock.iter() {
        println!("Usuarios conectados (antes del privsmg): {}", u.nickname);
    }

    let channel_lock = channels.lock().unwrap();

    if msg.parameters[0].is_empty() {
        let no_recipient = err_no_recpient(msg.command);
        response_vector.push(no_recipient);
        return Ok(response_vector);
    }

    if msg.parameters[1].is_empty() {
        let no_text = err_no_text_tosend();
        response_vector.push(no_text);
        return Ok(response_vector);
    }

    for receiver in msg.parameters[0].split(',') {
        // Receiver is a channel
        if receiver.starts_with('#') || receiver.starts_with('&') {
            if let Some(channel) = channel_lock.iter().find(|channel| channel.name == receiver) {
                //si el canal no puede recibir mensajes externos pero el usuario esta adentro, o si el canal es moderado y el usuario puede hablar o si no hay restricciones de quien manda mensajes, se manda el mensaje
                if channel.is_no_msg_outside() && channel.has_user(&nick)
                    || channel.is_moderated() && channel.can_send_msg(&nick)
                    || channel.is_not_msg_restricted()
                {
                    for user_name in &channel.users {
                        if let Some(recipient) =
                            users_lock.iter().find(|user| &user.nickname == user_name)
                        {
                            if &recipient.nickname != msg.prefix.as_ref().unwrap() {
                                if recipient.socket.is_some() {
                                    send_message_to_user(
                                        recipient,
                                        &msg,
                                        &nick,
                                        &mut response_vector,
                                    );
                                } else {
                                    for connected_server in
                                        server.connected_servers.lock().unwrap().iter()
                                    {
                                        println!("{} {}", connected_server.name, recipient.server);

                                        if recipient.server == connected_server.name
                                            || connected_server.is_connected_to(&recipient.server)
                                        {
                                            println!("Camino hacia usuario");
                                            let message = String::from(msg.clone());

                                            let _ = connected_server
                                                .socket
                                                .as_ref()
                                                .unwrap()
                                                .as_ref()
                                                .write((message + "\n").as_bytes());
                                            break;
                                        }
                                    }
                                }
                                // let _ = recipient.send_private_message(msg.clone());
                            }
                        } else {
                            let no_such_nick_msg = error_no_such_nick(receiver.to_string());
                            response_vector.push(no_such_nick_msg)
                        }
                    }
                } else {
                    let not_send_to_chan = err_can_not_send_to_chan(channel.name.clone());
                    response_vector.push(not_send_to_chan);
                }
            } else {
                let no_such_nick_msg = error_no_such_nick(receiver.to_string());
                response_vector.push(no_such_nick_msg)
            }
        }
        // Receiver is a user
        else if let Some(recipient) = users_lock.iter().find(|user| user.nickname == receiver) {
            if recipient.socket.is_some() {
                send_message_to_user(recipient, &msg, &nick, &mut response_vector);
            } else {
                for connected_server in server.connected_servers.lock().unwrap().iter() {
                    println!("{} {}", connected_server.name, recipient.server);

                    if recipient.server == connected_server.name
                        || connected_server.is_connected_to(&recipient.server)
                    {
                        println!("Camino hacia usuario");
                        let message = String::from(msg.clone());

                        let _ = connected_server
                            .socket
                            .as_ref()
                            .unwrap()
                            .as_ref()
                            .write((message + "\n").as_bytes());
                        break;
                    }
                }
            }
        } else {
            let no_such_nick_msg = error_no_such_nick(receiver.to_string());
            response_vector.push(no_such_nick_msg)
        }
    }

    Ok(response_vector)
}

fn send_message_to_user(
    recipient: &User,
    msg: &Message,
    nick: &String,
    response_vector: &mut Vec<Message>,
) {
    let _ = recipient.send_private_message(msg.clone());
    if let Some(away_message) = &recipient.away_message {
        println!("User is AFK. Autoreplying...");
        autoreply(recipient, nick, away_message, response_vector);
    }
}

fn autoreply(
    recipient: &User,
    nickname: &String,
    away_message: &String,
    response_vector: &mut Vec<Message>,
) {
    let sender_nickname = &recipient.nickname;
    let recipient_nickname = nickname;
    let automatic_reply_msg = Message::from(format!(
        ":{sender_nickname} PRIVMSG {recipient_nickname} :[Mensaje automático] {away_message}"
    ));
    response_vector.push(automatic_reply_msg);
}

/// Se encarga de interpretar el mensaje de NOTICE enviado a un servidor
/// y en caso de éxito enviar un mensaje privado a un canal o a un usuario,
/// sin recibir respuestas automáticas.
pub fn notice_msg(
    msg: Message,
    users: Arc<Mutex<Vec<User>>>,
    nick: String,
    channels: Arc<Mutex<Vec<Channel>>>,
) -> Result<Vec<Message>, ServerError> {
    let response_vector = Vec::new();
    let users_lock = users.lock().unwrap();
    let channel_lock = channels.lock().unwrap();
    if msg.parameters[0].is_empty() {
        return Ok(response_vector);
    }

    if msg.parameters[1].is_empty() {
        return Ok(response_vector);
    }
    for receiver in msg.parameters[0].split(',') {
        if receiver.starts_with('#') || receiver.starts_with('&') {
            if let Some(channel) = channel_lock.iter().find(|channel| channel.name == receiver) {
                //si el canal no puede recibir mensajes externos pero el usuario esta adentro, o si el canal es moderado y el usuario puede hablar o si no hay restricciones de quien manda mensajes, se manda el mensaje
                if channel.is_no_msg_outside() && channel.has_user(&nick)
                    || channel.is_moderated() && channel.can_send_msg(&nick)
                    || channel.is_not_msg_restricted()
                {
                    for user_name in &channel.users {
                        if let Some(recipient) =
                            users_lock.iter().find(|user| &user.nickname == user_name)
                        {
                            let _ = recipient.send_private_message(msg.clone());
                        }
                    }
                }
            }
        } else if let Some(recipient) = users_lock.iter().find(|user| user.nickname == receiver) {
            let _ = recipient.send_private_message(msg.clone());
        }
    }

    Ok(response_vector)
}

/*pub fn notice_msg() -> Result<(), ()> {
    println!("notice");
    Ok(())
}*/
/// Se encarga de interpretar el mensaje de OPER enviado a un servidor
/// y en caso de éxito le otorga a un usuario privilegios de operador sobre la red de servidores.
pub fn oper_msg(
    msg: Message,
    users: Arc<Mutex<Vec<User>>>,
    nickname: &String,
) -> Result<Vec<Message>, ServerError> {
    let mut response_vector = Vec::new();

    if msg.parameters.is_empty() {
        let more_params = err_need_more_params(msg.command);
        response_vector.push(more_params);
        println!("Incorrect number of parameters in message");
        return Ok(response_vector);
    }

    let username = msg.parameters[0].as_str();
    let password = msg.parameters[1].as_str();
    if username == "admin" && password == "1234" {
        let mut users_list = users
            .lock()
            .map_err(|_e| ServerError::new("Cannot filter users"))?;
        let user = users_list
            .iter_mut()
            .find(|u| &u.nickname == nickname)
            .ok_or_else(|| ServerError::new("Cannot get user"))?;

        user.become_admin();
        response_vector.push(rpl_you_are_oper());
    } else {
        println!("User or password incorrect for operator");
        response_vector.push(err_no_oper_host());
    }

    Ok(response_vector)
}

//permite devolver la info de màs de un usuario solo si hay mas de un server (no puede haber dos usuarios con el
//mismo nick en un mismo server)
/// Se encarga de interpretar el mensaje de WHOIS enviado a un servidor
/// y en caso de éxito brinda la información de un determinado usuario.
pub fn whois_msg(msg: Message, users: Arc<Mutex<Vec<User>>>) -> Result<Vec<Message>, ServerError> {
    let mut response_vector = Vec::new();
    let lock_users = users.lock().unwrap();

    if msg.parameters.is_empty() {
        println!("Incorrect number of parameters in message");

        let more_params = err_need_more_params(msg.command);
        response_vector.push(more_params);
        return Ok(response_vector);
    }
    let mut no_matches = true;
    for user in lock_users.iter() {
        for param in msg.parameters.clone() {
            if user.nickname == param {
                no_matches = false;
                let own_string: String = "".to_owned();
                let aux = stringfy_user_info(own_string, user);
                let rlp_whois = rpl_whoisuser(aux);
                response_vector.push(rlp_whois);
                break;
            }
        }
    }
    if no_matches {
        let no_nick = err_no_such_nick();
        response_vector.push(no_nick);
        println!("There are not matches.");
    }

    Ok(response_vector)
}

fn stringfy_user_info(mut own_string: String, user_requested: &User) -> String {
    own_string.push_str(&user_requested.nickname);
    own_string.push(' ');

    own_string.push_str(&user_requested.username);
    own_string.push(' ');

    own_string.push_str(&user_requested.hostname);
    own_string.push(' ');

    own_string.push_str("* :");
    own_string.push_str(&user_requested.realname);

    own_string
}

//si no se le pasa ningun parametro a who, devuelve todos los usuarios
//si no devuelve algùn match si lo hay
/// Se encarga de interpretar el mensaje de WHO enviado a un servidor.
/// Si  no recibe parámetros, se devuelve la información de todos los usuarios. En caso contrario
/// brinda la información sobre el usuario solicitado, si este existe.
pub fn who_msg(msg: Message, users: Arc<Mutex<Vec<User>>>) -> Result<Vec<Message>, ServerError> {
    let mut response_vector = Vec::new();
    let lock_user = users.lock().unwrap();
    let mut own_string: String = " ".to_owned();

    if msg.parameters.is_empty() {
        for user in lock_user.iter() {
            own_string.push_str(&user.nickname);
            own_string.push(' ');
        }
        println!("usuarios listados{:?}", users);
        let who_reply = rpl_who_reply(own_string);
        response_vector.push(who_reply);
    } else {
        let users_iter = lock_user.iter();
        let mut users_to_display: Vec<&User> = vec![];

        if msg.parameters.len() == 2 {
            match msg.parameters[1].as_str() {
                "o" => {
                    for user in users_iter {
                        if (user.realname == msg.parameters[0]
                            || user.server == msg.parameters[0]
                            || user.hostname == msg.parameters[0]
                            || user.nickname == msg.parameters[0]
                            || user.username == msg.parameters[0])
                            && (user.is_admin)
                        {
                            users_to_display.push(user);
                        }
                    }
                    response_vector = find_users(users_to_display, response_vector);
                }
                _ => {
                    println!("wrong parameter");
                }
            }
        } else {
            for user in users_iter {
                if user.username == msg.parameters[0]
                    || user.nickname == msg.parameters[0]
                    || user.hostname == msg.parameters[0]
                    || user.server == msg.parameters[0]
                    || user.realname == msg.parameters[0]
                {
                    users_to_display.push(user);
                }
            }
            response_vector = find_users(users_to_display, response_vector);
        }
    }

    Ok(response_vector)
}

fn find_users(users_to_display: Vec<&User>, mut response_vector: Vec<Message>) -> Vec<Message> {
    let mut own_string: String = " ".to_owned();

    for user in users_to_display.iter() {
        own_string.push_str(&user.nickname);
        own_string.push(' ');
    }
    if users_to_display.is_empty() {
        println!("There is no matches");
    } else {
        println!("usuarios listados{:?}", users_to_display);
        let who_reply = rpl_who_reply(own_string);
        response_vector.push(who_reply);
    }
    response_vector
}
/// Se encarga de interpretar el mensaje de AWAY enviado a un servidor
/// y en caso de éxito setea la respuesta automática que dará cuando alguien
/// se contacte con ese usuario.
pub fn away_msg(msg: Message, users: Arc<Mutex<Vec<User>>>) -> Result<Vec<Message>, ServerError> {
    let nickname = msg.prefix.ok_or_else(|| ServerError::new("Unknown user"))?;
    let mut away_message: Option<String> = None;
    let mut response_vec = Vec::new();
    if !msg.parameters.is_empty() {
        away_message = Some(msg.parameters[0].to_string());
        println!("User {nickname} is now AFK");
        response_vec.push(rpl_away());
    } else {
        println!("User {nickname} is not AFK any more");
        response_vec.push(rpl_unaway());
    }
    let mut users_lock = users
        .lock()
        .map_err(|_e| ServerError::new("Cannot obtain user"))?;
    let user = users_lock
        .iter_mut()
        .find(|u| u.nickname == nickname)
        .ok_or_else(|| ServerError::new("Unknown user"))?;
    user.set_away_message(away_message);
    Ok(response_vec)
}
///Verifica que se cumplan las condiciones de spanning tree para aceptar la conexión de
/// un nuevo servidor.
fn attempt_sv_connection(
    msg: Message,
    server: Arc<Server>,
    user_nickname: String,
) -> Result<Vec<Message>, ServerError> {
    let mut response_vector = Vec::new();
    let input = msg.parameters[0].clone() + " " + &msg.parameters[1];

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
        response_vector.push(err_no_privileges);
        return Ok(response_vector);
    }

    attempt_server_conection(server.clone(), input, server.name.clone());

    Ok(response_vector)
}

#[cfg(test)]
mod tests_interpreter {
    use crate::interpreter::process_message;
    use crate::{channel::Channel, message::Message, server::Server, user::User};
    use std::sync::{Arc, Mutex};

    use super::away_msg;

    #[test]
    fn test_set_away_message() {
        let msg = Message::from(":leo AWAY :me fui al kiosco".to_string());
        let mut user = User::new(None);
        user.nickname = "leo".to_string();
        let users = Arc::new(Mutex::new(vec![user]));

        let result = away_msg(msg, users);

        assert!(result.is_ok());
        let vector = result.unwrap();
        assert_eq!((&vector).len(), 1);
        let message = String::from(vector[0].clone());
        assert_eq!(
            message,
            "306 :You have been marked as being away".to_string()
        );
    }

    #[test]
    fn test_unset_away_message() {
        let msg = Message::from(":leo AWAY".to_string());
        let mut user = User::new(None);
        user.nickname = "leo".to_string();
        let users = Arc::new(Mutex::new(vec![user]));

        let result = away_msg(msg, users);

        assert!(result.is_ok());
        let vector = result.unwrap();
        assert_eq!((&vector).len(), 1);
        let message = String::from(vector[0].clone());
        assert_eq!(
            message,
            "305 :You are no longer marked as being away".to_string()
        );
    }

    #[test]
    fn test_cannot_set_away_message_if_nickname_is_not_found() {
        let msg = Message::from(":leo AWAY".to_string());
        let user = User::new(None);
        let users = Arc::new(Mutex::new(vec![user]));

        let result = away_msg(msg, users);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.msg, "Unknown user".to_string());
    }

    #[test]
    fn test_change_nick_successfull() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let aux_user = user.clone();
        let mut_nickname = &mut user.nickname;
        let users = Arc::new(Mutex::new(vec![aux_user]));
        let msg = Message::from("NICK nick2".to_string());
        let mut server = Server::new();
        server.users = users;
        let arc_server = Arc::new(server);
        let _ = process_message(msg, arc_server.clone(), mut_nickname);
        let aux = arc_server.users.clone();
        let mut found_nickname = String::new();
        match aux.lock().map_err(|_e| ()) {
            Ok(lock) => {
                match lock
                    .iter()
                    .find(|x| x.nickname == "nick2".to_string())
                    .ok_or_else(|| ())
                {
                    Ok(current_user) => {
                        found_nickname = current_user.nickname.clone();
                    }
                    Err(()) => {}
                }
            }
            Err(()) => {}
        }
        assert_eq!(found_nickname, "nick2".to_string());
    }
    #[test]
    fn test_repeated_nick() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let aux_user = user.clone();
        let mut_nickname = &mut user.nickname;
        let mut user2 = User::new(None);
        user2.nickname = "nick2".to_string();
        let aux_user2 = user2.clone();
        let users = Arc::new(Mutex::new(vec![aux_user, aux_user2]));
        let msg = Message::from("NICK nick2".to_string());
        let mut server = Server::new();
        server.users = users;
        let arc_server = Arc::new(server);
        let _ = process_message(msg, arc_server.clone(), mut_nickname);
        let aux = arc_server.users.clone();
        let mut found_nickname = String::new();
        match aux.lock().map_err(|_e| ()) {
            Ok(lock) => {
                match lock
                    .iter()
                    .find(|x| x.nickname == "nick1".to_string())
                    .ok_or_else(|| ())
                {
                    Ok(_) => {
                        found_nickname = "cant change".to_string();
                    }
                    Err(()) => {}
                }
            }
            Err(()) => {}
        }
        assert_eq!(found_nickname, "cant change".to_string());
    }

    #[test]
    fn test_make_oper() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        user.password = "1234".to_string();
        user.username = "admin".to_string();
        let aux_user = user.clone();
        let mut_nickname = &mut user.nickname;
        let users = Arc::new(Mutex::new(vec![aux_user]));
        let msg = Message::from("OPER admin 1234".to_string());
        let mut server = Server::new();
        server.users = users;
        let arc_server = Arc::new(server);
        let _ = process_message(msg, arc_server.clone(), mut_nickname);
        let mut is_oper = false;
        let aux = arc_server.users.clone();
        match aux.lock().map_err(|_e| ()) {
            Ok(lock) => {
                match lock
                    .iter()
                    .find(|x| x.nickname == "nick1".to_string())
                    .ok_or_else(|| ())
                {
                    Ok(current_user) => {
                        is_oper = current_user.is_admin;
                    }
                    Err(()) => {}
                }
            }
            Err(()) => {}
        }
        assert_eq!(is_oper, true);
    }
    #[test]
    fn test_cant_make_oper_wrong_pass() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        user.password = "password1".to_string();
        user.username = "user1".to_string();
        let aux_user = user.clone();
        let mut_nickname = &mut user.nickname;
        let users = Arc::new(Mutex::new(vec![aux_user]));
        let msg = Message::from("OPER incorrect_password user1".to_string());
        let mut server = Server::new();
        server.users = users;
        let arc_server = Arc::new(server);
        let _ = process_message(msg, arc_server.clone(), mut_nickname);
        let mut is_oper = false;
        let aux = arc_server.users.clone();
        match aux.lock().map_err(|_e| ()) {
            Ok(lock) => {
                match lock
                    .iter()
                    .find(|x| x.nickname == "nick1".to_string())
                    .ok_or_else(|| ())
                {
                    Ok(current_user) => {
                        is_oper = current_user.is_admin;
                    }
                    Err(()) => {}
                }
            }
            Err(()) => {}
        }
        assert_eq!(is_oper, false);
    }

    #[test]
    fn test_kick_user_successfully() {
        let mut user = User::new(None);
        user.nickname = "nick_oper".to_string();
        user.password = "password1".to_string();
        user.username = "user1".to_string();
        user.is_admin = true;
        let kicked_user = "kicked_nick".to_string();
        let aux_user = user.clone();
        let mut_nickname = &mut user.nickname;
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin(aux_user.clone().nickname);
        channel.add_user(kicked_user.clone());
        let channels = Arc::new(Mutex::new(vec![channel.clone()]));
        let users = Arc::new(Mutex::new(vec![aux_user]));
        let msg = Message::from("KICK #canal1 kicked_nick".to_string());
        let mut server = Server::new();
        server.users = users;
        server.channels = channels;
        let arc_server = Arc::new(server);
        let arc_server_aux = arc_server.clone();

        let _ = process_message(msg, arc_server.clone(), mut_nickname);
        let lock = arc_server_aux.channels.lock().map_err(|_e| ()).unwrap();
        let canal_server = lock
            .iter()
            .find(|canal| canal.name == "#canal1".to_string())
            .unwrap();
        assert_eq!(canal_server.has_user(&kicked_user), false);
    }

    #[test]
    fn test_does_not_kick_not_oper() {
        let mut user = User::new(None);
        user.nickname = "nick_oper".to_string();
        user.password = "password1".to_string();
        user.username = "user1".to_string();
        user.is_admin = true;
        let kicked_user = "kicked_nick".to_string();
        let aux_user = user.clone();
        let mut_nickname = &mut user.nickname;
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_user(aux_user.clone().nickname);
        channel.add_user(kicked_user.clone());
        let channels = Arc::new(Mutex::new(vec![channel.clone()]));
        let users = Arc::new(Mutex::new(vec![aux_user]));
        let msg = Message::from("KICK #canal1 kicked_nick".to_string());
        let mut server = Server::new();
        server.users = users;
        server.channels = channels;
        let arc_server = Arc::new(server);
        let arc_server_aux = arc_server.clone();

        let _ = process_message(msg, arc_server.clone(), mut_nickname);
        let lock = arc_server_aux.channels.lock().map_err(|_e| ()).unwrap();
        let canal_server = lock
            .iter()
            .find(|canal| canal.name == "#canal1".to_string())
            .unwrap();
        assert_eq!(canal_server.has_user(&kicked_user), true);
    }
}
