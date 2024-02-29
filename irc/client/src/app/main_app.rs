use crate::app::client::Client;

use gtk::glib;
use gtk::prelude::*;
use gtk::Application;
use std::fs::OpenOptions;
use std::io::Read;

use std::env;
use std::path::PathBuf;

use gtk::Button;
use server::threadpool::ThreadPool;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::app::widgets::channel_button_list::ChannelButtonList;
use server::message::Message;

use super::buttons_handlers::add_channel_button_handle::{
    handle_add_channel, handle_add_channel_multiserver,
};
use super::buttons_handlers::connect_server_handle::handle_connect_server_button;
use super::buttons_handlers::password_button_handle::handle_password_button;

use super::buttons_handlers::away_button_handle::{handle_away_button, handle_unaway_button};
use super::buttons_handlers::console_button_handle::handle_console_button;
use super::buttons_handlers::nick_button_handler::handle_nick_button;
use super::buttons_handlers::quit_button_handle::{handle_quit_button, handle_squit_button};
use super::buttons_handlers::remove_channel_button_handle::handle_remove_channel;
use super::reply_codes::RPL_WHOIUSER;
use super::widgets::chat_button_list::ChattingButtonList;
use super::widgets::chat_container::ChatContainer;

struct ClientData {
    client: Arc<Client>,
    //client_address: String,
}

pub fn create_main_window(client: Arc<Client>, app: &gtk::Application) {
    //client.set_client_address(client_address.clone());
    let window = gtk::ApplicationWindow::new(app);
    window.set_title("IRC");
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1000, 600);

    let client_ref = client.clone();

    // Contenedores principales de la GUI
    let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let info_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let buttons_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    hbox.set_widget_name("main_hbox");

    let logged_label = gtk::Label::new(None);
    let display_msg = "Logged in as: ".to_string() + &client.get_nickname();
    logged_label.set_text(&display_msg);
    info_row.add(&logged_label);

    let quit_button = gtk::Button::with_label("Quit");
    let console_button = gtk::Button::with_label("Consola");
    let away_button = gtk::Button::with_label("AFK");
    let nick_button = gtk::Button::with_label("Change nick");
    let oper_button = gtk::Button::with_label("Become operator");
    let squit_button = gtk::Button::with_label("Quit Server");
    let sconnect_button = gtk::Button::with_label("Connect Server");

    buttons_row.add(&quit_button);
    buttons_row.add(&console_button);
    buttons_row.add(&away_button);
    buttons_row.add(&nick_button);
    buttons_row.add(&oper_button);
    buttons_row.add(&squit_button);
    buttons_row.add(&sconnect_button);

    quit_button.connect_clicked(glib::clone!(@weak app, @weak client_ref => move |_| {
        handle_quit_button(&app,client_ref);
    }));

    console_button.connect_clicked(glib::clone!(@weak app, @weak client_ref => move |_| {
        handle_console_button(&app,client_ref);
    }));
    oper_button.connect_clicked(glib::clone!(@weak app, @weak client_ref => move |_| {
        handle_password_button(&app,client_ref,"Username: ","Password: ","Become operator");
    }));
    squit_button.connect_clicked(glib::clone!(@weak app, @weak client_ref => move |_| {
        handle_squit_button(&app,client_ref);
    }));

    let autoreply = Arc::new(Mutex::new(String::new()));
    connect_clicked_away_button(
        Arc::new(away_button),
        app,
        client_ref.clone(),
        autoreply.clone(),
    );

    sconnect_button.connect_clicked(glib::clone!(@weak app, @weak client_ref => move |_| {
        handle_connect_server_button(&app,client_ref);
    }));
    connect_clicked_nick_button(Arc::new(nick_button), app, client_ref.clone());

    let chats: Vec<ChatContainer> = Vec::new();
    let chats_ref = Arc::new(Mutex::new(chats));

    let vbox_channels = gtk::Box::new(gtk::Orientation::Vertical, 10);
    vbox_channels.set_widget_name("vbox_channels");
    let label_channels = gtk::Label::new(Some("CHANNELS"));
    let channels_header = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    channels_header.set_widget_name("label_channels");
    channels_header.add(&label_channels);
    let add_channel_button = gtk::Button::with_label("+");
    channels_header.add(&add_channel_button);
    let remove_channel_button = gtk::Button::with_label("-");
    channels_header.add(&remove_channel_button);
    let add_channel_multiserver_button = gtk::Button::with_label("Add multiserver channel");
    channels_header.add(&add_channel_multiserver_button);
    vbox_channels.add(&channels_header);
    let _ = client.send("LIST".to_string());

    add_channel_button.connect_clicked(glib::clone!(@weak app, @weak client_ref => move |_| {
        handle_add_channel(&app,client_ref);
    }));
    add_channel_multiserver_button.connect_clicked(
        glib::clone!(@weak app, @weak client_ref => move |_| {
            handle_add_channel_multiserver(&app,client_ref);
        }),
    );

    remove_channel_button.connect_clicked(glib::clone!(@weak app, @weak client_ref => move |_| {
        handle_remove_channel(&app,client_ref);
    }));

    let vbox_chat = gtk::Box::new(gtk::Orientation::Vertical, 10);
    vbox_chat.set_widget_name("vbox_chat");
    let label_chat = gtk::Label::new(Some("Welcome To IRC Chat!"));
    label_chat.set_widget_name("label_chat");
    vbox_chat.add(&label_chat);
    let vbox_chat_ref = Arc::new(vbox_chat.clone());

    let vbox_users = gtk::Box::new(gtk::Orientation::Vertical, 10);
    vbox_users.set_widget_name("vbox_users");
    let label_users = gtk::Label::new(Some("SERVER USERS"));
    label_users.set_widget_name("label_users");

    let users_button_list = ChattingButtonList::new();
    // let channels_button_list = ChattingButtonList::new();
    let channels_button_list = ChannelButtonList::new();
    vbox_users.add(&label_users);
    vbox_users.add(&users_button_list.button_list);
    vbox_channels.add(&channels_button_list.button_list);
    let users_button_ref = Arc::new(Mutex::new(users_button_list));
    let channels_button_ref = Arc::new(Mutex::new(channels_button_list));

    // Welcome Message
    let welcome_box = gtk::ListBox::new();
    welcome_box.set_widget_name("received_messages_box");
    let welcome_message = gtk::Label::new(Some("Click on a user or channel to begin chatting!"));
    welcome_box.add(&welcome_message);
    welcome_box.show_all();
    vbox_chat.add(&welcome_box);

    let input_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    input_row.set_widget_name("input_row");
    vbox_chat.add(&input_row);

    hbox.add(&vbox_channels);
    hbox.add(&vbox_chat);
    hbox.add(&vbox_users);
    main_vbox.add(&buttons_row);
    main_vbox.add(&hbox);
    main_vbox.add(&info_row);
    window.add(&main_vbox);
    window.show_all();

    let app_ref = Arc::new(app.clone());
    let client_data = ClientData { client };

    let _ = app_loop(
        app_ref,
        client_data,
        users_button_ref,
        channels_button_ref,
        chats_ref,
        vbox_chat_ref,
        autoreply,
    );
}

fn connect_clicked_away_button(
    button: Arc<gtk::Button>,
    app: &gtk::Application,
    client: Arc<Client>,
    autoreply: Arc<Mutex<String>>,
) {
    let window = Arc::new(gtk::Window::new(gtk::WindowType::Toplevel));
    app.add_window(window.as_ref());
    let button_ref = button.clone();
    button.connect_clicked(move |_| {
        let mut afk_lock = autoreply.lock().unwrap();
        if afk_lock.deref().is_empty() {
            handle_away_button(
                window.clone(),
                client.clone(),
                autoreply.clone(),
                button_ref.clone(),
            );
        } else {
            handle_unaway_button(client.clone());
            *afk_lock.deref_mut() = String::new();
            button_ref.clone().set_label("AFK");
        }
    });
}

fn connect_clicked_nick_button(
    button: Arc<gtk::Button>,
    app: &gtk::Application,
    client: Arc<Client>,
) {
    let window = Arc::new(gtk::Window::new(gtk::WindowType::Toplevel));
    app.add_window(window.as_ref());
    button.connect_clicked(move |_| {
        handle_nick_button(window.clone(), client.clone());
    });
}

pub fn register_connection(
    parametros: Vec<String>,
    app: &gtk::Application,
) -> Result<Arc<Client>, ()> {
    let full_address = parametros[0].clone() + ":" + &parametros[1];
    let client = Arc::new(
        Client::new(full_address, parametros[3].clone())
            .expect("No se pudo inicializar el cliente"),
    );

    if parametros[0].is_empty()
        || parametros[1].is_empty()
        || parametros[2].is_empty()
        || parametros[3].is_empty()
        || parametros[4].is_empty()
        || parametros[5].is_empty()
        || parametros[6].is_empty()
        || parametros[7].is_empty()
    {
        popup_error_window("ERROR: 1 or more parameters are empty.".to_string(), app);
        return Err(()); // Algun campo esta vacio
    }

    if parametros[2].contains(char::is_whitespace) {
        popup_error_window("ERROR: Password cant contain spaces.".to_string(), app);
        return Err(());
    }

    if parametros[3].contains(char::is_whitespace) {
        popup_error_window("ERROR: Nickname cant contain spaces.".to_string(), app);
        return Err(());
    }

    if parametros[5].contains(char::is_whitespace) {
        popup_error_window("ERROR: Hostname cant contain spaces.".to_string(), app);
        return Err(());
    }

    if parametros[4].contains(char::is_whitespace) {
        popup_error_window("ERROR: Username cant contain spaces.".to_string(), app);
        return Err(());
    }

    if parametros[6].contains(char::is_whitespace) {
        popup_error_window("ERROR: Servername cant contain spaces.".to_string(), app);
        return Err(());
    }

    let pass_message = "PASS".to_string() + " " + &parametros[2];
    let nick_message = "NICK".to_string() + " " + &parametros[3];
    let user_message = "USER".to_string()
        + " "
        + &parametros[4]
        + " "
        + &parametros[5]
        + " "
        + &parametros[6]
        + " "
        + &parametros[7];

    client
        .send(pass_message)
        .expect("No se pudo enviar el mensaje");
    client
        .send(nick_message)
        .expect("No se pudo enviar el mensaje");
    client
        .send(user_message)
        .expect("No se pudo enviar el mensaje");
    Ok(client)
}

fn app_loop(
    app_ref: Arc<Application>,
    client_data: ClientData,
    users_list: Arc<Mutex<ChattingButtonList>>,
    channels_list: Arc<Mutex<ChannelButtonList>>,
    chats: Arc<Mutex<Vec<ChatContainer>>>,
    chats_vbox_ref: Arc<gtk::Box>,
    autoreply: Arc<Mutex<String>>,
) -> std::io::Result<()> {
    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let client = client_data.client;
    //let client_address = client_data.client_address;
    let tx_ref = tx.clone();
    let client_ref = client.clone();

    let _p2p_receiver_process = thread::spawn(move || {
        let listener = TcpListener::bind("localhost:0").unwrap();
        let port = listener.local_addr().unwrap().port().to_string();
        client_ref.set_client_address("localhost".to_string(), port);

        let tx_r = tx_ref.clone();
        let client_r = client_ref;

        let thread_pool = ThreadPool::new(4);
        for client_stream in listener.incoming() {
            let tx_re = tx_r.clone();
            let client_re = client_r.clone();

            // reemplazar unwrap por match error check
            let client_s = Arc::new(client_stream.unwrap());

            thread_pool.execute(move || {
                let _ = client_re.handle_p2p(tx_re.clone(), client_s);
            });
        }
    });

    let client_ref = client.clone();

    let _receiver_process = thread::spawn(move || match client.receive(tx) {
        Ok(()) => (),
        Err(e) => println!("Error reciviendo mensaje: {}", e),
    });

    // Handle received message
    rx.attach(None, move |texto| {
        let msg = Message::from(texto);
        if let Some(mut sender) = msg.prefix {
            // Display message
            if msg.command.as_str() == "PRIVMSG" {
                let display_msg = sender.clone() + " said: " + "\n" + &msg.parameters[1] + "\n";

                // Check if message is a P2P Chat Request
                if msg.parameters.len() == 4 {
                    if msg.parameters[1] == "CHAT" {
                        if !client_ref.client_is_p2p_connected(sender.clone()) {
                            let popup = Arc::new(gtk::Window::new(gtk::WindowType::Toplevel));
                            app_ref.as_ref().add_window(popup.as_ref());
                            let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
                            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 10);
                            let popup_label = gtk::Label::new(Some(&format!(
                                "El usuario {} inició una conexión P2P",
                                &sender
                            )));
                            let confirm_button = gtk::Button::with_label("Confirmar");
                            let reject_button = gtk::Button::with_label("Rechazar");
                            vbox.add(&popup_label);
                            hbox.add(&confirm_button);
                            hbox.add(&reject_button);
                            vbox.add(&hbox);
                            popup.add(&vbox);
                            popup.set_title("Iniciar chat P2P");
                            popup.set_position(gtk::WindowPosition::Center);
                            popup.show_all();

                            let popup_reject_ref = popup.clone();
                            reject_button.connect_clicked(move |_| {
                                popup_reject_ref.close();
                            });

                            let address = msg.parameters[2].clone() + ":" + &msg.parameters[3];
                            let nickname = client_ref.get_nickname();
                            let client_address = client_ref.client_address.lock().unwrap().clone();
                            let client_port = client_ref.client_port.lock().unwrap().clone();
                            let cli = client_ref.clone();
                            confirm_button.connect_clicked(move |_| {
                                popup.close();
                                let socket = TcpStream::connect(address.clone()).unwrap();
                                println!("Cliente conectado mediante p2p a {}", address.clone());

                                let request_message = ":".to_string()
                                    + nickname.as_str()
                                    + " "
                                    + "PRIVMSG "
                                    + sender.as_str()
                                    + " CHAT "
                                    + client_address.as_str()
                                    + " "
                                    + client_port.as_str();
                                let _ = cli.send(request_message);
                                cli.register_p2p_connection(sender.clone(), socket);
                            });
                        }
                        // Check if message is a P2P Close Request
                    } else if msg.parameters[1] == "CLOSE" {
                        client_ref.p2p_disconnect(sender);
                    }
                } else if msg.parameters.len() == 6 && msg.parameters[1] == "SEND" {
                    let filepath = msg.parameters[2].clone();
                    let filesize = msg.parameters[5].clone();

                    popup_file_confirmation_window(client_ref.clone(), sender, filepath, filesize);
                } else if msg.parameters.len() == 5 && msg.parameters[1] == "ACCEPT" {
                    let filepath = msg.parameters[2].clone();
                    let address = msg.parameters[3].clone();
                    let port = msg.parameters[4].clone();

                    client_ref.p2p_write_file(filepath, address, port, None);
                } else if msg.parameters.len() == 6 && msg.parameters[1] == "RESUME" {
                    let filepath = msg.parameters[2].clone();
                    let address = msg.parameters[3].clone();
                    let port = msg.parameters[4].clone();
                    let str_pos = msg.parameters[5].clone();
                    let pos: u64 = str_pos.parse().unwrap();

                    println!("Pos de lectura: {}", pos);

                    client_ref.p2p_write_file(filepath, address, port, Some(pos));
                }
                // Check if destination Chat Container is a channel
                else if (msg.parameters[0].starts_with('#'))
                    || (msg.parameters[0].starts_with('&'))
                        && sender != client_ref.clone().get_nickname()
                {
                    sender = msg.parameters[0].clone();
                    for container in chats.lock().unwrap().iter() {
                        container.attach_received_message(display_msg.clone(), sender.clone());
                    }
                } else if (!msg.parameters[0].starts_with('#'))
                    || (!msg.parameters[0].starts_with('&'))
                {
                    for container in chats.lock().unwrap().iter() {
                        container.attach_received_message(display_msg.clone(), sender.clone());
                        autoreply_if_away(
                            autoreply.lock().unwrap(),
                            &client_ref.clone(),
                            container,
                            &sender,
                        );
                    }
                }
            }
        }
        if msg.command.as_str().starts_with('4') {
            popup_error_main_window(msg.parameters.clone());
        }

        match msg.command.as_str() {
            "UPDATE_SERVER_USERS" => {
                update_user_list(
                    client_ref.clone(),
                    users_list.clone(),
                    msg.parameters,
                    chats.clone(),
                    chats_vbox_ref.clone(),
                );
            }
            // "353" => { // Mensaje recibido: "353 #[channel name] : [U1] [U2] ..."
            //     update_channel_list(client_ref.clone(),channels_list.clone(),msg.parameters,
            //     chats_vbox_ref.clone(),chats.clone());
            // }
            "322" => {
                update_channel_list(
                    client_ref.clone(),
                    channels_list.clone(),
                    msg.parameters,
                    chats_vbox_ref.clone(),
                    chats.clone(),
                );
            }
            RPL_WHOIUSER => {
                // "<nick> <user> <host> * :<real name>"
                update_user_info(users_list.clone(), msg.parameters);
            }
            "353" => {
                let names_text = format!(
                    "Members of {}: {}",
                    msg.parameters[0].clone(),
                    msg.parameters[1].clone()
                );
                popup_window(names_text, "Channel members");
            }
            "332" => {
                let topic_text = format!(
                    "{}: {}",
                    msg.parameters[0].clone(),
                    msg.parameters[1].clone()
                );
                popup_window(topic_text, "Channel topic");
            }
            "331" => {
                let topic_text = format!(
                    "{}: {}",
                    msg.parameters[0].clone(),
                    msg.parameters[1].clone()
                );
                popup_window(topic_text, "Channel topic");
            }
            "381" => {
                let op_text = msg.parameters[0].clone();
                popup_window(op_text, "IRC operator");
            }
            "367" => {
                let ban_text = format!(
                    "Ban mask for channel {}: {}",
                    msg.parameters[0].clone(),
                    msg.parameters[1].clone()
                );
                popup_window(ban_text, "Ban list");
            }
            "475" => {
                popup_password_window(client_ref.clone());
            }
            _ => {
                //
            }
        }
        glib::Continue(true)
    });

    Ok(())
}

fn autoreply_if_away(
    mutex_guard: std::sync::MutexGuard<String>,
    client_ref: &Arc<Client>,
    container: &ChatContainer,
    sender: &str,
) {
    let autoreply = mutex_guard.deref();
    if !autoreply.is_empty() {
        let client_nickname = client_ref.get_nickname();
        let formatted_autoreply =
            format!("{client_nickname} said: \n[Mensaje automático] {autoreply} \n");
        container.attach_received_message(formatted_autoreply, sender.to_owned());
    }
}

fn update_user_list(
    //app_ref: Arc<Application>,
    client: Arc<Client>,
    user_button_list: Arc<Mutex<ChattingButtonList>>,
    new_users: Vec<String>,
    chats: Arc<Mutex<Vec<ChatContainer>>>,
    chat_vbox_ref: Arc<gtk::Box>,
) {
    let current_size = user_button_list.lock().unwrap().len();
    let new_size = new_users.len();

    match new_size == current_size {
        //  cambio de NICK
        true => {
            user_button_list
                .lock()
                .unwrap()
                .change_nick(new_users, client, chats, chat_vbox_ref);
        }
        false => {
            match new_size < current_size {
                true => {
                    // If a user disconnects from server, remove it from the list.
                    user_button_list
                        .lock()
                        .unwrap()
                        .remove_disconnected_used(new_users, chats);
                }
                false => {
                    // If a user joined the server, add it to the list
                    let size_dif = new_size - current_size;
                    for _ in 0..size_dif {
                        user_button_list.lock().unwrap().add_new_user(
                            new_users.clone(),
                            client.clone(),
                            chats.clone(),
                            chat_vbox_ref.clone(),
                            //app_ref.clone()
                        );
                    }
                }
            }
        }
    }

    // Display the updated button list
    user_button_list.lock().unwrap().show_all();
}

fn update_user_info(users_list: Arc<Mutex<ChattingButtonList>>, parameters: Vec<String>) {
    let nickname = parameters[0].to_string();
    let username = parameters[1].to_string();
    let hostname = parameters[2].to_string();
    let realname = parameters[4].to_string();

    users_list
        .lock()
        .unwrap()
        .update_user_info(nickname, username, realname, hostname);
}

fn update_channel_list(
    client: Arc<Client>,
    channels_list: Arc<Mutex<ChannelButtonList>>,
    msg: Vec<String>,
    chat_vbox_ref: Arc<gtk::Box>,
    chats: Arc<Mutex<Vec<ChatContainer>>>,
) {
    channels_list
        .lock()
        .unwrap()
        .add_channel(msg[0].clone(), client, chat_vbox_ref, chats);

    // channels_list.lock().unwrap().add_channel(msg[0].clone(), client, chat_vbox_ref,chats);
}

fn popup_error_window(error_msg: String, application: &gtk::Application) {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Error");
    application.add_window(&window);
    let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let label = gtk::Label::new(Some(&error_msg));
    label.set_widget_name("error_label");
    let ok_button = gtk::Button::with_label("Continue");
    main_vbox.add(&label);
    main_vbox.add(&ok_button);
    window.add(&main_vbox);
    window.show_all();

    ok_button.connect_clicked(glib::clone!(@weak application => move |_| {
        window.close();
    }));
}

fn popup_error_main_window(parameters: Vec<String>) {
    let error_msg = parameters.join(" ");
    let window = gtk::Window::new(gtk::WindowType::Popup);
    window.set_title("Error");
    let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let label = gtk::Label::new(Some(&error_msg));
    label.set_widget_name("error_label");
    let ok_button = gtk::Button::with_label("Continue");
    main_vbox.add(&label);
    main_vbox.add(&ok_button);
    window.add(&main_vbox);
    window.show_all();

    ok_button.connect_clicked(move |_| {
        window.close();
    });
}

fn popup_window(text: String, title: &str) {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title(title);
    let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let label = gtk::Label::new(Some(&text));
    label.set_widget_name("topic_label");
    let ok_button = gtk::Button::with_label("Continue");
    main_vbox.add(&label);
    main_vbox.add(&ok_button);
    window.add(&main_vbox);
    window.show_all();

    ok_button.connect_clicked(move |_| {
        window.close();
    });
}

fn popup_password_window(client: Arc<Client>) {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_position(gtk::WindowPosition::Center);
    window.set_title("Join channel");
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
    let input_username = gtk::Entry::new();
    let row_username = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    let label_username = gtk::Label::new(Some("Channel name"));
    row_username.add(&label_username);
    row_username.add(&input_username);

    let input_pass = gtk::Entry::new();
    input_pass.set_visibility(false);
    let row_pass = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    let label_pass = gtk::Label::new(Some("Password channel"));
    row_pass.add(&label_pass);
    row_pass.add(&input_pass);

    vbox.add(&row_username);
    vbox.add(&row_pass);
    let button = Button::with_label("Join channel");
    vbox.add(&button);
    window.add(&vbox);
    window.show_all();
    button.connect_clicked(move |_| {
        let username = input_username.buffer().text();
        let pass = input_pass.buffer().text();
        let message = format!("JOIN {} {}", username, pass);
        client.send(message).expect("No se pudo enviar el mensaje");
        window.close();
    });
}

/// Crea ventana para que el usuario elija que hacer ante la llegada de un archivo (ignorar, aceptar, resumir)
fn popup_file_confirmation_window(
    client: Arc<Client>,
    recipient_name: String,
    filepath: String,
    filesize: String,
) {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_position(gtk::WindowPosition::CenterOnParent);
    window.set_title("Incoming File");
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);

    let filename_text = filepath.split('/').last().unwrap();

    let username_info = format!("Usuario emisor: {}", recipient_name);
    let filename_text = format!("Nombre del archivo: {}", filename_text);
    let filesize_text = format!("Tamanio del archivo: {} bytes", filesize);

    let label_username = gtk::Label::new(Some(&username_info));
    let label_filename = gtk::Label::new(Some(&filename_text));
    let label_filesize = gtk::Label::new(Some(&filesize_text));

    let button_row = gtk::Box::new(gtk::Orientation::Horizontal, 5);

    let accept_button = Button::with_label("Accept");
    let resume_button = Button::with_label("Resume");
    let ignore_button = Button::with_label("Ignore");
    button_row.add(&accept_button);
    button_row.add(&ignore_button);

    let resume_row = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    let input_resume = gtk::Entry::new();

    let resume_label = gtk::Label::new(Some("Posicion de lectura: "));
    resume_row.add(&resume_label);
    resume_row.add(&input_resume);
    resume_row.add(&resume_button);

    vbox.add(&label_username);
    vbox.add(&label_filename);
    vbox.add(&label_filesize);
    vbox.add(&button_row);
    vbox.add(&resume_row);

    window.add(&vbox);
    let w1 = window.clone();
    let w2 = window.clone();
    let client_ref = client.clone();
    let cli_ref = client;
    let filep = filepath.clone();
    let receiver_name = recipient_name.clone();

    window.show_all();

    // Usuario acepta el archivo entrante.
    accept_button.connect_clicked(move |_| {
        // Abrir socket en puerto random disponible
        let listener = TcpListener::bind("localhost:0").unwrap(); // Bind to port 0 to get a random available port
        let port = listener.local_addr().unwrap().port(); // Get the actual port number that was bound to
        let file_path = PathBuf::from(filep.clone());
        let filename = file_path.file_name().unwrap().to_str().unwrap();

        // Construir mensaje de accept
        let accept_msg = ":".to_string()
            + &client_ref.get_nickname()
            + " "
            + "PRIVMSG "
            + &recipient_name
            + " ACCEPT "
            + &filep
            + " "
            + "localhost "
            + &port.to_string();

        let file = get_dir_and_file(&client_ref, filename, "Accept".to_string());

        // Socket esperando para leer mensaje --> voy leyendo y escribiendo
        receive_file(listener, file);

        // Mando mensaje de accept
        let _ = client_ref.send(accept_msg);

        window.close();
    });

    resume_button.connect_clicked(move |_| {
        let pos = input_resume.buffer().text();
        if pos.is_empty() {
            println!("No puede estar vacio este campo");
            w1.close();
            return;
        }

        // Abrir socket en puerto random disponible
        let listener = TcpListener::bind("localhost:0").unwrap(); // Bind to port 0 to get a random available port
        let port = listener.local_addr().unwrap().port(); // Get the actual port number that was bound to
        let file_pth = PathBuf::from(filepath.clone());
        let filename = file_pth.file_name().unwrap().to_str().unwrap();

        // Construir mensaje de resume
        let resume_msg = ":".to_string()
            + &cli_ref.get_nickname()
            + " "
            + "PRIVMSG "
            + &receiver_name
            + " RESUME "
            + &filepath
            + " "
            + "localhost "
            + &port.to_string()
            + " "
            + &pos;

        let file = get_dir_and_file(&cli_ref, filename, "Resume".to_string());

        // Socket esperando para leer mensaje --> voy leyendo y escribiendo
        receive_file(listener, file);

        // Mando mensaje de accept
        let _ = cli_ref.send(resume_msg);
        w1.close();
    });

    ignore_button.connect_clicked(move |_| {
        // Usuario rechaza el archivo entrante.
        w2.close();
    });
}

/// Escribe en el archivo los datos leidos en el TcpListener
fn receive_file(listener: TcpListener, mut file: File) {
    let _file_receiver_process = thread::spawn(move || {
        let (mut client_stream, _socket_addr) = listener.accept().unwrap();
        let mut buffer = [0; 32];

        loop {
            let bytes_read = client_stream.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                // End of stream
                break;
            }
            let _ = file.write(&buffer[..bytes_read]);
        }
    });
}

/// Obtiene el archivo que se va a escribir durante el recibo de archivo.
fn get_dir_and_file(cli_ref: &Arc<Client>, filename: &str, modo: String) -> File {
    // Ver si existe directorio "downloads_[clientname]", si no existe lo creo
    let current_directory = env::current_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap();
    let dir_path = current_directory + "/downloads_" + &cli_ref.get_nickname();

    let path = Path::new(&dir_path);
    if !path.exists() {
        match fs::create_dir_all(dir_path.clone()) {
            Ok(_) => println!("Created directory {}", dir_path),
            Err(e) => println!("Failed to create directory {}: {}", dir_path, e),
        }
    }

    // Creo el archivo si no existe y lo abro.
    let updated_filename = change_file_extension(filename.to_string());
    let file_path = dir_path + "/" + &updated_filename;

    let mut file = OpenOptions::new().create(true).open(&file_path);

    if modo == "Resume" {
        file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&file_path);
    }

    match file {
        Ok(f) => f,
        Err(_) => File::create(file_path).unwrap(),
    }
}

/// Cambia la extension del archivo a '.zip'
pub fn change_file_extension(filename: String) -> String {
    let parts: Vec<&str> = filename.split('.').collect();

    if parts.len() == 2 {
        return parts[0].to_string() + ".zip";
    }

    let mut new_name = "".to_string();
    for part in parts {
        new_name = new_name + "." + part;
    }

    new_name += ".zip";

    new_name
}
