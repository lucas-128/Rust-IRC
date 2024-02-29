use crate::channel_modes::ChannelModes;
use crate::message::{InvalidMessageError, Message};
use crate::replies::{
    err_bad_channel_key, err_banned_from_chan, err_chan_opriv_is_needed, err_channel_is_full,
    err_invite_only_chan, err_key_set, err_need_more_params, err_no_oper_host, err_no_such_channel,
    err_not_on_channel, err_unknown_mode, err_user_on_channel, error_no_such_nick, rpl_banlist,
    rpl_end_of_ban_list, rpl_end_of_names, rpl_inviting, rpl_list, rpl_list_end, rpl_list_start,
    rpl_name_rply, rpl_no_topic, rpl_topic, rpl_you_are_oper,
};
use crate::server::Server;
use crate::server_errors::ServerError;
use crate::server_messages_interpreter::forward_invite_msg;
use crate::user::User;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
///Medio de comunicación para un determinado grupo de clientes. Cada canal tiene un nombre,
/// usuarios que forman parte, tópico, usuarios operadores del canal y un conjunto de modos
/// que limitan determinadas acciones.
pub struct Channel {
    pub name: String,
    pub users: Vec<String>,
    pub topic: Option<String>,
    pub admins: Vec<String>,
    pub mode: ChannelModes,
    // el limite de usuarios en un canal solo se puede setear una vez y no puede ser menor que la cantidad de usuarios actual del canal
    pub limit: Option<usize>,
    pub ban_masks: Vec<String>,
    pub can_speak_users: Vec<String>,
    pub password: Option<String>,
}
///Instancia un nuevo canal.
impl Channel {
    pub fn new(name: &String) -> Self {
        println!("Se creo el canal {}", &name);

        Self {
            name: name.to_string(),
            users: Vec::new(),
            topic: None,
            admins: Vec::new(),
            mode: ChannelModes::new(),
            limit: None,
            ban_masks: Vec::new(),
            can_speak_users: Vec::new(),
            password: None,
        }
    }

    pub fn get_users_list(&self) -> String {
        let mut list = String::new();
        for user in &self.users {
            list += &(",".to_string() + user);
        }
        list
    }
    pub fn set_users(&mut self, mut list: String) {
        list.remove(0);
        let v: Vec<String> = list.split(',').map(|s| s.to_string()).collect();
        self.users = v;
    }
    pub fn get_admins_list(&self) -> String {
        let mut list = String::new();
        for user in &self.admins {
            list += &(",".to_string() + user);
        }
        list
    }
    pub fn set_admins(&mut self, mut list: String) {
        list.remove(0);
        let v: Vec<String> = list.split(',').map(|s| s.to_string()).collect();
        self.admins = v;
    }
    pub fn get_ban_list(&self) -> String {
        let mut list = String::new();
        if self.ban_masks.is_empty() {
            ".".to_string()
        } else {
            for mask in &self.ban_masks {
                list += &(",".to_string() + mask);
            }
            list
        }
    }
    pub fn set_ban_list(&mut self, mut list: String) {
        if !list.starts_with('.') {
            list.remove(0);
            let v: Vec<String> = list.split(',').map(|s| s.to_string()).collect();
            self.ban_masks = v;
        }
    }
    pub fn get_can_speak_users_list(&self) -> String {
        let mut list = String::new();
        if self.can_speak_users.is_empty() {
            ".".to_string()
        } else {
            for user in &self.can_speak_users {
                list += &(",".to_string() + user);
            }
            list
        }
    }
    pub fn set_speak_users(&mut self, mut list: String) {
        if !list.starts_with('.') {
            list.remove(0);
            let v: Vec<String> = list.split(',').map(|s| s.to_string()).collect();
            self.can_speak_users = v;
        }
    }
    pub fn get_topic_option(&self) -> String {
        if let Some(topic) = self.topic.clone() {
            topic
        } else {
            ".".to_string()
        }
    }

    pub fn set_topic(&mut self, topic: String) {
        if !topic.starts_with('.') {
            self.topic = Some(topic);
        } else {
            self.topic = None
        }
    }

    pub fn get_limit_option(&self) -> String {
        if let Some(limit) = self.limit {
            limit.to_string()
        } else {
            ".".to_string()
        }
    }

    pub fn set_limit(&mut self, limit: String) {
        if let Ok(limit_number) = limit.parse::<usize>() {
            self.limit = Option::from(limit_number);
        } else {
            self.limit = None;
        }
    }
    pub fn get_password_option(&self) -> String {
        if let Some(password) = self.password.clone() {
            password
        } else {
            ".".to_string()
        }
    }
    pub fn set_password(&mut self, password: String) {
        if !password.starts_with('.') {
            self.password = Some(password);
        } else {
            self.password = None
        }
    }
    ///Verifica si un determinado usuario es operador del canal.
    pub fn is_admin(&self, user_nickname: &String) -> bool {
        self.admins.iter().any(|user| user_nickname == user)
    }
    ///Concede a un determinado usuario privilegios de operador en el canal.
    pub fn add_admin(&mut self, user_nickname: String) {
        if !self.is_admin(&user_nickname) {
            println!("Se hizo admin de canal al usuario {}", &user_nickname);
            self.admins.push(user_nickname);
            println!("Admins del canal {:?}", self.admins);
        }
    }
    ///Quita a un determinado usuario privilegios de operador en el canal.
    pub fn remove_admin(&mut self, user_nickname: String) {
        self.admins
            .iter()
            .position(|user| &user_nickname == user)
            .map(|position| self.admins.remove(position));
    }
    ///Quita a un determinado usuario la posibilidad de hablar en el canal.
    pub fn remove_speaker(&mut self, user_nickname: String) {
        if self.can_speak(&user_nickname) {
            self.can_speak_users
                .iter()
                .position(|user| &user_nickname == user)
                .map(|position| self.can_speak_users.remove(position));
        }
    }

    ///Verifica si un determinado usuario está en el canal.
    pub fn has_user(&self, user_nickname: &String) -> bool {
        self.users.iter().any(|user| user_nickname == user)
    }

    ///Verifica si un determinado usuario está habilitado para hablar en el canal.
    pub fn can_speak(&self, user_nickname: &String) -> bool {
        self.can_speak_users
            .iter()
            .any(|user| user_nickname == user)
    }
    ///Agrega un usuario al canal.
    pub fn add_user(&mut self, user_nickname: String) {
        if !self.has_user(&user_nickname) {
            println!("Se agrego al usuario {}", &user_nickname);
            self.users.push(user_nickname);
            println!("Usuarios del canal {:?}", self.users);
        }
    }
    ///Evalúa si es posible agregar a un usuario al canal.
    pub fn can_add_user(&self) -> bool {
        !self.has_limit() || (self.has_limit() && self.has_free_space())
    }
    ///Evalúa si se alcanzó el límite de usuarios para el canal.
    pub fn has_free_space(&self) -> bool {
        if let Some(lim) = self.limit {
            lim > self.users.len()
        } else {
            false
        }
    }

    ///Habilita a un usuario a esribir en el canal.
    pub fn add_speaker(&mut self, user_nickname: String) {
        if !self.can_speak(&user_nickname) {
            self.can_speak_users.push(user_nickname)
        }
    }
    ///Quita a un usuario del canal.
    pub fn remove_user(&mut self, user_nickname: &String) {
        self.users
            .iter()
            .position(|user| user_nickname == user)
            .map(|position| self.users.remove(position));
        self.admins
            .iter()
            .position(|user| user_nickname == user)
            .map(|position| self.admins.remove(position));
        println!("Usuarios del canal {:?}", self.users);
    }
    ///Elimina máscara de ban del listado.
    pub fn remove_ban(&mut self, ban_mask: String) {
        self.ban_masks
            .iter()
            .position(|mask| &ban_mask == mask)
            .map(|position| self.ban_masks.remove(position));
    }
    ///Evalúa si el canal es visible.
    pub fn is_visible(&self) -> bool {
        !self.is_private() && !self.is_secret()
    }
    ///Evalúa si el canal es privado.
    pub fn is_private(&self) -> bool {
        self.mode.p
    }
    ///Evalúa si el canal es secreto.
    pub fn is_secret(&self) -> bool {
        self.mode.s
    }
    ///Evalúa si el canal es invite only.
    pub fn is_invite_only(&self) -> bool {
        self.mode.i
    }
    ///Evalúa si solo los operadores pueden cambiar el topic.
    pub fn is_topic_operator_only(&self) -> bool {
        self.mode.t
    }
    ///Evalúa si el canal no permite mensajes de afuera.
    pub fn is_no_msg_outside(&self) -> bool {
        self.mode.n
    }
    ///Evalúa si el canal es moderado.
    pub fn is_moderated(&self) -> bool {
        self.mode.m
    }
    ///Evalúa si el canal no tiene restricciones sobre quién puede enviar mensajes.

    pub fn is_not_msg_restricted(&self) -> bool {
        !self.is_no_msg_outside() && !self.is_moderated()
    }
    ///Evalúa si el canal tiene límite de usuarios.
    pub fn has_limit(&self) -> bool {
        self.mode.l
    }
    ///Evalúa si el canal tiene contraseña.
    pub fn has_key(&self) -> bool {
        self.mode.k
    }
    ///Evalúa si un determinado usuario puede enviar mensajes.
    pub fn can_send_msg(&self, user_nickname: &String) -> bool {
        self.is_admin(user_nickname) || self.can_speak(user_nickname)
    }

    ///Evalúa si un determinado usuario esta banneado.
    pub fn is_banned(&self, username: &String, hostname: &String) -> bool {
        if self.ban_masks.is_empty() {
            return false;
        }
        for mask in &self.ban_masks {
            if check_banned(mask, username, hostname) {
                return true;
            }
        }
        false
    }

    ///Obtiene un listado de los usuarios del canal.
    pub fn list_users(&self) -> Message {
        let mut line = String::from("Users from channel ") + &self.name;
        for user_nickname in &self.users {
            line = [line, user_nickname.clone()].join("\n");
        }
        crate::message::Message::from(line)
    }
    ///Obtiene el topic del canal.
    pub fn get_topic(&self) -> String {
        let topic = self.topic.clone();
        match topic {
            Some(topic_text) => topic_text,
            None => "No topic is set".to_string(),
        }
    }
    pub fn has_topic(&self) -> bool {
        match &self.topic {
            Some(_topic_text) => true,
            None => false,
        }
    }
    pub fn change_topic(&mut self, topic: String) {
        self.topic = Option::from(topic)
    }
    pub fn show_channel_topic(&self) -> Message {
        let line = format!("Topic from channel {}: {}", &self.name, &self.get_topic());
        crate::message::Message::from(line)
    }
    pub fn correct_key(&self, parameters: Vec<String>) -> bool {
        parameters.len() == 2 && self.password == Some(parameters[1].clone())
    }

    pub fn activate_modes(
        &mut self,
        msg: Message,
        mut response_vector: Vec<Message>,
    ) -> Vec<Message> {
        for mode in msg.parameters[1].chars().skip(1) {
            response_vector = self.activate_mode(msg.clone(), mode, response_vector);
        }
        response_vector
    }
    pub fn deactivate_modes(
        &mut self,
        msg: Message,
        mut response_vector: Vec<Message>,
    ) -> Vec<Message> {
        for mode in msg.parameters[1].chars().skip(1) {
            response_vector = self.deactivate_mode(msg.clone(), mode, response_vector);
        }
        response_vector
    }

    pub fn activate_mode(
        &mut self,
        msg: Message,
        mode: char,
        mut response_vector: Vec<Message>,
    ) -> Vec<Message> {
        match mode {
            'o' => self.operator_mode(msg, response_vector),
            'p' => {
                self.mode.activate_p();
                response_vector
            }
            's' => {
                self.mode.activate_s();
                response_vector
            }
            't' => {
                self.mode.activate_t();
                response_vector
            }
            'i' => {
                self.mode.activate_i();
                response_vector
            }
            'n' => {
                self.mode.activate_n();
                response_vector
            }
            'm' => {
                self.mode.activate_m();
                response_vector
            }
            'v' => self.speak_mode(msg, response_vector),
            'l' => self.limit_mode(msg, response_vector),
            'b' => self.ban_mode(msg, response_vector),
            'k' => self.key_mode(msg, response_vector),
            _ => {
                let unknown_mode = err_unknown_mode(mode);
                response_vector.push(unknown_mode);
                response_vector
            }
        }
    }
    ///Si se cumplen las condiciones, otorga permisos de operador de canal al usuario
    /// pasado por parámetro.
    pub fn operator_mode(
        &mut self,
        msg: Message,
        mut response_vector: Vec<Message>,
    ) -> Vec<Message> {
        if msg.parameters.len() < 3 {
            let err_need_more_params = err_need_more_params(msg.command);
            response_vector.push(err_need_more_params);
        } else if !self.has_user(&msg.parameters[2]) {
            let no_such_nick_msg = error_no_such_nick(msg.parameters[1].clone());
            response_vector.push(no_such_nick_msg);
        } else {
            self.add_admin(msg.parameters[2].clone())
        }
        response_vector
    }
    ///Si se cumplen las condiciones, quita permisos de operador de canal al usuario
    /// pasado por parámetro.
    pub fn deoperator_mode(
        &mut self,
        msg: Message,
        mut response_vector: Vec<Message>,
    ) -> Vec<Message> {
        if msg.parameters.len() < 3 {
            let err_need_more_params = err_need_more_params(msg.command);
            response_vector.push(err_need_more_params);
        } else if !self.has_user(&msg.parameters[2]) {
            let no_such_nick_msg = error_no_such_nick(msg.parameters[1].clone());
            response_vector.push(no_such_nick_msg);
        } else {
            self.remove_admin(msg.parameters[2].clone())
        }
        response_vector
    }
    ///Si se cumplen las condiciones, agrega  al usuario pasado por parámetro la posibilidad
    /// de enviar mensajes en ese canal.
    pub fn speak_mode(&mut self, msg: Message, mut response_vector: Vec<Message>) -> Vec<Message> {
        if msg.parameters.len() < 3 {
            let err_need_more_params = err_need_more_params(msg.command);
            response_vector.push(err_need_more_params);
        } else if !self.has_user(&msg.parameters[2]) {
            let no_such_nick_msg = error_no_such_nick(msg.parameters[1].clone());
            response_vector.push(no_such_nick_msg);
        } else {
            self.add_speaker(msg.parameters[2].clone())
        }
        response_vector
    }
    ///Si se cumplen las condiciones, le quita al usuario pasado por parámetro la posibilidad
    /// de enviar mensajes en ese canal.
    pub fn despeak_mode(
        &mut self,
        msg: Message,
        mut response_vector: Vec<Message>,
    ) -> Vec<Message> {
        if msg.parameters.len() < 3 {
            let err_need_more_params = err_need_more_params(msg.command);
            response_vector.push(err_need_more_params);
        } else if !self.has_user(&msg.parameters[2]) {
            let no_such_nick_msg = error_no_such_nick(msg.parameters[1].clone());
            response_vector.push(no_such_nick_msg);
        } else {
            self.remove_speaker(msg.parameters[2].clone())
        }
        response_vector
    }
    ///Si se cumplen las condiciones, establece un límite de usuarios que pueden acceder
    /// al canal.
    pub fn limit_mode(&mut self, msg: Message, mut response_vector: Vec<Message>) -> Vec<Message> {
        if msg.parameters.len() < 3 {
            let err_need_more_params = err_need_more_params(msg.command);
            response_vector.push(err_need_more_params);
        }
        // el limite de usuarios en un canal solo se puede setear una vez y no puede ser menor que la cantidad de usuarios actual del canal
        else if let Ok(limit) = msg.parameters[2].parse::<usize>() {
            if !self.has_limit() && limit >= self.users.len() {
                self.limit = Option::from(limit);
                self.mode.activate_l();
            }
        }
        response_vector
    }
    ///Si se cumplen las condiciones, agrega una mascara de ban a la lista de
    /// máscaras del canal.
    pub fn ban_mode(&mut self, msg: Message, mut response_vector: Vec<Message>) -> Vec<Message> {
        if msg.parameters.len() < 2 {
            let err_need_more_params = err_need_more_params(msg.command);
            response_vector.push(err_need_more_params);
        } else if msg.parameters.len() == 2 {
            for ban_mask in &self.ban_masks {
                let ban_list = rpl_banlist(self.name.clone(), ban_mask.clone());
                response_vector.push(ban_list)
            }
            let end_ban_list = rpl_end_of_ban_list(self.name.clone());
            response_vector.push(end_ban_list)
        } else if msg.parameters.len() == 3 && self.ban_masks.len() < 3 {
            self.ban_masks.push(msg.parameters[2].clone());
        }
        response_vector
    }
    ///Si se cumplen las condiciones, quita una mascara de ban de la lista de
    /// máscaras del canal.
    pub fn unban_mode(&mut self, msg: Message, mut response_vector: Vec<Message>) -> Vec<Message> {
        if msg.parameters.len() < 3 {
            let err_need_more_params = err_need_more_params(msg.command);
            response_vector.push(err_need_more_params);
        } else {
            self.remove_ban(msg.parameters[2].clone());
        }
        response_vector
    }
    ///Establece la contraseña de un canal.
    pub fn key_mode(&mut self, msg: Message, mut response_vector: Vec<Message>) -> Vec<Message> {
        if msg.parameters.len() < 3 {
            let err_need_more_params = err_need_more_params(msg.command);
            response_vector.push(err_need_more_params);
        } else if self.has_key() {
            let err_key_set = err_key_set(self.name.clone());
            response_vector.push(err_key_set);
        } else {
            self.password = Some(msg.parameters[2].clone());
            self.mode.activate_k();
        }
        response_vector
    }
    pub fn deactivate_mode(
        &mut self,
        msg: Message,
        mode: char,
        mut response_vector: Vec<Message>,
    ) -> Vec<Message> {
        match mode {
            'o' => self.deoperator_mode(msg, response_vector),
            'p' => {
                self.mode.deactivate_p();
                response_vector
            }
            's' => {
                self.mode.deactivate_s();
                response_vector
            }
            't' => {
                self.mode.deactivate_t();
                response_vector
            }
            'i' => {
                self.mode.deactivate_i();
                response_vector
            }
            'n' => {
                self.mode.deactivate_n();
                response_vector
            }
            'm' => {
                self.mode.deactivate_m();
                response_vector
            }
            'v' => self.despeak_mode(msg, response_vector),
            'l' => {
                self.limit = None;
                self.mode.deactivate_l();
                response_vector
            }
            'b' => self.unban_mode(msg, response_vector),
            'k' => {
                self.password = None;
                self.mode.deactivate_k();
                response_vector
            }
            _ => {
                let unknown_mode = err_unknown_mode(mode);
                response_vector.push(unknown_mode);
                response_vector
            }
        }
    }
}
///Crea un canal si el nombre seteado es válido.
pub fn create_valid_channel(name: String) -> Result<Channel, InvalidMessageError> {
    let vec_bytes = name.as_bytes();
    // ASCII 35 = '#' , ASCII 38 = '&' , ASCII 32 = ' ', ASCII 44 = ',', ASCII 7 = Ctrl G
    if vec_bytes.len() > 200
        || (!vec_bytes.starts_with(&[35]) && !vec_bytes.starts_with(&[38]))
        || vec_bytes.contains(&32)
        || vec_bytes.contains(&44)
        || vec_bytes.contains(&7)
    {
        Err(InvalidMessageError {
            error_message: "No se puede crear un canal con ese nombre".to_owned(),
        })
    } else {
        Ok(Channel::new(&name))
    }
}
///Evalúa si un usuario esta banneado en un canal.
pub fn check_banned(mask: &str, username: &String, hostname: &String) -> bool {
    let mask_parts: Vec<&str> = mask.split('@').collect();
    let mask_username = &mask_parts[0][1..];
    let mask_hostname = &mask_parts[1];
    match_ban_expression(mask_username, username) && match_ban_expression(mask_hostname, hostname)
}
pub fn match_ban_expression(mask_part: &str, query_part: &String) -> bool {
    if mask_part == "*" || mask_part.is_empty() {
        return true;
    }
    match mask_part.strip_prefix('*') {
        Some(stripped) => query_part.ends_with(stripped),
        None => mask_part == query_part,
    }
}

/// Se encarga de interpretar el mensaje de JOIN enviado a un servidor
/// y en caso de éxito agregar al usuario al canal o crear un nuevo canal y luego agregar
/// al usuario.

pub fn join_msg(
    msg: Message,
    mut user_nickname: String,
    users: Arc<Mutex<Vec<User>>>,
    channels: Arc<Mutex<Vec<Channel>>>,
    connected_servers: Arc<Mutex<Vec<Server>>>,
    server_name: String,
) -> Result<Vec<Message>, ServerError> {
    let mut response_vector = Vec::new();
    let mut lock_user = users.lock().unwrap();
    if msg.parameters[0].starts_with('&') && msg.parameters.len() == 2 {
        user_nickname = msg.parameters[1].clone();
    }
    if let Some(user) = lock_user
        .iter_mut()
        .find(|user| user.nickname == user_nickname)
    {
        if msg.parameters.is_empty() {
            let need_more_params_message = err_need_more_params(msg.command);
            response_vector.push(need_more_params_message);
            return Ok(response_vector);
        }
        //si existe el canal, agrego al usuario
        let mut lock = channels.lock().unwrap();

        for channel_name in msg.parameters[0].split(',') {
            if let Some(channel) = lock.iter_mut().find(|channel| channel.name == channel_name) {
                if channel.name.starts_with('&') {
                    let mut invite_msg =
                        Message::from(format!("JOIN {} {}", channel_name, user_nickname));
                    if msg.prefix.is_none() {
                        invite_msg = Message::from(format!(
                            ":{} JOIN {} {}",
                            server_name.clone(),
                            channel_name,
                            user_nickname.clone()
                        ));
                    }

                    forward_invite_msg(invite_msg, connected_servers.clone());
                }
                let replies = add_user_to_channel(channel, user, msg.parameters.clone());
                response_vector.extend(replies);
            }
            //si no existe el canal, creo el canal, agrego al usuario y sumo el canal al server
            else {
                let channel = create_valid_channel(channel_name.to_string().clone());
                match channel {
                    Err(_e) => {
                        let no_such_channel_message =
                            err_no_such_channel(channel_name.to_string().clone());
                        response_vector.push(no_such_channel_message);
                    }
                    Ok(mut channel) => {
                        channel.add_user(user_nickname.clone());
                        user.add_channel(&channel_name.to_string());
                        channel.add_admin(user_nickname.clone());
                        let topic_message =
                            rpl_topic(channel.name.clone(), channel.get_topic().clone());
                        let namerply_message =
                            rpl_name_rply(channel.name.clone(), channel.users.clone());
                        let end_of_names_message = rpl_end_of_names(channel.name.clone());
                        notify_new_channel(
                            users.clone(),
                            channel.name.clone(),
                            channel.get_topic(),
                        );
                        lock.push(channel);
                        response_vector.push(topic_message);
                        response_vector.push(namerply_message);
                        response_vector.push(end_of_names_message);
                    }
                }
            }
        }
    }
    Ok(response_vector)
}
///Notifica al resto de los servidores que existe un nuevo canal
fn notify_new_channel(users: Arc<Mutex<Vec<User>>>, channel_name: String, channel_topic: String) {
    thread::spawn(move || {
        let users_lock = users.lock().unwrap();
        for u in users_lock.iter() {
            let _ = u.send_private_message(rpl_list(
                channel_name.to_string(),
                channel_topic.to_string(),
            ));
            let _ = u.send_private_message(rpl_list_end());
        }
    });
}

/// Si el usuario tiene permitido ingresar al canal, lo agrega.
/// En caso contrario le notifica el motivo por el cuál no puede ser agregado al canal.
pub fn add_user_to_channel(
    channel: &mut Channel,
    user: &mut User,
    parameters: Vec<String>,
) -> Vec<Message> {
    let mut responses = Vec::new();
    if channel.is_invite_only() {
        let err_invite_only = err_invite_only_chan(channel.name.clone());
        responses.push(err_invite_only);
    } else if channel.is_banned(&user.username, &user.hostname) {
        let err_banned = err_banned_from_chan(channel.name.clone());
        responses.push(err_banned);
    } else if channel.has_limit() && !channel.has_free_space() {
        let err_channel_full = err_channel_is_full(channel.name.clone());
        responses.push(err_channel_full);
    } else if channel.has_key() && !channel.correct_key(parameters) {
        let err_bad_chan_key = err_bad_channel_key(channel.name.clone());
        responses.push(err_bad_chan_key);
    } else {
        channel.add_user(user.nickname.clone());
        user.add_channel(&channel.name.clone());
        let topic_message = rpl_topic(channel.name.clone(), channel.get_topic());
        let namerply_message = rpl_name_rply(channel.name.clone(), channel.users.clone());
        let end_of_names_message = rpl_end_of_names(channel.name.clone());
        responses.push(topic_message);
        responses.push(namerply_message);
        responses.push(end_of_names_message);
        list_channel(channel, user, &mut responses)
    }

    responses
}
/// Se encarga de interpretar el mensaje de PART enviado a un servidor
/// y en caso de éxito quitar al usuario del canal.
pub fn part_msg(
    msg: Message,
    user_nickname: String,
    users: Arc<Mutex<Vec<User>>>,
    channels: Arc<Mutex<Vec<Channel>>>,
) -> Result<Vec<Message>, ServerError> {
    let mut response_vector = Vec::new();
    let mut lock_user = users.lock().unwrap();
    if let Some(user) = lock_user
        .iter_mut()
        .find(|user| user.nickname == user_nickname)
    {
        if msg.parameters.is_empty() {
            let need_more_params_message = err_need_more_params(msg.command);
            response_vector.push(need_more_params_message);
            return Ok(response_vector);
        }
        //Busco si existe el canal
        let mut lock = channels.lock().unwrap();
        for channel_name in msg.parameters[0].split(',') {
            if let Some(channel) = lock.iter_mut().find(|channel| channel.name == channel_name) {
                //si el canal tiene al usuario
                if channel.has_user(&user_nickname) {
                    channel.remove_user(&user_nickname);
                    user.leave_channel(&channel_name.to_string())
                } else {
                    let not_on_channel = err_not_on_channel(channel_name.to_string().clone());
                    response_vector.push(not_on_channel);
                }
            }
            //si no existe el canal
            else {
                let no_such_channel_message = err_no_such_channel(channel_name.to_string().clone());
                response_vector.push(no_such_channel_message);
            }
        }
    }

    Ok(response_vector)
}

/// Le permite a un usuario convertirse en operador del canal.
pub fn oper_ch_msg(
    msg: Message,
    users: Arc<Mutex<Vec<User>>>,
    channels: Arc<Mutex<Vec<Channel>>>,
    nickname: String,
) -> Result<Vec<Message>, ServerError> {
    let mut response_vector = Vec::new();

    if msg.parameters.len() != 3 {
        let more_params = err_need_more_params(msg.command);
        response_vector.push(more_params);
        println!("Incorrect number of parameters in message");
        return Ok(response_vector);
    }
    let lock_users = users.lock().unwrap();
    let mut lock_channel = channels.lock().unwrap();
    let _current_user = lock_users
        .iter()
        .find(|user| user.nickname == nickname)
        .unwrap();
    match lock_users
        .iter()
        .find(|user| user.username == msg.parameters[1] && user.password == msg.parameters[2])
        .ok_or(())
    {
        Ok(admin) => {
            match lock_channel
                .iter()
                .find(|channel| channel.has_user(&admin.nickname))
                .ok_or(())
            {
                Ok(channel) => {
                    let mut aux_channel = channel.clone();
                    let index = lock_channel
                        .iter()
                        .position(|x| x.name == channel.name)
                        .unwrap();
                    lock_channel.remove(index);
                    aux_channel.add_admin(admin.clone().nickname);
                    lock_channel.push(aux_channel);
                    let oper_rply = rpl_you_are_oper();
                    response_vector.push(oper_rply);
                }
                Err(()) => {
                    println!("Join channel before become admin");
                    let err_oper = err_no_oper_host();
                    response_vector.push(err_oper);
                }
            };
        }

        Err(()) => {
            println!("User or password incorrect");
            let err_oper = err_no_oper_host();
            response_vector.push(err_oper);
        }
    };

    Ok(response_vector)
}
/// Se encarga de interpretar el mensaje de INVITE enviado a un servidor
/// y en caso de éxito agregar al usuario invitado al canal. Si no, le notifica el motivo
/// por el cual no pudo ser agregado.
pub fn invite_msg(
    msg: Message,
    users: Arc<Mutex<Vec<User>>>,
    nick: String,
    channels: Arc<Mutex<Vec<Channel>>>,
) -> Result<Vec<Message>, ServerError> {
    let mut response_vector = Vec::new();
    if msg.parameters.len() != 2 {
        let need_more_params_message = err_need_more_params(msg.command);
        response_vector.push(need_more_params_message);
        return Ok(response_vector);
    }
    let mut lock_user = users.lock().unwrap();
    //si existe el usuario que se quiere invitar
    if let Some(invited_user) = lock_user
        .iter_mut()
        .find(|user| user.nickname == msg.parameters[0])
    {
        let mut lock_channel = channels.lock().unwrap();
        //si existe el canal
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == msg.parameters[1])
        {
            // si el usuario que invita no esta en el canal
            if !channel.has_user(&nick) {
                let err_not_on_channel = err_not_on_channel(channel.name.clone());
                response_vector.push(err_not_on_channel)
            }
            // si el usuario a invitar ya esta en el canal
            else if channel.has_user(&invited_user.nickname) {
                let err_user_on_channel =
                    err_user_on_channel(invited_user.nickname.clone(), channel.name.clone());
                response_vector.push(err_user_on_channel)
            }
            // si el canal es invite only y el que envia el mensaje no es operador de ese canal
            else if channel.is_invite_only() && !channel.is_admin(&nick) {
                let chan_opriv_msg = err_chan_opriv_is_needed(channel.name.clone());
                response_vector.push(chan_opriv_msg)
            }
            // si el canal tiene limite de espacio y no hay mas lugar
            else if channel.has_limit() && !channel.has_free_space() {
                let err_channel_full = err_channel_is_full(channel.name.clone());
                response_vector.push(err_channel_full);
            }
            // si el usuario a invitar esta banneado
            else if channel.is_banned(&invited_user.username, &invited_user.hostname) {
                let err_banned = err_banned_from_chan(channel.name.clone());
                response_vector.push(err_banned);
            } else {
                channel.add_user(invited_user.nickname.clone());
                invited_user.add_channel(&msg.parameters[1]);
                let inviting_message =
                    rpl_inviting(invited_user.nickname.clone(), channel.name.clone());
                response_vector.push(inviting_message)
            }
        } else {
            let no_such_nick_msg = error_no_such_nick(msg.parameters[1].clone());
            response_vector.push(no_such_nick_msg)
        }
    } else {
        let no_such_nick_msg = error_no_such_nick(msg.parameters[0].clone());
        response_vector.push(no_such_nick_msg)
    }

    Ok(response_vector)
}
/// Se encarga de interpretar el mensaje de NAMES enviado a un servidor
/// y en caso de éxito informar los nombres de los usuarios del canal.
pub fn names_msg(
    msg: Message,
    users: Arc<Mutex<Vec<User>>>,
    nick: String,
    channels: Arc<Mutex<Vec<Channel>>>,
) -> Result<Vec<Message>, ServerError> {
    let mut response_vector = Vec::new();
    let lock_user = users.lock().unwrap();
    let user = lock_user.iter().find(|user| user.nickname == nick).unwrap();

    let lock_channel = channels.lock().unwrap();
    if msg.parameters.is_empty() {
        let mut listed_users = HashSet::new();
        for channel in lock_channel.iter() {
            if let Some(channel_users) = names_channel(channel, user) {
                if channel.has_user(&user.nickname) || channel.is_visible() {
                    let namerply_message =
                        rpl_name_rply(channel.name.clone(), channel.users.clone());
                    let end_of_names_message = rpl_end_of_names(channel.name.clone().to_string());
                    response_vector.push(namerply_message);
                    response_vector.push(end_of_names_message);
                }
                listed_users.extend(channel_users);
            }
        }
        let mut not_listed_users = Vec::new();
        for element in lock_user.iter() {
            if element.channels.is_empty() || !listed_users.contains(&element.nickname) {
                not_listed_users.push(element.nickname.clone());
            }
        }
        if !not_listed_users.is_empty() {
            let namerply_message = rpl_name_rply("*".to_string(), not_listed_users);
            let end_of_names_message = rpl_end_of_names("*".to_string());
            response_vector.push(namerply_message);
            response_vector.push(end_of_names_message);
        }
    } else {
        for channel_name in msg.parameters[0].split(',') {
            if let Some(channel) = lock_channel
                .iter()
                .find(|channel| channel.name == channel_name)
            {
                if channel.has_user(&user.nickname) || channel.is_visible() {
                    let namerply_message =
                        rpl_name_rply(channel.name.clone(), channel.users.clone());
                    let end_of_names_message = rpl_end_of_names(channel.name.clone().to_string());
                    response_vector.push(namerply_message);
                    response_vector.push(end_of_names_message);
                }
            }
        }
    }
    Ok(response_vector)
}
pub fn names_channel(channel: &Channel, user: &User) -> Option<HashSet<String>> {
    if channel.has_user(&user.nickname) || channel.is_visible() {
        Some(HashSet::from_iter(channel.users.clone()))
    } else {
        None
    }
}

/// Se encarga de interpretar el mensaje de LIST enviado a un servidor
/// y en caso de éxito listar los canales y sus tópicos, si los canales no son secretos.
pub fn list_msg(
    msg: Message,
    nick: String,
    users: Arc<Mutex<Vec<User>>>,
    channels: Arc<Mutex<Vec<Channel>>>,
) -> Result<Vec<Message>, ServerError> {
    let mut response_vector = Vec::new();
    let list_start_message = rpl_list_start();
    response_vector.push(list_start_message);

    let lock_user = users.lock().unwrap();
    let user = lock_user.iter().find(|user| user.nickname == nick).unwrap();

    let lock_channel = channels.lock().unwrap();
    if msg.parameters.is_empty() {
        for channel in lock_channel.iter() {
            list_channel(channel, user, &mut response_vector);
        }
    } else {
        for channel_name in msg.parameters[0].split(',') {
            if let Some(channel) = lock_channel
                .iter()
                .find(|channel| channel.name == channel_name)
            {
                list_channel(channel, user, &mut response_vector)
            }
        }
    }
    let list_end_message = rpl_list_end();
    response_vector.push(list_end_message);
    Ok(response_vector)
}

pub fn list_channel(channel: &Channel, user: &User, response_vector: &mut Vec<Message>) {
    if channel.is_visible() || (channel.is_secret() && channel.has_user(&user.nickname)) {
        let rpl_list = rpl_list(channel.name.clone(), channel.get_topic());
        response_vector.push(rpl_list);
    } else if channel.is_private() {
        if channel.has_user(&user.nickname) {
            let rpl_list = rpl_list(channel.name.clone(), channel.get_topic());
            response_vector.push(rpl_list);
        } else {
            let rpl_list = rpl_list(channel.name.clone(), "".to_string());
            response_vector.push(rpl_list);
        }
    }
}

/// Se encarga de interpretar el mensaje de TOPIC enviado a un servidor
/// y en caso de éxito informar el tópico del canal o cambiarlo.
pub fn topic_msg(
    msg: Message,
    nick: String,
    channels: Arc<Mutex<Vec<Channel>>>,
) -> Result<Vec<Message>, ServerError> {
    let mut response_vector = Vec::new();
    if msg.parameters.is_empty() {
        let need_more_params_message = err_need_more_params(msg.command);
        response_vector.push(need_more_params_message);
        return Ok(response_vector);
    }
    let mut lock_channel = channels.lock().unwrap();
    //si existe el canal
    if let Some(channel) = lock_channel
        .iter_mut()
        .find(|channel| channel.name == msg.parameters[0])
    {
        //TOPIC channel devuelve el topico
        if msg.parameters.len() == 1 {
            if !channel.has_user(&nick) {
                let not_on_channel = err_not_on_channel(channel.name.clone());
                response_vector.push(not_on_channel);
                return Ok(response_vector);
            } else if channel.has_topic() {
                let rpl_topic = rpl_topic(channel.name.clone(), channel.get_topic());
                response_vector.push(rpl_topic);
                return Ok(response_vector);
            } else {
                let rpl_no_topic = rpl_no_topic(channel.name.clone());
                response_vector.push(rpl_no_topic);
                return Ok(response_vector);
            }
        }

        //TOPIC channel topic , cambia el topico
        if msg.parameters.len() == 2 {
            if (channel.is_topic_operator_only() && channel.is_admin(&nick))
                || (!channel.is_topic_operator_only() && channel.has_user(&nick))
            {
                channel.change_topic(msg.parameters[1].clone());
                let rpl_topic = rpl_topic(channel.name.clone(), channel.get_topic());
                response_vector.push(rpl_topic);
                return Ok(response_vector);
            } else {
                let chan_opriv_msg = err_chan_opriv_is_needed(channel.name.clone());
                response_vector.push(chan_opriv_msg)
            }
        }
    }

    Ok(response_vector)
}

/// Se encarga de interpretar el mensaje de MODE enviado a un servidor
/// y de agregar o quitar modos en la configuración del canal.
pub fn mode_msg(
    msg: Message,
    nick: String,
    channels: Arc<Mutex<Vec<Channel>>>,
) -> Result<Vec<Message>, ServerError> {
    let mut response_vector = Vec::new();
    if msg.parameters.len() < 2 {
        let need_more_params_message = err_need_more_params(msg.command);
        response_vector.push(need_more_params_message);
        return Ok(response_vector);
    }
    let mut lock_channel = channels.lock().unwrap();
    if let Some(channel) = lock_channel
        .iter_mut()
        .find(|channel| channel.name == msg.parameters[0])
    {
        if !channel.is_admin(&nick) {
            let chan_opriv_msg = err_chan_opriv_is_needed(channel.name.clone());
            response_vector.push(chan_opriv_msg);
            return Ok(response_vector);
        }
        // evaluar si activa o desactiva modos
        let activate_flag = msg.parameters[1].chars().next().unwrap();
        let response_vector = match activate_flag {
            '+' => channel.activate_modes(msg, response_vector),
            '-' => channel.deactivate_modes(msg, response_vector),
            _ => {
                let unkwnown_mode = err_unknown_mode(activate_flag);
                response_vector.push(unkwnown_mode);
                response_vector
            }
        };
        Ok(response_vector)
    } else {
        let no_such_channel_message = err_no_such_channel(msg.parameters[0].clone());
        response_vector.push(no_such_channel_message);
        Ok(response_vector)
    }
}

#[cfg(test)]
mod tests_channel {
    use crate::channel::create_valid_channel;

    #[test]
    fn test_recibir_nombre_valido_crea_canal() {
        let name = "#channel1".to_string();
        let channel = create_valid_channel(name);
        assert_eq!("#channel1", channel.unwrap().name);
    }

    #[test]
    fn test_agregar_usuario_a_canal_y_despues_encuentra_usuario() {
        let channel_name = "#channel1".to_string();
        let mut channel = create_valid_channel(channel_name).unwrap();
        let user_nickname = "nick1".to_string();
        channel.add_user(user_nickname);
        assert!(channel.has_user(&"nick1".to_string()));
    }

    #[test]
    fn test_agregar_usuario_a_canal_y_eliminarlo() {
        let channel_name = "#channel1".to_string();
        let mut channel = create_valid_channel(channel_name).unwrap();
        let user_nickname = "nick1".to_string();
        channel.add_user(user_nickname);
        assert!(channel.has_user(&"nick1".to_string()));
        channel.remove_user(&"nick1".to_string());
        assert!(!channel.has_user(&"nick1".to_string()));
    }

    #[test]
    fn test_recibir_nombre_invalido_lanza_invalid_message_error() {
        let name = "channel1".to_string();
        let channel = create_valid_channel(name);
        assert!(channel.is_err());
    }

    #[test]
    fn test_user_list() {
        let channel_name = "#channel1".to_string();
        let mut channel = create_valid_channel(channel_name).unwrap();
        let user_nickname = "nick1".to_string();
        let user_nickname2 = "nick2".to_string();
        channel.add_user(user_nickname);
        channel.add_user(user_nickname2);
        assert_eq!(channel.get_users_list(), ",nick1,nick2");
    }
}

#[cfg(test)]
mod tests_channel_msgs {

    use std::sync::{Arc, Mutex};
    //use std::sync::mpsc::channel;
    use crate::channel::{
        invite_msg, join_msg, list_msg, mode_msg, names_msg, part_msg, topic_msg, Channel,
    };
    use crate::message::Message;
    use crate::server::Server;
    use crate::user::User;

    #[test]
    fn test_join_con_nombre_valido_crea_canal() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(Vec::new()));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let result = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].command, "332");
        assert_eq!(result[1].command, "353");
        assert_eq!(result[2].command, "366");
    }

    #[test]
    fn test_join_con_nombre_invalido_devuelve_no_such_channel() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(Vec::new()));
        let msg = Message::from("JOIN canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let result = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "403");
    }
    #[test]
    fn test_tres_joins_validos_genera_resultado_correcto() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(Vec::new()));
        let msg = Message::from("JOIN #canal1,#canal2,#canal3".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let result = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        assert_eq!(result.len(), 9);
        assert_eq!(result[0].command, "332");
        assert_eq!(result[1].command, "353");
        assert_eq!(result[2].command, "366");
        assert_eq!(result[3].command, "332");
        assert_eq!(result[4].command, "353");
        assert_eq!(result[5].command, "366");
        assert_eq!(result[6].command, "332");
        assert_eq!(result[7].command, "353");
        assert_eq!(result[8].command, "366");
    }
    #[test]
    fn test_part_con_nombre_valido_y_usuario_en_canal_lo_expulsa_del_canal() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_user("nick1".to_string());
        user.add_channel(&"#canal1".to_string());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg_part = Message::from("PART #canal1".to_string());
        let result = part_msg(
            msg_part,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
        )
        .unwrap();
        assert_eq!(result.len(), 0);
    }
    #[test]
    fn test_part_con_parametros_insuficientes_devuelve_need_more_params() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_user("nick1".to_string());
        user.add_channel(&"#canal1".to_string());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg_part = Message::from("PART".to_string());
        let result = part_msg(
            msg_part,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "461");
    }

    #[test]
    fn test_part_con_canal_no_existente_devuelve_no_such_channel() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(Vec::new()));
        let msg_part = Message::from("PART #canal2".to_string());
        let result = part_msg(
            msg_part,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "403");
    }

    #[test]
    fn test_part_con_usuario_no_esta_en_ese_canal_devuelve_no_on_channel() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let channel = Channel::new(&"#canal2".to_string());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg_part = Message::from("PART #canal2".to_string());
        let result = part_msg(
            msg_part,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "442");
    }
    #[test]
    fn test_names_de_un_canal_devuelve_usuarios_del_canal() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut user2 = User::new(None);
        user2.nickname = "nick2".to_string();
        let mut user3 = User::new(None);
        user3.nickname = "nick3".to_string();
        let mut user4 = User::new(None);
        user4.nickname = "nick4".to_string();
        let users = Arc::new(Mutex::new(vec![user, user2, user3, user4]));
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_user("nick1".to_string());
        channel.add_user("nick2".to_string());
        channel.add_user("nick3".to_string());
        channel.add_user("nick4".to_string());

        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg_names = Message::from("NAMES #canal1".to_string());
        let result = names_msg(
            msg_names,
            users.clone(),
            "nick1".to_string(),
            channels.clone(),
        )
        .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].command, "353");
        assert_eq!(
            result[0].parameters,
            vec!["#canal1", " nick1 nick2 nick3 nick4"]
        );
        assert_eq!(result[1].command, "366");
    }
    #[test]
    fn test_names_de_dos_canales_devuelve_usuarios_de_canales() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut user2 = User::new(None);
        user2.nickname = "nick2".to_string();
        let mut user3 = User::new(None);
        user3.nickname = "nick3".to_string();
        let mut user4 = User::new(None);
        user4.nickname = "nick4".to_string();
        let users = Arc::new(Mutex::new(vec![user, user2, user3, user4]));
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_user("nick1".to_string());
        channel.add_user("nick2".to_string());
        let mut channel2 = Channel::new(&"#canal2".to_string());
        channel2.add_user("nick1".to_string());
        channel2.add_user("nick3".to_string());
        channel2.add_user("nick4".to_string());

        let channels = Arc::new(Mutex::new(vec![channel, channel2]));
        let msg_names = Message::from("NAMES #canal1,#canal2".to_string());
        let result = names_msg(
            msg_names,
            users.clone(),
            "nick1".to_string(),
            channels.clone(),
        )
        .unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].command, "353");
        assert_eq!(result[0].parameters, vec!["#canal1", " nick1 nick2"]);
        assert_eq!(result[1].command, "366");
        assert_eq!(result[2].command, "353");
        assert_eq!(result[2].parameters, vec!["#canal2", " nick1 nick3 nick4"]);
        assert_eq!(result[3].command, "366");
    }
    #[test]
    fn test_names_sin_parametros_devuelve_usuarios_de_canales() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut user2 = User::new(None);
        user2.nickname = "nick2".to_string();
        let mut user3 = User::new(None);
        user3.nickname = "nick3".to_string();
        let mut user4 = User::new(None);
        user4.nickname = "nick4".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_user("nick1".to_string());
        user.add_channel(&"#canal1".to_string());
        channel.add_user("nick2".to_string());
        user2.add_channel(&"#canal1".to_string());
        let mut channel2 = Channel::new(&"#canal2".to_string());
        channel2.add_user("nick1".to_string());
        user.add_channel(&"#canal2".to_string());
        channel2.add_user("nick3".to_string());
        user3.add_channel(&"#canal2".to_string());
        channel2.add_user("nick4".to_string());
        user4.add_channel(&"#canal2".to_string());

        let users = Arc::new(Mutex::new(vec![user, user2, user3, user4]));

        let channels = Arc::new(Mutex::new(vec![channel, channel2]));

        let msg_names = Message::from("NAMES".to_string());
        let result = names_msg(
            msg_names,
            users.clone(),
            "nick1".to_string(),
            channels.clone(),
        )
        .unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].command, "353");
        assert_eq!(result[0].parameters, vec!["#canal1", " nick1 nick2"]);
        assert_eq!(result[1].command, "366");
        assert_eq!(result[2].command, "353");
        assert_eq!(result[2].parameters, vec!["#canal2", " nick1 nick3 nick4"]);
        assert_eq!(result[3].command, "366");
    }
    #[test]
    fn test_names_sin_parametros_y_usuario_sin_canal_devuelve_usuarios_de_canales_y_usuario_sin_canal(
    ) {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut user2 = User::new(None);
        user2.nickname = "nick2".to_string();
        let mut user3 = User::new(None);
        user3.nickname = "nick3".to_string();
        let mut user4 = User::new(None);
        user4.nickname = "nick4".to_string();
        let mut user5 = User::new(None);
        user5.nickname = "nick5".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_user("nick1".to_string());
        user.add_channel(&"#canal1".to_string());
        channel.add_user("nick2".to_string());
        user2.add_channel(&"#canal1".to_string());
        let mut channel2 = Channel::new(&"#canal2".to_string());
        channel2.add_user("nick1".to_string());
        user.add_channel(&"#canal2".to_string());
        channel2.add_user("nick3".to_string());
        user3.add_channel(&"#canal2".to_string());
        channel2.add_user("nick4".to_string());
        user4.add_channel(&"#canal2".to_string());
        let users = Arc::new(Mutex::new(vec![user, user2, user3, user4, user5]));
        let channels = Arc::new(Mutex::new(vec![channel, channel2]));
        let msg_names = Message::from("NAMES".to_string());
        let result = names_msg(
            msg_names,
            users.clone(),
            "nick1".to_string(),
            channels.clone(),
        )
        .unwrap();
        assert_eq!(result.len(), 6);
        assert_eq!(result[0].command, "353");
        assert_eq!(result[0].parameters, vec!["#canal1", " nick1 nick2"]);
        assert_eq!(result[1].command, "366");
        assert_eq!(result[2].command, "353");
        assert_eq!(result[2].parameters, vec!["#canal2", " nick1 nick3 nick4"]);
        assert_eq!(result[3].command, "366");
        assert_eq!(result[4].command, "353");
        assert_eq!(result[4].parameters, vec!["*", " nick5"]);
    }
    #[test]
    fn test_invite_de_un_canal_devuelve_rpl_inviting() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut user2 = User::new(None);
        user2.nickname = "nick2".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_user("nick1".to_string());
        user.add_channel(&"canal1".to_string());
        let users = Arc::new(Mutex::new(vec![user, user2]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg_invite = Message::from("INVITE nick2 #canal1".to_string());
        let result = invite_msg(
            msg_invite,
            users.clone(),
            "nick1".to_string(),
            channels.clone(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "341");
    }
    #[test]
    fn test_invite_con_parametros_insuficientes_devuelve_need_more_params() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut user2 = User::new(None);
        user2.nickname = "nick2".to_string();
        let users = Arc::new(Mutex::new(vec![user, user2]));
        let channels = Arc::new(Mutex::new(Vec::new()));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_invite = Message::from("INVITE nick2".to_string());
        let result = invite_msg(
            msg_invite,
            users.clone(),
            "nick1".to_string(),
            channels.clone(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "461");
    }
    #[test]
    fn test_invite_con_nick_inexsitente_devuelve_no_such_nick() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut user2 = User::new(None);
        user2.nickname = "nick2".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_user("nick1".to_string());
        user.add_channel(&"#canal1".to_string());
        let users = Arc::new(Mutex::new(vec![user, user2]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg_invite = Message::from("INVITE nick3 #canal1".to_string());
        let result = invite_msg(
            msg_invite,
            users.clone(),
            "nick1".to_string(),
            channels.clone(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "401");
    }
    #[test]
    fn test_invite_a_canal_invite_only_sin_permiso_devuelve_chan_opriv() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut user2 = User::new(None);
        user2.nickname = "nick2".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_user("nick1".to_string());
        user.add_channel(&"canal1".to_string());
        channel.mode.activate_i();
        let users = Arc::new(Mutex::new(vec![user, user2]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg_invite = Message::from("INVITE nick2 #canal1".to_string());
        let result = invite_msg(
            msg_invite,
            users.clone(),
            "nick1".to_string(),
            channels.clone(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "482");
    }
    #[test]
    fn test_invite_a_canal_invite_only_con_permiso_devuelve_rpl_inviting() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut user2 = User::new(None);
        user2.nickname = "nick2".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_user("nick1".to_string());
        user.add_channel(&"canal1".to_string());
        channel.mode.activate_i();
        channel.add_admin("nick1".to_string());
        let users = Arc::new(Mutex::new(vec![user, user2]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg_invite = Message::from("INVITE nick2 #canal1".to_string());
        let result = invite_msg(
            msg_invite,
            users.clone(),
            "nick1".to_string(),
            channels.clone(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "341");
    }
    #[test]
    fn test_list_de_un_canal_devuelve_rpl_list() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let users = Arc::new(Mutex::new(vec![user]));
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.topic = Some("my topic".to_string());
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("LIST #canal1".to_string());
        let result = list_msg(msg, "nick1".to_string(), users.clone(), channels.clone()).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].command, "321");
        assert_eq!(result[1].command, "322");
        assert_eq!(result[1].parameters, vec!["#canal1", "#", "my topic"]);
        assert_eq!(result[2].command, "323");
    }
    #[test]
    fn test_list_de_un_canal_privado_devuelve_rpl_list_sin_topic() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let users = Arc::new(Mutex::new(vec![user]));
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.topic = Some("my topic".to_string());
        channel.mode.activate_p();
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("LIST #canal1".to_string());
        let result = list_msg(msg, "nick1".to_string(), users.clone(), channels.clone()).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].command, "321");
        assert_eq!(result[1].command, "322");
        assert_eq!(result[1].parameters, vec!["#canal1", "#"]);
        assert_eq!(result[2].command, "323");
    }
    #[test]
    fn test_list_de_un_canal_privado_con_usuario_en_canal_devuelve_rpl_list() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let users = Arc::new(Mutex::new(vec![user]));
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.topic = Some("my topic".to_string());
        channel.mode.activate_p();
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_list = Message::from("LIST #canal1".to_string());
        let result = list_msg(
            msg_list,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
        )
        .unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].command, "321");
        assert_eq!(result[1].command, "322");
        assert_eq!(result[1].parameters, vec!["#canal1", "#", "my topic"]);
        assert_eq!(result[2].command, "323");
    }
    #[test]
    fn test_list_sin_parametro_devuelve_info_de_canales() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let users = Arc::new(Mutex::new(vec![user]));
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.topic = Some("my topic".to_string());
        channel.mode.activate_p();
        let mut channel2 = Channel::new(&"#canal2".to_string());
        channel2.topic = Some("my topic2".to_string());
        let mut channel3 = Channel::new(&"#canal3".to_string());
        channel3.topic = Some("my topic3".to_string());
        let channels = Arc::new(Mutex::new(vec![channel, channel2, channel3]));
        let msg_list = Message::from("LIST".to_string());
        let result = list_msg(
            msg_list,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
        )
        .unwrap();
        assert_eq!(result.len(), 5);
        assert_eq!(result[0].command, "321");
        assert_eq!(result[1].command, "322");
        assert_eq!(result[1].parameters, vec!["#canal1", "#"]);
        assert_eq!(result[2].parameters, vec!["#canal2", "#", "my topic2"]);
        assert_eq!(result[3].parameters, vec!["#canal3", "#", "my topic3"]);
        assert_eq!(result[4].command, "323");
    }
    #[test]
    fn test_topic_con_parametros_insuficientes_devuelve_need_more_params() {
        let channels = Arc::new(Mutex::new(Vec::new()));
        let msg = Message::from("TOPIC".to_string());
        let result = topic_msg(msg, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "461");
    }
    #[test]
    fn test_topic_con_un_parametro_y_usuario_no_esta_en_canal_devuelve_not_on_channel() {
        let channel = Channel::new(&"#canal2".to_string());
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("TOPIC #canal2".to_string());
        let result = topic_msg(msg, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "442");
    }
    #[test]
    fn test_topic_con_un_parametro_usuario_en_canal_y_sin_topic_devuelve_no_topic() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(Vec::new()));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_topic = Message::from("TOPIC #canal1".to_string());
        let result = topic_msg(msg_topic, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "331");
    }
    #[test]
    fn test_topic_con_un_parametro_usuario_en_canal_y_topic_devuelve_topic() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let users = Arc::new(Mutex::new(vec![user]));
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.topic = Some("my topic".to_string());
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_topic = Message::from("TOPIC #canal1".to_string());
        let result = topic_msg(msg_topic, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "332");
        assert_eq!(result[0].parameters, ["#canal1", "my topic"]);
    }
    #[test]
    fn test_topic_con_modo_canal_y_usuario_admin_cambia_topic() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.mode.activate_t();
        channel.add_admin("nick1".to_string());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_topic = Message::from("TOPIC #canal1 :cambio el topic".to_string());
        let result = topic_msg(msg_topic, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "332");
        assert_eq!(result[0].parameters, ["#canal1", "cambio el topic"]);
    }
    #[test]
    fn test_topic_con_modo_canal_y_usuario_no_admin_devuelve_chanopriv() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.mode.activate_t();
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_topic = Message::from("TOPIC #canal1 :cambio el topic".to_string());
        let result = topic_msg(msg_topic, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "482");
    }
    #[test]
    fn test_topic_sin_modo_canal_y_usuario_admin_cambia_topic() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_topic = Message::from("TOPIC #canal1 :cambio el topic".to_string());
        let result = topic_msg(msg_topic, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "332");
        assert_eq!(result[0].parameters, ["#canal1", "cambio el topic"]);
    }
    #[test]
    fn test_mode_o_sin_activate_flag_devuelve_unknown_mode() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut user2 = User::new(None);
        user2.nickname = "nick2".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        let users = Arc::new(Mutex::new(vec![user, user2]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 o nick2".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "472");
    }
    #[test]
    fn test_mode_o_sin_usuario_operador_devuelve_chanopriv() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut user2 = User::new(None);
        user2.nickname = "nick2".to_string();
        let channel = Channel::new(&"#canal1".to_string());
        let users = Arc::new(Mutex::new(vec![user, user2]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 +o nick2".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "482");
    }

    #[test]
    fn test_mode_o_agrega_usuario_admin() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut user2 = User::new(None);
        user2.nickname = "nick2".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        let users = Arc::new(Mutex::new(vec![user, user2]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let msg2 = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let con_servers2: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));

        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let _ = join_msg(
            msg2,
            "nick2".to_string(),
            users.clone(),
            channels.clone(),
            con_servers2,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 +o nick2".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(channel.is_admin(&"nick2".to_string()));
        }
    }
    #[test]
    fn test_mode_de_o_elimina_usuario_admin() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut user2 = User::new(None);
        user2.nickname = "nick2".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        channel.add_admin("nick2".to_string());
        assert!(channel.is_admin(&"nick2".to_string()));
        let users = Arc::new(Mutex::new(vec![user, user2]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let msg2 = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let con_servers2: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let _ = join_msg(
            msg2,
            "nick2".to_string(),
            users.clone(),
            channels.clone(),
            con_servers2,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 -o nick2".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(!channel.is_admin(&"nick2".to_string()));
        }
    }
    #[test]
    fn test_mode_o_sin_usuario_en_canal_devuelve_no_such_nick() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut user2 = User::new(None);
        user2.nickname = "nick2".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        let users = Arc::new(Mutex::new(vec![user, user2]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 +o nick2".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "401");
    }
    #[test]
    fn test_mode_p_cambia_el_canal_a_privado() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        assert!(!channel.is_private());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 +p".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(channel.is_private());
        }
    }
    #[test]
    fn test_mode_de_p_cambia_el_canal_a_no_privado() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        channel.mode.activate_p();
        assert!(channel.is_private());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 -p".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(!channel.is_private());
        }
    }
    #[test]
    fn test_mode_s_cambia_el_canal_a_secreto() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        assert!(!channel.is_secret());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 +s".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(channel.is_secret());
        }
    }
    #[test]
    fn test_mode_de_s_cambia_el_canal_a_no_secreto() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        channel.mode.activate_s();
        assert!(channel.is_secret());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 -s".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(!channel.is_secret());
        }
    }
    #[test]
    fn test_mode_t_cambia_el_canal_a_topic_only_oper() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        assert!(!channel.is_topic_operator_only());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 +t".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(channel.is_topic_operator_only());
        }
    }
    #[test]
    fn test_mode_de_t_cambia_el_canal_a_no_topic_only_oper() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        channel.mode.activate_t();
        assert!(channel.is_topic_operator_only());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 -t".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(!channel.is_topic_operator_only());
        }
    }
    #[test]
    fn test_mode_i_cambia_el_canal_a_invite_only() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        assert!(!channel.is_invite_only());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 +i".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(channel.is_invite_only());
        }
    }
    #[test]
    fn test_mode_de_i_cambia_el_canal_a_no_invite_only() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        channel.mode.activate_i();
        assert!(channel.is_invite_only());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 -i".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(!channel.is_invite_only());
        }
    }
    #[test]
    fn test_mode_n_cambia_no_msg_from_outside() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        assert!(!channel.is_no_msg_outside());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 +n".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(channel.is_no_msg_outside());
        }
    }
    #[test]
    fn test_mode_de_n_cambia_a_msg_from_outside() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        channel.mode.activate_n();
        assert!(channel.is_no_msg_outside());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 -n".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(!channel.is_no_msg_outside());
        }
    }
    #[test]
    fn test_mode_m_cambia_moderated_channel() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        assert!(!channel.is_moderated());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 +m".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(channel.is_moderated());
        }
    }
    #[test]
    fn test_mode_de_m_cambia_no_moderated_channel() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        channel.mode.activate_m();
        assert!(channel.is_moderated());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 -m".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(!channel.is_moderated());
        }
    }
    #[test]
    fn test_mode_v_agrega_speakers() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut user2 = User::new(None);
        user2.nickname = "nick2".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        assert_eq!(channel.can_speak_users.len(), 0);
        let users = Arc::new(Mutex::new(vec![user, user2]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let msg2 = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let con_servers2: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let _ = join_msg(
            msg2,
            "nick2".to_string(),
            users.clone(),
            channels.clone(),
            con_servers2,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 +v nick2".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert_eq!(channel.can_speak_users.len(), 1);
            assert!(channel.can_speak(&"nick2".to_string()))
        }
    }
    #[test]
    fn test_mode_de_v_elimina_speakers() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut user2 = User::new(None);
        user2.nickname = "nick2".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        channel.add_speaker("nick2".to_string());
        assert_eq!(channel.can_speak_users.len(), 1);
        assert!(channel.can_speak(&"nick2".to_string()));
        let users = Arc::new(Mutex::new(vec![user, user2]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let msg2 = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let con_servers2: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let _ = join_msg(
            msg2,
            "nick2".to_string(),
            users.clone(),
            channels.clone(),
            con_servers2,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 -v nick2".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert_eq!(channel.can_speak_users.len(), 0);
            assert!(!channel.can_speak(&"nick2".to_string()))
        }
    }
    #[test]
    fn test_mode_l_canal_sin_limite_cambia_limite() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        assert!(!channel.has_limit());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 +l 10".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(channel.has_limit());
            assert_eq!(channel.limit, Some(10));
        }
    }
    #[test]
    fn test_mode_de_l_saca_limite() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        assert!(!channel.has_limit());
        channel.limit = Option::from(5);
        channel.mode.activate_l();
        assert!(channel.has_limit());
        assert_eq!(channel.limit, Some(5));
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 -l".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(!channel.has_limit());
            assert_eq!(channel.limit, None);
        }
    }
    #[test]
    fn test_mode_l_canal_sin_limite_y_limite_menor_a_cant_usuarios_no_cambia_limite() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut user2 = User::new(None);
        user2.nickname = "nick2".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        assert!(!channel.has_limit());
        let users = Arc::new(Mutex::new(vec![user, user2]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let msg2 = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let con_servers2: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let _ = join_msg(
            msg2,
            "nick2".to_string(),
            users.clone(),
            channels.clone(),
            con_servers2,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 +l 1".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(!channel.has_limit());
        }
    }
    #[test]
    fn test_mode_l_canal_con_limite_no_cambia_limite_de_vuelta() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        assert!(!channel.has_limit());
        channel.limit = Option::from(5);
        channel.mode.activate_l();
        assert!(channel.has_limit());
        assert_eq!(channel.limit, Some(5));
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 +l 10".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(channel.has_limit());
            assert_eq!(channel.limit, Some(5));
        }
    }
    #[test]
    fn test_mode_k_canal_sin_password_cambia_password() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        assert!(!channel.has_key());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 +k password".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(channel.has_key());
            assert_eq!(channel.password, Some("password".to_string()));
        }
    }
    #[test]
    fn test_mode_k_canal_con_password_no_cambia_password() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        assert!(!channel.has_key());
        channel.password = Some("password".to_string());
        channel.mode.activate_k();
        assert!(channel.has_key());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1 password".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 +k passwordchange".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "467");
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(channel.has_key());
            assert_eq!(channel.password, Some("password".to_string()));
        }
    }
    #[test]
    fn test_mode_de_k_saca_password() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        assert!(!channel.has_key());
        channel.password = Some("password".to_string());
        channel.mode.activate_k();
        assert!(channel.has_key());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1 password".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 -k".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(!channel.has_key());
            assert_eq!(channel.password, None);
        }
    }
    #[test]
    fn test_mode_b_agrega_ban_mask() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        assert!(channel.ban_masks.is_empty());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 +b *!*@*".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(!channel.ban_masks.is_empty());
            assert_eq!(channel.ban_masks[0], "*!*@*");
        }
    }
    #[test]
    fn test_mode_b_con_ban_mask_devuelve_ban_list() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        channel.ban_masks.push("*!*@*".to_string());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 +b".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].command, "367");
        assert_eq!(result[0].parameters, ["#canal1", "*!*@*"]);
        assert_eq!(result[1].command, "368");
    }

    #[test]
    fn test_mode_de_b_elimina_ban_mask() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        channel.ban_masks.push("*!*@*".to_string());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 -b *!*@*".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(channel.ban_masks.is_empty());
        }
    }
    #[test]
    fn test_mode_ims_cambia_el_canal_a_invite_only_moderated_y_secret() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.add_admin("nick1".to_string());
        assert!(!channel.is_invite_only());
        assert!(!channel.is_moderated());
        assert!(!channel.is_secret());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let _ = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        let msg_mode = Message::from("MODE #canal1 +ims".to_string());
        let result = mode_msg(msg_mode, "nick1".to_string(), channels.clone()).unwrap();
        assert_eq!(result.len(), 0);
        let mut lock_channel = channels.lock().unwrap();
        if let Some(channel) = lock_channel
            .iter_mut()
            .find(|channel| channel.name == "#canal1")
        {
            assert!(channel.is_invite_only());
            assert!(channel.is_moderated());
            assert!(channel.is_secret());
        }
    }
    #[test]
    fn test_join_con_canal_lleno_devuelve_canal_lleno() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.mode.activate_l();
        channel.limit = Some(2);
        channel.add_user("user1".to_string());
        channel.add_user("user2".to_string());
        assert_eq!(channel.users.len(), 2);
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));

        let result = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "471");
    }
    #[test]
    fn test_join_a_canal_invite_only_devuelve_error_invite_only() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.mode.activate_i();
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let result = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "473");
    }
    #[test]
    fn test_join_a_canal_con_clave_sin_poner_clave_devuelve_error() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.mode.activate_k();
        channel.password = Some("password".to_string());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let result = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "475");
    }

    #[test]
    fn test_join_a_canal_con_clave_con_clave_incorrecta_devuelve_error() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.mode.activate_k();
        channel.password = Some("password".to_string());
        let users = Arc::new(Mutex::new(vec![user]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg = Message::from("JOIN #canal1 invalidpassword".to_string());
        let con_servers: Arc<Mutex<Vec<Server>>> = Arc::new(Mutex::new(vec![]));
        let result = join_msg(
            msg,
            "nick1".to_string(),
            users.clone(),
            channels.clone(),
            con_servers,
            "".to_string(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "475");
    }
    #[test]
    fn test_invite_con_canal_lleno_devuelve_canal_lleno() {
        let mut user = User::new(None);
        user.nickname = "nick1".to_string();
        let mut user2 = User::new(None);
        user2.nickname = "nick2".to_string();
        let mut channel = Channel::new(&"#canal1".to_string());
        channel.mode.activate_l();
        channel.limit = Some(1);
        channel.add_user("nick1".to_string());
        user.add_channel(&"#canal1".to_string());
        let users = Arc::new(Mutex::new(vec![user, user2]));
        let channels = Arc::new(Mutex::new(vec![channel]));
        let msg_invite = Message::from("INVITE nick2 #canal1".to_string());
        let result = invite_msg(
            msg_invite,
            users.clone(),
            "nick1".to_string(),
            channels.clone(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "471");
    }

    #[test]
    fn test_is_banned_cumple_con_mask_sin_asteriscos_devuelve_true() {
        let mut channel = Channel::new(&"canal1".to_string());
        let ban_mask = "!hola@chau".to_string();
        channel.ban_masks.push(ban_mask);
        let username = "hola".to_string();
        let hostname = "chau".to_string();
        assert!(channel.is_banned(&username, &hostname));
    }
    #[test]
    fn test_is_banned_no_cumple_con_mask_con_asteriscos_devuelve_false() {
        let mut channel = Channel::new(&"canal1".to_string());
        let ban_mask = "!hola@chau".to_string();
        channel.ban_masks.push(ban_mask);
        let username = "aaahola".to_string();
        let hostname = "chau".to_string();
        assert!(!channel.is_banned(&username, &hostname));
    }
    #[test]
    fn test_is_banned_cumple_con_mask_sin_texto_username_y_asterisco_hostname_devuelve_true() {
        let mut channel = Channel::new(&"canal1".to_string());
        let ban_mask = "!@*".to_string();
        channel.ban_masks.push(ban_mask);
        let username = "hola".to_string();
        let hostname = "chau".to_string();
        assert!(channel.is_banned(&username, &hostname));
    }
    #[test]
    fn test_is_banned_cumple_con_mask_con_asterisco_username_y_asterisco_hostname_devuelve_true() {
        let mut channel = Channel::new(&"canal1".to_string());
        let ban_mask = "!*@*".to_string();
        channel.ban_masks.push(ban_mask);
        let username = "hola".to_string();
        let hostname = "chau".to_string();
        assert!(channel.is_banned(&username, &hostname));
    }
    #[test]
    fn test_is_banned_con_username_correcto_hostname_incorrecto_devuelve_false() {
        let mut channel = Channel::new(&"canal1".to_string());
        let ban_mask = "!hola@dos".to_string();
        channel.ban_masks.push(ban_mask);
        let username = "hola".to_string();
        let hostname = "chau".to_string();
        assert!(!channel.is_banned(&username, &hostname));
    }
    #[test]
    fn test_is_banned_con_username_incorrecto_hostname_correcto_devuelve_false() {
        let mut channel = Channel::new(&"canal1".to_string());
        let ban_mask = "!hola@dos".to_string();
        channel.ban_masks.push(ban_mask);
        let username = "hola".to_string();
        let hostname = "chau".to_string();
        assert!(!channel.is_banned(&username, &hostname));
    }
    #[test]
    fn test_is_banned_cumple_con_mask_sin_texto_username_y_asterisco_y_sufijo_hostname_devuelve_true(
    ) {
        let mut channel = Channel::new(&"canal1".to_string());
        let ban_mask = "!@*hau".to_string();
        channel.ban_masks.push(ban_mask);
        let username = "hola".to_string();
        let hostname = "chau".to_string();
        assert!(channel.is_banned(&username, &hostname));
    }
    #[test]
    fn test_is_banned_no_cumple_ninguna_mask_devuelve_false() {
        let mut channel = Channel::new(&"canal1".to_string());
        let ban_mask = "!@*prueba1".to_string();
        let ban_mask2 = "!@*prueba2".to_string();
        let ban_mask3 = "!@*prueba3".to_string();
        channel.ban_masks.push(ban_mask);
        channel.ban_masks.push(ban_mask2);
        channel.ban_masks.push(ban_mask3);
        let username = "hola".to_string();
        let hostname = "chau".to_string();
        assert!(!channel.is_banned(&username, &hostname));
    }
    #[test]
    fn test_is_banned_cumple_alguna_mask_devuelve_true() {
        let mut channel = Channel::new(&"canal1".to_string());
        let ban_mask = "!@*prueba1".to_string();
        let ban_mask2 = "!@*chau".to_string();
        let ban_mask3 = "!@*prueba3".to_string();
        channel.ban_masks.push(ban_mask);
        channel.ban_masks.push(ban_mask2);
        channel.ban_masks.push(ban_mask3);
        let username = "hola".to_string();
        let hostname = "chau".to_string();
        assert!(channel.is_banned(&username, &hostname));
    }
}
