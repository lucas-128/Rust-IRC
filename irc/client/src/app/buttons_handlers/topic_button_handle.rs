use crate::app::client::Client;
use gtk::traits::{ButtonExt, ContainerExt, EntryExt, GtkWindowExt, WidgetExt};
use std::sync::Arc;

pub fn handle_topic_button(channel_name: String, client: Arc<Client>) {
    let main_vbox = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let window = Arc::new(gtk::Window::new(gtk::WindowType::Toplevel));

    let see_topic = gtk::Button::with_label("See topic");
    let change_topic = gtk::Button::with_label("Change topic");
    let input_row = gtk::Box::new(gtk::Orientation::Vertical, 2);
    input_row.add(&see_topic);
    input_row.add(&change_topic);
    main_vbox.add(&input_row);
    window.add(&main_vbox);
    window.show_all();

    let channel_see_topic = channel_name.clone();
    let client_ref_see_topic = client.clone();

    see_topic.connect_clicked(move |_| {
        let message = format!("TOPIC {}", channel_see_topic);
        client_ref_see_topic
            .send(message)
            .expect("No se pudo enviar el mensaje");
    });
    let channel_change_topic = channel_name;
    let client_ref_change_topic = client;

    change_topic.connect_clicked(move |_| {
        handle_new_topic(
            channel_change_topic.clone(),
            client_ref_change_topic.clone(),
        );
    });
}

pub fn handle_new_topic(channel_name: String, client: Arc<Client>) {
    let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let window = Arc::new(gtk::Window::new(gtk::WindowType::Toplevel));

    let input_label = gtk::Label::new(Some("Enter new topic"));
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
        let message = format!("TOPIC {} :{}", channel_name, input);
        client.send(message).expect("No se pudo enviar el mensaje");

        window.close();
    });
}
