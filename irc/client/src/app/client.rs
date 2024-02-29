use std::{
    env,
    fs::{self, File},
    io::{BufRead, BufReader, Error, Read, Seek, SeekFrom, Write},
    net::Shutdown,
    net::TcpStream,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use gtk::glib::Sender;
use server::message::Message;
use std::collections::HashMap;
use zip::{write::FileOptions, ZipWriter};

use crate::app::main_app::change_file_extension;

///Estructura que le permite a un usuario conectarse a un servidor. Se almacena
/// el nickname del usuario y el socket por el que se comunica con el servidor.
pub struct Client {
    pub nickname: Arc<Mutex<String>>,
    pub socket: TcpStream,
    pub client_address: Arc<Mutex<String>>,
    pub client_port: Arc<Mutex<String>>,
    pub p2p_conections: Arc<Mutex<HashMap<String, TcpStream>>>,
}

///Crea un nuevo cliente válido.
impl Client {
    pub fn new(address: String, nick: String) -> Result<Client, Error> {
        let socket = Self::run(&address)?;
        let nickname = Arc::new(Mutex::new(nick));
        let client_address = Arc::new(Mutex::new("".to_string()));
        let client_port = Arc::new(Mutex::new("".to_string()));
        let p2p_conections = Arc::new(Mutex::new(HashMap::new()));
        Ok(Client {
            nickname,
            socket,
            client_address,
            client_port,
            p2p_conections,
        })
    }
    ///Conecta al cliente al servidor pasado por parámetro.
    fn run(address: &String) -> std::io::Result<TcpStream> {
        let socket = TcpStream::connect(address)?;
        println!("Cliente conectado a {}", address);
        Ok(socket)
    }
    ///Permite enviarle mensajes del cliente al servidor u otros clientes p2p.
    pub fn send(&self, msg: String) -> Result<usize, Error> {
        let msg_struct = Message::from(msg.clone());
        let mut destinatario = "".to_string();

        if !msg_struct.parameters.is_empty() {
            destinatario = msg_struct.parameters[0].clone();
        }

        if self.client_is_p2p_connected(destinatario.clone()) {
            let p2p_users = self.p2p_conections.lock().unwrap();
            let mut socket = p2p_users.get(&destinatario).unwrap();
            let content = msg + "\n";
            println!("Enviando {:?}", content);
            socket.write(content.as_bytes())
        } else {
            let mut socket = &self.socket;
            let content = msg + "\n";
            socket.write(content.as_bytes())
        }
    }
    ///Permite que el cliente reciba mensajes del servidor.
    pub fn receive(&self, tx: Sender<String>) -> Result<(), Error> {
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
        Ok(())
    }

    ///Desconecta al cliente del servidor.
    pub fn shutdown(&self) {
        self.socket
            .shutdown(Shutdown::Both)
            .expect("shutdown call failed");
    }

    ///Permite que el cliente reciba mensajes de otros clientes p2p
    pub fn handle_p2p(&self, tx: Sender<String>, socket: Arc<TcpStream>) -> Result<(), Error> {
        let stream = socket.as_ref();
        let reader = BufReader::new(stream);
        let mut lines = reader.lines();

        while let Some(Ok(line)) = lines.next() {
            println!("Mensaje recibido: {:?}", line);
            match tx.send(line) {
                Ok(()) => (),
                Err(e) => eprintln!("Error enviando mensaje: {}", e),
            }
        }
        Ok(())
    }

    /// Elimina al usuario indicado del HashMap y cierra su socket.
    pub fn p2p_disconnect(&self, removed_user_nick: String) {
        let mut p2p_users = self.p2p_conections.lock().unwrap();
        let socket = p2p_users.get(&removed_user_nick).unwrap();
        socket
            .shutdown(Shutdown::Both)
            .expect("shutdown call failed");
        p2p_users.remove(&removed_user_nick);
    }

    /// Envia el archivo indicado en 'filename' al usuario indicado en 'recipient_name'
    pub fn p2p_write_file(
        &self,
        filepath: String,
        address: String,
        port: String,
        position: Option<u64>,
    ) {
        let full_address = address + ":" + &port;
        let file_path = Some(PathBuf::from(filepath));

        match file_path {
            Some(path) => {
                let mut f = File::open(path.clone()).expect("Failed to open file");
                let filename = path
                    .file_name()
                    .expect("Failed to get filename")
                    .to_str()
                    .expect("Failed to convert filename to string");

                match position {
                    Some(start_position) => {
                        let _ = f.seek(SeekFrom::Start(start_position));
                        write_and_zip_file_into_socket(f, full_address, filename);
                    }
                    None => {
                        write_and_zip_file_into_socket(f, full_address, filename);
                    }
                }
            }
            None => println!("File not found"),
        }
    }

    pub fn get_nickname(&self) -> String {
        self.nickname.lock().unwrap().to_string()
    }

    pub fn set_client_address(&self, address: String, port: String) {
        *self.client_address.lock().unwrap() = address;
        *self.client_port.lock().unwrap() = port;
    }

    /// Devuelve true si el nickname indicado en 'sender'
    /// esta conectado en forma p2p con el cliente
    pub fn client_is_p2p_connected(&self, sender: String) -> bool {
        self.p2p_conections.lock().unwrap().contains_key(&sender)
    }

    pub fn register_p2p_connection(&self, connected_user_nick: String, socket: TcpStream) {
        self.p2p_conections
            .lock()
            .unwrap()
            .insert(connected_user_nick, socket);
    }
}

/// Comprime el archivo y lo escribe en la direccion dada.
fn write_and_zip_file_into_socket(f: File, address: String, filename: &str) {
    if !is_compressed(filename.to_string()) {
        let compressed_file_path = compress_file(filename, f);
        let file = File::open(compressed_file_path.clone()).unwrap();
        write_file_into_socket(file, address);
        let _ = fs::remove_file(compressed_file_path);
    } else {
        write_file_into_socket(f, address);
    }
}

/// Comprime el archivo, devuelve la direccion del archivo ya comprimido.
fn compress_file(filename: &str, mut f: File) -> String {
    // Ver si existe directorio "temp", si no existe lo creo
    let current_directory = env::current_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap();
    let dir_path = current_directory + "/tmp";

    let path = Path::new(&dir_path);
    if !path.exists() {
        match fs::create_dir_all(dir_path.clone()) {
            Ok(_) => println!("Created directory {}", dir_path),
            Err(e) => println!("Failed to create directory {}: {}", dir_path, e),
        }
    }

    let tmp_filename = change_file_extension(filename.to_string());
    let compressed_file_path = dir_path + "/" + &tmp_filename;
    let file = File::create(compressed_file_path.clone()).unwrap();

    let mut buffer = Vec::new();
    let _ = f.read_to_end(&mut buffer);

    let options = FileOptions::default().compression_method(zip::CompressionMethod::Bzip2);

    let mut zip = ZipWriter::new(file);
    let _ = zip.start_file(filename, options);
    let _ = zip.write_all(&buffer);
    let _ = zip.finish();

    compressed_file_path
}

/// Escribe el archivo indicado en el TcpStream indicado por el address.
fn write_file_into_socket(mut f: File, address: String) {
    let mut socket = TcpStream::connect(address).unwrap();

    loop {
        let mut buf = [0; 32];
        let n = f.read(&mut buf).expect("Failed to read file");

        if n == 0 {
            break; // EOF
        }

        let _ = socket.write(&buf[..n]);
    }
}

/// Devuelve True si el archivo termina con .zip
fn is_compressed(filename: String) -> bool {
    let parts: Vec<&str> = filename.split('.').collect();
    parts[parts.len() - 1] == "zip"
}
