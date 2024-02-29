use crate::app::main_app;
use gtk::{prelude::*, Button};

pub fn build_ui(app: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(app);

    window.set_title("IRC - Login");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);

    let input_address = gtk::Entry::builder().text("localhost").build();
    let row_address = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    let label_address = gtk::Label::new(Some("Direccion: "));
    row_address.add(&label_address);
    row_address.add(&input_address);

    let input_port = gtk::Entry::builder().text("8080").build();
    let row_port = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    let label_port = gtk::Label::new(Some("Puerto: "));
    row_port.add(&label_port);
    row_port.add(&input_port);

    let input_pass = gtk::Entry::new();
    input_pass.set_visibility(false);
    let row_pass = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    let label_pass = gtk::Label::new(Some("Pass: "));
    row_pass.add(&label_pass);
    row_pass.add(&input_pass);

    let input_nick = gtk::Entry::new();
    let row_nick = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    let label_nick = gtk::Label::new(Some("Nickname: "));
    row_nick.add(&label_nick);
    row_nick.add(&input_nick);

    let input_username = gtk::Entry::new();
    let row_username = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    let label_username = gtk::Label::new(Some("Username: "));
    row_username.add(&label_username);
    row_username.add(&input_username);

    let input_hostname = gtk::Entry::new();
    let row_hostname = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    let label_hostname = gtk::Label::new(Some("Hostname: "));
    row_hostname.add(&label_hostname);
    row_hostname.add(&input_hostname);

    let input_realname = gtk::Entry::new();
    let row_realname = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    let label_realname = gtk::Label::new(Some("Realname: "));
    row_realname.add(&label_realname);
    row_realname.add(&input_realname);

    vbox.add(&row_address);
    vbox.add(&row_port);

    vbox.add(&row_nick);
    vbox.add(&row_username);
    vbox.add(&row_realname);
    vbox.add(&row_pass);
    vbox.add(&row_hostname);

    let button = Button::with_label("Connect");
    vbox.add(&button);

    window.add(&vbox);
    window.show_all();

    let app_clone = app.clone();

    button.connect_clicked(move |_| {
        let address = input_address.buffer().text();
        let port = input_port.buffer().text();
        let pass = input_pass.buffer().text();
        let nickname = input_nick.buffer().text();
        let username = input_username.buffer().text();
        let hostname = input_hostname.buffer().text();
        let server = address.clone() + &port;
        let realname = ":".to_string() + &input_realname.buffer().text();

        let mut parametros: Vec<String> = Vec::new();
        parametros.push(address);
        parametros.push(port);
        parametros.push(pass);
        parametros.push(nickname);
        parametros.push(username);
        parametros.push(hostname);
        parametros.push(server);
        parametros.push(realname);

        match main_app::register_connection(parametros, &app_clone) {
            Ok(client) => {
                window.close();
                main_app::create_main_window(client, &app_clone);
            }
            Err(_) => {
                println!("Error rellenando los campos")
            }
        }
    });
}
