use std::{
    ops::DerefMut,
    sync::{Arc, Mutex},
};

use gtk::traits::{ButtonExt, ContainerExt, EntryExt, GtkWindowExt, WidgetExt};

use crate::app::client::Client;

pub fn handle_away_button(
    window: Arc<gtk::Window>,
    client: Arc<Client>,
    away_message: Arc<Mutex<String>>,
    afk_button: Arc<gtk::Button>,
) {
    let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);

    let input_label = gtk::Label::new(Some("Message: "));
    let input = gtk::Entry::new();
    let send_button = gtk::Button::with_label("Send");

    let input_row = gtk::Box::new(gtk::Orientation::Horizontal, 2);

    input_row.add(&input_label);
    input_row.add(&input);
    input_row.add(&send_button);

    main_vbox.add(&input_row);

    window.add(&main_vbox);
    window.set_position(gtk::WindowPosition::Center);
    window.show_all();

    send_button.connect_clicked(move |_| {
        let input = input.buffer().text();
        let nickname = client.get_nickname();
        let message = format!(":{nickname} AWAY :{input}");
        client.send(message).expect("No se pudo enviar el mensaje");

        window.close();

        *away_message.lock().unwrap().deref_mut() = input;
        afk_button.clone().set_label("I'm back!");
    });
}

pub fn handle_unaway_button(client: Arc<Client>) {
    let nickname = client.get_nickname();
    let message = format!(":{nickname} AWAY");
    client.send(message).expect("No se pudo enviar el mensaje");
}
