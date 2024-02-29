use std::sync::Arc;

use gtk::gdk;
use gtk::{
    glib,
    traits::{ButtonExt, ContainerExt, GtkMenuExt, WidgetExt},
    Inhibit,
};

use crate::app::buttons_handlers::p2p_handle::{
    close_p2p_connection, init_p2p_connection, p2p_send,
};
use crate::app::client::Client;

use super::chat_container::ChatContainer;

#[derive(Clone)]
pub struct ChattingButton {
    pub name: String,
    pub chat_button: Arc<gtk::Button>,
}

impl ChattingButton {
    pub fn new(user_nick: String) -> ChattingButton {
        let name = user_nick.clone();
        let chat_button = Arc::new(gtk::Button::with_label(&user_nick));

        let boton_tooltip = "Chat with ".to_string() + &user_nick;
        chat_button.set_tooltip_text(Some(&boton_tooltip));
        chat_button.set_widget_name("user_button");

        ChattingButton { name, chat_button }
    }

    pub fn set_button_handle(
        &self,
        chat_container: Arc<ChatContainer>,
        chat_vbox: Arc<gtk::Box>,
        //main_app: Arc<Application>,
        client: Arc<Client>,
    ) {
        //let chat_container_ref = chat_container.clone();
        self.set_left_clicked_handle(chat_vbox, chat_container);

        let recipient_name = self.name.clone();
        //let cc_ref = chat_container_ref.clone();
        self.set_right_clicked_handle(client, recipient_name);
    }

    fn set_right_clicked_handle(
        &self,
        //main_app: Arc<Application>,
        client: Arc<Client>,
        recipient_name: String,
    ) {
        let chat_button_ref = self.chat_button.clone();
        self.chat_button.connect_button_press_event(move |_, e| {
            if e.button() == 3 {
                // Presionó click derecho
                display_p2p_context_menu(
                    recipient_name.clone(),
                    client.clone(),
                    chat_button_ref.clone(),
                );
            }
            Inhibit(false)
        });
    }

    fn set_left_clicked_handle(
        &self,
        chat_vbox: Arc<gtk::Box>,
        user_container: Arc<ChatContainer>,
    ) {
        self.chat_button.connect_clicked(move |_| {
            // Remove current chat context
            for child in chat_vbox.children() {
                chat_vbox.remove(&child);
            }

            // Add the selected user elements
            chat_vbox.add(&user_container.chat_label);
            chat_vbox.add(&user_container.chat_box);

            chat_vbox.add(&user_container.input.clone());
            chat_vbox.add(&user_container.send_button.clone());

            chat_vbox.show_all();
        });
    }

    pub fn set_tooltip_window(&self, tooltip_window: gtk::Window) {
        self.chat_button.set_tooltip_window(Some(&tooltip_window));
    }

    pub fn connect_motion_event<
        F: Fn(&gtk::Button, &gdk::EventMotion) -> glib::signal::Inhibit + 'static,
    >(
        &self,
        _f: F,
    ) {
        /*self.chat_button
        .add_events(gdk::EventMask::POINTER_MOTION_MASK);
        self.chat_button.connect_motion_notify_event(f);
        FUNCIONALIDAD COMENTADA PARA FACILITAR PRUEBAS SIN ENSUCIAR LOS LOGS PARA EL EXAMEN FINAL
        ESTO SIRVE PARA MOSTRAR LOS DETALLES DE UN USUARIO AL PASAR EL MOUSE POR ENCIMA
        */
    }
}

fn display_p2p_context_menu(
    recipient_name: String,
    client: Arc<Client>,
    chat_button: Arc<gtk::Button>,
) {
    let context_menu = gtk::Menu::new();
    let p2p_connect = gtk::MenuItem::with_label("P2P Connect");
    let p2p_close = gtk::MenuItem::with_label("P2P Close");
    let p2p_send_file = gtk::MenuItem::with_label("P2P Send File");

    let client_ref = client.clone();
    let cli_ref = client.clone();
    let recipient_name_ref = recipient_name.clone();
    let recipient_n = recipient_name.clone();

    p2p_connect.connect_button_press_event(move |_, b| {
        if b.button() == 1 {
            // Mandar la solicitud P2P
            init_p2p_connection(
                client.client_address.lock().unwrap().clone(),
                client.client_port.lock().unwrap().clone(),
                recipient_name.clone(),
                client.clone(),
            );
        }
        Inhibit(false)
    });

    p2p_close.connect_button_press_event(move |_, b| {
        if b.button() == 1 {
            // Cerrar la conexion P2P
            close_p2p_connection(
                client_ref.client_address.lock().unwrap().clone(),
                client_ref.client_port.lock().unwrap().clone(),
                recipient_name_ref.clone(),
                client_ref.clone(),
            );
        }
        Inhibit(false)
    });

    p2p_send_file.connect_button_press_event(move |_, b| {
        if b.button() == 1 {
            // Enviar archivo
            p2p_send(recipient_n.clone(), cli_ref.clone());
        }
        Inhibit(false)
    });

    context_menu.add(&p2p_connect);
    context_menu.add(&p2p_close);
    context_menu.add(&p2p_send_file);

    context_menu.show_all();

    context_menu.popup_at_widget(
        chat_button.as_ref(),
        gdk::Gravity::Center,
        gdk::Gravity::NorthWest,
        None,
    );
}
