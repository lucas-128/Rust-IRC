use gtk::{
    prelude::DialogExtManual,
    traits::{ButtonExt, ContainerExt, DialogExt, FileChooserExt, GtkWindowExt, WidgetExt},
    FileChooserAction, FileChooserDialog,
};
use std::{fs, path::PathBuf, sync::Arc};

use crate::app::client::Client;

/// Inicia conexion p2p con el usuario indicado
pub fn init_p2p_connection(
    address: String,
    port: String,
    recipient_name: String,
    client_ref: Arc<Client>,
) {
    //No se puede iniciar conexion con uno mismo.
    if recipient_name == client_ref.get_nickname() {
        popup_message("No se puede iniciar conexion p2p con uno mismo", "Error");
        return;
    }

    //La conexion con el recipient ya es p2p.
    if client_ref.client_is_p2p_connected(recipient_name.clone()) {
        popup_message("El usuario ya esta conectado en forma p2p", "Error");
        return;
    }

    let request_message = ":".to_string()
        + &client_ref.get_nickname()
        + " "
        + "PRIVMSG "
        + &recipient_name
        + " CHAT "
        + &address
        + " "
        + &port;

    let _ = client_ref.send(request_message);
}

/// Cierra la conexion p2p con el usuario indicado
pub fn close_p2p_connection(
    address: String,
    port: String,
    recipient_name: String,
    client_ref: Arc<Client>,
) {
    //No se puede cerrar conexion con uno mismo.
    if recipient_name == client_ref.get_nickname() {
        popup_message("No se puede cerrar conexion p2p con uno mismo", "Error");
        return;
    }

    //La conexion con el recipient no es p2p.
    if !client_ref.client_is_p2p_connected(recipient_name.clone()) {
        popup_message("El usuario no esta conectado en forma p2p", "Error");
        return;
    }

    let request_message = ":".to_string()
        + &client_ref.get_nickname()
        + " "
        + "PRIVMSG "
        + &recipient_name
        + " CLOSE "
        + &address
        + " "
        + &port;

    let _ = client_ref.send(request_message);
    client_ref.p2p_disconnect(recipient_name);
}

/// Envia archivo a usuario indicado.
pub fn p2p_send(recipient_name: String, client_ref: Arc<Client>) {
    //No se puede enviar un archivo a uno mismo.
    if recipient_name == client_ref.get_nickname() {
        popup_message("No se puede enviar un archivo a uno mismo", "Error");
        return;
    }

    let path = popup_file_selection();

    if let Some(file_path) = path {
        let metadata = fs::metadata(file_path.clone()).unwrap();
        let file_size = metadata.len();

        let msg = ":".to_string()
            + &client_ref.get_nickname()
            + " "
            + "PRIVMSG "
            + &recipient_name
            + " SEND "
            + file_path.to_str().unwrap()
            + " "
            + &client_ref.client_address.lock().unwrap()
            + " "
            + &client_ref.client_port.lock().unwrap()
            + " "
            + &file_size.to_string();

        println!("Mensaje de request enviado: {}", msg);

        let _ = client_ref.send(msg);
    }
}

/// Crea una ventana de 'File Selection' para que el usuario elija el archivo que quiere enviar
fn popup_file_selection() -> Option<PathBuf> {
    // Ventana de dialogo
    let dialog = FileChooserDialog::new(
        Some("Select a file"),
        None::<&gtk::Window>,
        FileChooserAction::Open,
    );

    // Botones para cancelar
    dialog.add_buttons(&[
        ("Cancel", gtk::ResponseType::Cancel),
        ("Open", gtk::ResponseType::Ok),
    ]);

    if dialog.run() == gtk::ResponseType::Ok {
        dialog.close();
        return dialog.filename();
    }

    dialog.close();
    None
}

fn popup_message(text: &str, title: &str) {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title(title);
    let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let label = gtk::Label::new(Some(text));
    let ok_button = gtk::Button::with_label("Continue");
    main_vbox.add(&label);
    main_vbox.add(&ok_button);
    window.add(&main_vbox);
    window.show_all();

    ok_button.connect_clicked(move |_| {
        window.close();
    });
}
