use gtk::prelude::*;
use std::{sync::Arc, thread};

use crate::app::client::Client;

pub fn handle_console_button(application: &gtk::Application, client: Arc<Client>) {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    application.add_window(&window);

    let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);

    let input_label = gtk::Label::new(Some("Message: "));
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
        let message_input = message_input.buffer().text();
        let client = client.clone();
        let _ = thread::spawn(move || {
            client
                .send(message_input)
                .expect("No se pudo enviar el mensaje");
        });

        window.close();
    });
}
