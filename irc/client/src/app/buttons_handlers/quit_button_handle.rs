use std::process;
use std::sync::Arc;

use gtk::prelude::*;

use crate::app::client::Client;

pub fn handle_quit_button(application: &gtk::Application, client: Arc<Client>) {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    application.add_window(&window);

    let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);

    let input_label = gtk::Label::new(Some("Quit message: "));
    let message_input = gtk::Entry::new();
    let send_button = gtk::Button::with_label("Send");

    let input_row = gtk::Box::new(gtk::Orientation::Horizontal, 2);

    input_row.add(&input_label);
    input_row.add(&message_input);
    input_row.add(&send_button);

    main_vbox.add(&input_row);

    window.add(&main_vbox);
    window.set_position(gtk::WindowPosition::Center);
    window.show_all();

    send_button.connect_clicked(move |_| {
        let quit_message_input = message_input.buffer().text();

        if quit_message_input.is_empty() {
            let quit_message =
                ":".to_string() + &client.get_nickname() + " QUIT :" + &client.get_nickname();
            client
                .send(quit_message)
                .expect("No se pudo enviar el mensaje");
            client.shutdown();
            process::exit(1);
        }

        let quit_message =
            ":".to_string() + &client.get_nickname() + " QUIT :" + &quit_message_input;
        client
            .send(quit_message)
            .expect("No se pudo enviar el mensaje");
        client.shutdown();
        process::exit(1);
    });
}

pub fn handle_squit_button(application: &gtk::Application, client: Arc<Client>) {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    application.add_window(&window);

    let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);

    let input_label = gtk::Label::new(Some("Server name: "));
    let comment_label = gtk::Label::new(Some("Comment: "));
    let server_input = gtk::Entry::new();
    let message_input = gtk::Entry::new();
    let send_button = gtk::Button::with_label("Send");

    let input_row = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    let input_row2 = gtk::Box::new(gtk::Orientation::Horizontal, 2);

    input_row.add(&input_label);
    input_row.add(&server_input);

    input_row2.add(&comment_label);
    input_row2.add(&message_input);

    main_vbox.add(&input_row);
    main_vbox.add(&input_row2);
    main_vbox.add(&send_button);

    window.add(&main_vbox);
    window.set_position(gtk::WindowPosition::Center);
    window.show_all();

    send_button.connect_clicked(move |_| {
        let quit_message_input = message_input.buffer().text();
        let mut quit_message = "SQUIT ".to_string() + &server_input.buffer().text();

        if !quit_message_input.is_empty() {
            quit_message = quit_message + " :" + &quit_message_input;
        }

        client
            .send(quit_message)
            .expect("No se pudo enviar el mensaje");

        window.close();
    });
}
