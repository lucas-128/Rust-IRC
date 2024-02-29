use std::{io::Write, net::TcpStream, sync::Arc};

use crate::message::Message;
use crate::replies::err_nickname_in_use;

#[derive(Debug, Clone)]
///Es la representación de un usuario de un sistema de Internet Relay Chat.
/// Cuenta con los atributos necesarios para registrarse en la red, un flag para identificar
/// si es operador, el socket al que está conectado desde la aplicación cliente, los canales
/// a los que pertenece y, si tiene, un mensaje de away.
pub struct User {
    pub password: String,
    pub nickname: String,
    pub username: String,
    pub hostname: String,
    pub server: String,
    pub realname: String,
    pub is_admin: bool,
    pub socket: Option<Arc<TcpStream>>,
    pub channels: Vec<String>,
    pub away_message: Option<String>,
}

impl User {
    /// Evalúa si el usuario ya ha pasado por la etapa de registración.
    // All params are present
    pub fn is_registered(&self) -> bool {
        !(self.password.is_empty()
            || self.nickname.is_empty()
            || self.username.is_empty()
            || self.hostname.is_empty()
            || self.server.is_empty()
            || self.realname.is_empty())
    }
    ///Crea un nuevo usuario, inicializando sus atributos.
    pub fn new(socket: Option<Arc<TcpStream>>) -> Self {
        Self {
            password: Default::default(),
            nickname: Default::default(),
            username: Default::default(),
            hostname: Default::default(),
            server: Default::default(),
            realname: Default::default(),
            is_admin: false,
            socket,
            channels: Vec::new(),
            away_message: None,
        }
    }
    ///Se utiliza para enviarle un Mensaje al usuario.
    //Precondition: User must be online ("socket" must not be None)
    pub fn send_private_message(
        &self,
        msg: crate::message::Message,
    ) -> Result<usize, std::io::Error> {
        let content: String = msg.into();
        self.socket
            .as_ref()
            .unwrap()
            .as_ref()
            .write((content + "\n").as_bytes())
    }
    ///Convierte al usuario en operador.
    pub fn become_admin(&mut self) {
        self.is_admin = true;
        println!("{} is admin!", self.nickname);
    }
    ///Evalúa si el usuario pertenece a un determinado canal.
    pub fn is_in_channel(&self, channel_name: &String) -> bool {
        self.channels.iter().any(|channel| channel_name == channel)
    }
    ///Agrega un canal a la lista de canales a los que pertenece el usuario.
    pub fn add_channel(&mut self, channel_name: &String) {
        if !self.is_in_channel(channel_name) {
            self.channels.push(channel_name.clone());
        }
    }
    ///Cuando un usuario abandona un canal, elimina el canal de la lista de canales del usuario.
    pub fn leave_channel(&mut self, channel_name: &String) {
        self.channels
            .iter()
            .position(|channel| channel_name == channel)
            .map(|position| self.channels.remove(position));
    }

    pub fn update_server_users(&self, message: String) -> Result<usize, std::io::Error> {
        if self.socket.is_some() {
            // Construir el mensaje adecuado para enviar el update con el command
            let content: String = "UPDATE_SERVER_USERS ".to_string() + &message;
            let message = Message::from(content);
            self.send_private_message(message)
        } else {
            Ok(0)
        }
    }

    pub fn nick_collision_disconnect(&self, nick: String) -> Result<usize, std::io::Error> {
        if self.socket.is_some() {
            // Construir el mensaje adecuado para enviar el update con el command
            let message = err_nickname_in_use(nick);
            self.send_private_message(message)
        } else {
            Ok(0)
        }
    }

    pub fn update_server_cannels(&self, message: String) -> Result<usize, std::io::Error> {
        // Construir el mensaje adecuado para enviar el update con el command
        let content: String = "UPDATE_SERVER_CHANNELS ".to_string() + &message;
        let message = Message::from(content);
        self.send_private_message(message)
    }
    ///Configura el mensaje de away del usuario.
    pub fn set_away_message(&mut self, away_message: Option<String>) {
        self.away_message = away_message;
    }
    ///Configura el nickname del usuario.
    pub fn set_nickname(&mut self, nick: String) {
        self.nickname = nick;
    }
    ///Configura el servidor al que está conectado el usuario.
    pub fn set_server(&mut self, servername: String) {
        self.server = servername;
    }
    ///Configura el hostname del usuario.
    pub fn set_host(&mut self, host: String) {
        self.hostname = host;
    }
    ///Configura el realname del usuario.
    pub fn set_realname(&mut self, realname: String) {
        self.realname = realname;
    }
    ///Configura el username del usuario.
    pub fn set_username(&mut self, username: String) {
        self.username = username;
    }
}
