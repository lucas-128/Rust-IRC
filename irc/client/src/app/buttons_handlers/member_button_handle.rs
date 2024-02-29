use crate::app::client::Client;
use gtk::traits::{ButtonExt, ContainerExt, EntryExt, GtkWindowExt, WidgetExt};
use std::sync::Arc;

pub fn handle_member_button(channel_name: String, client: Arc<Client>) {
    let main_vbox = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let window = Arc::new(gtk::Window::new(gtk::WindowType::Toplevel));

    let invite_button = gtk::Button::with_label("Invite");
    let names_button = gtk::Button::with_label("See channel members");

    let input_row = gtk::Box::new(gtk::Orientation::Vertical, 2);
    input_row.add(&invite_button);
    input_row.add(&names_button);

    main_vbox.add(&input_row);
    window.add(&main_vbox);
    window.show_all();
    let channel_names = channel_name.clone();
    let client_ref_names = client.clone();

    names_button.connect_clicked(move |_| {
        let message = format!("NAMES {}", channel_names);
        client_ref_names
            .send(message)
            .expect("No se pudo enviar el mensaje");
    });

    let channel_invite = channel_name;
    let client_ref_invite = client;
    invite_button.connect_clicked(move |_| {
        handle_invite_button(channel_invite.clone(), client_ref_invite.clone());
    });
}
pub fn handle_invite_button(channel_name: String, client: Arc<Client>) {
    let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let window = Arc::new(gtk::Window::new(gtk::WindowType::Toplevel));
    let input_label = gtk::Label::new(Some("Enter user to invite"));
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
        let message = format!("INVITE {} {}", input, channel_name);
        client.send(message).expect("No se pudo enviar el mensaje");

        window.close();
    });
}
