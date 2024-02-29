use std::str::Chars;

#[derive(Debug)]
/// Error que notifica que el mensaje tiene formato inválido.
pub struct InvalidMessageError {
    pub error_message: String,
}

#[derive(Debug, Clone)]
///Mecanismo de comunicación cliente-servidor y servidor-servidor. Consiste de cero o un prefijo,
/// un comando y una lista de parámetros.
pub struct Message {
    pub prefix: Option<String>,
    pub command: String,
    pub parameters: Vec<String>,
}

impl From<String> for Message {
    ///Permite convertir el contenido de un string en un struct Message.
    fn from(string: String) -> Message {
        parse_line(&string)
    }
}

impl From<Message> for String {
    ///Permite convertir un Message en un string.
    fn from(message: Message) -> Self {
        message.marshal()
    }
}

impl Message {
    ///Convierte un Message en string, juntando el prefijo, el comando
    /// y los parámetros.
    fn marshal(self) -> String {
        let mut string = String::new();
        // Marshal prefix
        if self.prefix.is_some() {
            string = string + ":" + &self.prefix.unwrap() + " ";
        }
        // Marshal command
        string = string + &self.command + " ";
        // Marshal parameters
        let vec_length = self.parameters.len();
        for i in 0..vec_length {
            let param = &self.parameters[i];
            if param.contains(' ') {
                // Si el parametro tiene "SPACES", debe indicarse con dos puntos
                string += ":";
            }
            string = string + param;
            // Agrego un espacio si hay más palabras
            if i != vec_length - 1 {
                string += " ";
            }
        }
        string
    }
}

// una mensaje consiste de un prefijo (opcional), un comando, y sus parámetros.
//usando characters.next() me voy quedando con el resto de la cadena de caracteres
///Dada una línea en forma de string, se parsea el contenido y se convierte
/// en una estructura de tipo Message, separando el prefijo, el comando y los parámetros.
fn parse_line(line: &str) -> Message {
    let mut prefix = None;
    let mut command = String::new();
    let mut parameters = Vec::new();

    let mut characters: Chars = line.chars();
    while let Some(character) = characters.next() {
        match character {
            //si empieza con prefijo
            ':' => prefix = Some(get_prefix(&mut characters)),

            character => {
                (command, parameters) = get_command_and_parameters(&mut characters, character);
                break;
            }
        }
    }
    let message = Message {
        prefix,
        command,
        parameters,
    };
    println!("{:?}", message);
    message
}

//un prefijo empieza con ':', lo sigue una serie de caracteres y termina cuando hay un espacio
fn get_prefix(characters: &mut Chars) -> String {
    let mut prefix = String::new();
    for character in characters {
        match character {
            ' ' => break,
            _ => prefix.push(character),
        }
    }
    prefix
}

fn get_command_and_parameters(
    characters: &mut Chars,
    first_character: char,
) -> (String, Vec<String>) {
    //la primera palabra es el comando y el resto son los parametros
    let command = get_command(characters, first_character);
    let parameters = get_parameters(characters);
    (command, parameters)
}

fn get_command(characters: &mut Chars, first_character: char) -> String {
    let mut command = String::new();
    command.push(first_character);
    for character in characters {
        match character {
            ' ' => break,
            _ => command.push(character),
        }
    }
    command
}
///Dada una cadena de caracteres se obtienen los parámetros de un mensaje.
fn get_parameters(characters: &mut Chars) -> Vec<String> {
    let mut parameters: Vec<String> = Vec::new();
    loop {
        let parameter = get_parameter(characters);
        if parameter.is_empty() {
            break;
        }
        parameters.push(parameter);
    }
    parameters
}

// los parametros estan separados por espacio, salvo que lo preceda el caracter ':'
fn get_parameter(characters: &mut Chars) -> String {
    // si hay ':' se activa el flag para seguir tomando el resto de los
    //caracteres como un mismo parametro
    let mut exists_colon = false;
    let mut parameter = String::new();
    for character in characters {
        match character {
            ' ' if !exists_colon => break,
            ':' => exists_colon = true,
            _ => parameter.push(character),
        }
    }
    parameter
}

#[cfg(test)]
mod tests_message {
    use crate::message::Message;

    #[test]
    fn test_linea_vacia_devuelve_prefijo_vacio_comando_vacio_y_parametros_vacios() {
        let line = String::new();
        let message = Message::from(line);
        let prefix = message.prefix;
        let command = message.command;
        let parameters = message.parameters;

        assert_eq!(prefix.is_none(), true);
        assert_eq!(command.is_empty(), true);
        assert_eq!(parameters.is_empty(), true);
    }
    #[test]
    fn test_linea_solo_prefijo_devuelve_prefijo_comando_vacio_y_parametros_vacios() {
        let line = String::from(":user1");

        let message = Message::from(line);
        let prefix = message.prefix;
        let command = message.command;
        let parameters = message.parameters;

        assert_eq!(Some("user1".to_string()), prefix);
        assert_eq!(command.is_empty(), true);
        assert_eq!(parameters.is_empty(), true);
    }
    #[test]
    fn test_linea_solo_comando_devuelve_prefijo_vacio_comando_y_parametros_vacios() {
        let line = String::from("COMMAND");

        let message = Message::from(line);
        let prefix = message.prefix;
        let command = message.command;
        let parameters = message.parameters;

        assert_eq!(prefix.is_none(), true);
        assert_eq!("COMMAND", command);
        assert_eq!(parameters.is_empty(), true);
    }
    #[test]
    fn test_linea_comando_y_un_parametro_devuelve_prefijo_vacio_comando_y_un_parametro() {
        let line = String::from("COMMAND param1");

        let message = Message::from(line);
        let prefix = message.prefix;
        let command = message.command;
        let parameters = message.parameters;

        assert_eq!(prefix.is_none(), true);
        assert_eq!("COMMAND", command);
        assert_eq!(vec!["param1"], parameters);
    }
    #[test]
    fn test_linea_comando_y_un_parametro_de_muchas_palabras_devuelve_prefijo_vacio_comando_y_un_parametro(
    ) {
        let line = String::from("COMMAND :param1 tiene muchas palabras");

        let message = Message::from(line);
        let prefix = message.prefix;
        let command = message.command;
        let parameters = message.parameters;

        assert_eq!(prefix.is_none(), true);
        assert_eq!("COMMAND", command);
        assert_eq!(vec!["param1 tiene muchas palabras"], parameters);
    }

    #[test]
    fn test_linea_comando_y_multiples_parametros_devuelve_prefijo_vacio_comando_y_multiples_parametros(
    ) {
        let line = String::from("COMMAND param1 tiene muchos parametros");

        let message = Message::from(line);
        let prefix = message.prefix;
        let command = message.command;
        let parameters = message.parameters;

        assert_eq!(prefix.is_none(), true);
        assert_eq!("COMMAND", command);
        assert_eq!(vec!["param1", "tiene", "muchos", "parametros"], parameters);
    }

    #[test]
    fn test_linea_comando_y_multiples_parametros_con_muchas_palabras_devuelve_prefijo_vacio_comando_y_multiples_parametros(
    ) {
        let line = String::from("COMMAND param1 tiene :muchos parametros");

        let message = Message::from(line);
        let prefix = message.prefix;
        let command = message.command;
        let parameters = message.parameters;

        assert_eq!(prefix.is_none(), true);
        assert_eq!("COMMAND", command);
        assert_eq!(vec!["param1", "tiene", "muchos parametros"], parameters);
    }

    #[test]
    fn test_linea_prefijo_comando_y_parametro_devuelve_prefijo_comando_y_parametro() {
        let line = String::from(":user1 COMMAND param1");

        let message = Message::from(line);
        let prefix = message.prefix;
        let command = message.command;
        let parameters = message.parameters;

        assert_eq!(Some("user1".to_string()), prefix);
        assert_eq!("COMMAND", command);
        assert_eq!(vec!["param1"], parameters);
    }
    #[test]
    fn test_linea_prefijo_comando_y_multiples_parametros_devuelve_prefijo_comando_y_parametros() {
        let line = String::from(":user1 COMMAND param1 :mensaje de segundo parametro");

        let message = Message::from(line);
        let prefix = message.prefix;
        let command = message.command;
        let parameters = message.parameters;

        assert_eq!(Some("user1".to_string()), prefix);
        assert_eq!("COMMAND", command);
        assert_eq!(vec!["param1", "mensaje de segundo parametro"], parameters);
    }

    #[test]
    fn marshal_correcto_de_un_mensaje() {
        let message = Message {
            prefix: Some("aNick".to_string()),
            command: "PRIVMSG".to_string(),
            parameters: vec!["Hola, cómo estás?".to_string()],
        };

        let string: String = message.into();

        assert_eq!(":aNick PRIVMSG :Hola, cómo estás?", string);
    }

    #[test]
    fn marshal_correcto_de_un_mensaje_sin_prefijo() {
        let message = Message {
            prefix: None,
            command: "PRIVMSG".to_string(),
            parameters: vec!["Hola, cómo estás?".to_string()],
        };

        let string: String = message.into();

        assert_eq!("PRIVMSG :Hola, cómo estás?", string);
    }

    #[test]
    fn marshal_correcto_de_un_mensaje_con_varios_parametros() {
        let message = Message {
            prefix: None,
            command: "PRIVMSG".to_string(),
            parameters: vec![
                "Hola,".to_string(),
                "cómo".to_string(),
                "estás?".to_string(),
            ],
        };

        let string: String = message.into();

        assert_eq!("PRIVMSG Hola, cómo estás?", string);
    }
}
