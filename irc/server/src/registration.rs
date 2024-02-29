use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::sync::Arc;

use crate::message::{InvalidMessageError, Message};
use crate::server::Server;
use crate::server_errors::ServerError;
use crate::user::User;
///Verifica que la información de registración
///provista por la conexión entrante es correcta.
pub fn validate_connection(socket: Arc<TcpStream>) -> Result<(User, Server), ServerError> {
    let reader = BufReader::new(socket.as_ref());
    let mut lines = reader.lines();
    let mut user = User::new(Some(socket.clone()));
    let mut server = Server::new();
    while !user.is_registered() && !server.is_registered() {
        if let Some(Ok(line)) = lines.next() {
            let message = Message::from(line);
            let _ =
                register_data_for_connection(message, &mut user, &mut server, Some(socket.clone()))
                    .map_err(|e| println!("Invalid registration message: {}", e.error_message));
        }
    }
    Ok((user, server))
}

fn register_data_for_connection(
    message: Message,
    user: &mut User,
    server: &mut Server,
    socket: Option<Arc<TcpStream>>,
) -> Result<(), InvalidMessageError> {
    match message.command.as_str() {
        "PASS" => {
            check_params_lenght(&message, 1)?;
            user.password = message.parameters[0].to_owned();
        }
        "NICK" => {
            check_params_lenght(&message, 1)?;
            user.nickname = message.parameters[0].to_owned();
        }
        "USER" => {
            check_params_lenght(&message, 4)?;
            user.username = message.parameters[0].to_owned();
            user.hostname = message.parameters[1].to_owned();
            user.server = message.parameters[2].to_owned();
            user.realname = message.parameters[3].to_owned();
        }
        "SERVER" => {
            server.name = message.parameters[0].to_owned();
            server.socket = socket;
        }
        _ => {
            return Err(InvalidMessageError {
                error_message: "Not a registration command".to_owned(),
            })
        }
    };
    Ok(())
}

fn check_params_lenght(
    message: &Message,
    expected_lenght: usize,
) -> Result<(), InvalidMessageError> {
    if message.parameters.len() != expected_lenght {
        return Err(InvalidMessageError {
            error_message: "Invalid parameters length".to_owned(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests_registration {

    use crate::{
        message::Message, registration::register_data_for_connection, server::Server, user::User,
    };

    #[test]
    fn test_recibir_comando_incorrecto_devuelve_error_de_mensaje_invalido() {
        let mut user = User::new(None);
        let mut server = Server::new();
        let message = Message {
            prefix: None,
            command: "INVALID".to_string(),
            parameters: Vec::new(),
        };

        let result = register_data_for_connection(message, &mut user, &mut server, None);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().error_message,
            "Not a registration command"
        );
        assert!(!user.is_registered());
    }

    #[test]
    fn test_recibir_comando_con_cantidad_parametros_incorrecta_devuelve_error() {
        let mut user = User::new(None);
        let mut server = Server::new();
        let message = Message {
            prefix: None,
            command: "PASS".to_string(),
            parameters: Vec::new(),
        };

        let result = register_data_for_connection(message, &mut user, &mut server, None);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().error_message,
            "Invalid parameters length"
        );
    }

    #[test]
    fn test_comando_pass_setea_la_password() {
        let mut user = User::new(None);
        let mut server = Server::new();
        let message = Message {
            prefix: None,
            command: "PASS".to_string(),
            parameters: ["pass123".to_string()].to_vec(),
        };

        let _ = register_data_for_connection(message, &mut user, &mut server, None);

        assert_eq!(user.password, "pass123".to_string());
    }

    #[test]
    fn test_comando_nick_setea_el_nickname() {
        let mut user = User::new(None);
        let mut server = Server::new();
        let message = Message {
            prefix: None,
            command: "NICK".to_string(),
            parameters: ["my_nickname".to_string()].to_vec(),
        };

        let _ = register_data_for_connection(message, &mut user, &mut server, None);

        assert_eq!(user.nickname, "my_nickname".to_string());
    }

    #[test]
    fn test_comando_user_setea_datos_de_usuario() {
        let mut user = User::new(None);
        let mut server = Server::new();
        let message = Message {
            prefix: None,
            command: "USER".to_string(),
            parameters: [
                "my_username".to_string(),
                "host".to_string(),
                "server".to_string(),
                "Pablo D.".to_string(),
            ]
            .to_vec(),
        };

        let _ = register_data_for_connection(message, &mut user, &mut server, None);

        assert_eq!(user.username, "my_username".to_string());
        assert_eq!(user.hostname, "host".to_string());
        assert_eq!(user.server, "server".to_string());
        assert_eq!(user.realname, "Pablo D.".to_string());
    }
}
