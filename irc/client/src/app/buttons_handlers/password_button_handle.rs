use gtk::{prelude::*, Button};
use std::sync::Arc;

use crate::app::client::Client;

pub fn handle_password_button(
    application: &gtk::Application,
    client: Arc<Client>,
    name_label: &str,
    password_label: &str,
    button_label: &str,
) {
    let window = gtk::ApplicationWindow::new(application);
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
    let input_username = gtk::Entry::new();
    let row_username = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    let label_username = gtk::Label::new(Some(name_label));
    row_username.add(&label_username);
    row_username.add(&input_username);

    let input_pass = gtk::Entry::new();
    input_pass.set_visibility(false);
    let row_pass = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    let label_pass = gtk::Label::new(Some(password_label));
    row_pass.add(&label_pass);
    row_pass.add(&input_pass);

    vbox.add(&row_username);
    vbox.add(&row_pass);
    let button = Button::with_label(button_label);
    vbox.add(&button);
    window.add(&vbox);
    window.show_all();

    button.connect_clicked(move |_| {
        let username = input_username.buffer().text();
        let pass = input_pass.buffer().text();
        let message = format!("OPER {} {}", username, pass);
        client.send(message).expect("No se pudo enviar el mensaje");
        window.close();
    });
}
