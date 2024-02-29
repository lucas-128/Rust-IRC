use std::sync::Arc;

use gtk::prelude::*;

use crate::app::client::Client;

pub fn handle_connect_server_button(application: &gtk::Application, client: Arc<Client>) {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    application.add_window(&window);

    let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);

    let host_label = gtk::Label::new(Some("Host: "));
    let port_label = gtk::Label::new(Some("Port: "));
    let host_input = gtk::Entry::new();
    let port_input = gtk::Entry::new();
    let send_button = gtk::Button::with_label("Connect");

    let input_row = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    let input_row2 = gtk::Box::new(gtk::Orientation::Horizontal, 2);

    input_row.add(&host_label);
    input_row.add(&host_input);

    input_row2.add(&port_label);
    input_row2.add(&port_input);

    main_vbox.add(&input_row);
    main_vbox.add(&input_row2);
    main_vbox.add(&send_button);

    window.add(&main_vbox);
    window.set_position(gtk::WindowPosition::Center);
    window.show_all();

    send_button.connect_clicked(move |_| {
        let host_input_text = host_input.buffer().text();
        let port_input_text = port_input.buffer().text();

        let connect_message = ":".to_string()
            + &client.get_nickname()
            + " "
            + "SERVER_CONNECT "
            + &host_input_text
            + " "
            + &port_input_text;

        client
            .send(connect_message)
            .expect("No se pudo enviar el mensaje");

        window.close();
    });
}
