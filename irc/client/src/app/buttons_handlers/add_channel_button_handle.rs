use std::sync::Arc;

use gtk::prelude::*;

use crate::app::client::Client;

pub fn handle_add_channel(application: &gtk::Application, client: Arc<Client>) {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    application.add_window(&window);

    let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);

    let input_label = gtk::Label::new(Some("Channel name: "));
    let message_input = gtk::Entry::new();
    let send_button = gtk::Button::with_label("Add");

    let input_row = gtk::Box::new(gtk::Orientation::Horizontal, 2);

    input_row.add(&input_label);
    input_row.add(&message_input);
    input_row.add(&send_button);

    main_vbox.add(&input_row);

    window.add(&main_vbox);
    window.set_position(gtk::WindowPosition::Center);
    window.show_all();

    send_button.connect_clicked(move |_| {
        let input = message_input.buffer().text();

        if !input.is_empty() {
            let msg = ":".to_string() + &client.get_nickname() + " JOIN #" + &input;
            client.send(msg).expect("No se pudo enviar el mensaje");
            window.close();
        }
    });
}
pub fn handle_add_channel_multiserver(application: &gtk::Application, client: Arc<Client>) {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    application.add_window(&window);

    let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);

    let input_label = gtk::Label::new(Some("Channel name: "));
    let message_input = gtk::Entry::new();
    let send_button = gtk::Button::with_label("Add");

    let input_row = gtk::Box::new(gtk::Orientation::Horizontal, 2);

    input_row.add(&input_label);
    input_row.add(&message_input);
    input_row.add(&send_button);

    main_vbox.add(&input_row);

    window.add(&main_vbox);
    window.set_position(gtk::WindowPosition::Center);
    window.show_all();

    send_button.connect_clicked(move |_| {
        let input = message_input.buffer().text();

        if !input.is_empty() {
            let msg = ":".to_string() + &client.get_nickname() + " JOIN &" + &input;
            client.send(msg).expect("No se pudo enviar el mensaje");
            window.close();
        }
    });
}
