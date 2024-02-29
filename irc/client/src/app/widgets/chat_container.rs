use std::sync::Arc;

use gtk::{gdk::ffi::GDK_KEY_Return, glib, prelude::*, ListBox};
use server::message::Message;

use crate::app::client::Client;

//use super::client::Client;

#[derive(Clone)]
pub struct ChatContainer {
    pub client_ref: Arc<Client>,
    pub name: String,

    pub chat_label: gtk::Label,

    pub chat_box: gtk::ListBox,
    pub chat_box_ref: Arc<ListBox>,

    pub send_mode: String,
    pub input: gtk::Entry,
    pub input_ref: Arc<gtk::Entry>,

    pub send_button: gtk::Button,
    pub send_button_ref: Arc<gtk::Button>,
}

impl ChatContainer {
    pub fn new(send_mode: String, nick: String, client: Arc<Client>) -> ChatContainer {
        let name = nick.clone();
        let client_ref = client;

        let chat_label = gtk::Label::new(None);
        let display_msg = "Chatting with: ".to_string() + &nick;
        chat_label.set_text(&display_msg);
        chat_label.set_widget_name("label_chat");

        let chat_box = gtk::ListBox::new();
        let chat_box_ref = Arc::new(chat_box.clone());
        chat_box.set_widget_name("received_messages_box");

        let send_button = gtk::Button::with_label("Send");
        let send_button_ref = Arc::new(send_button.clone());
        send_button.set_widget_name("send_button");

        let send_mode = send_mode;

        let input = gtk::Entry::new();
        input.set_widget_name("message_input");
        let input_ref = Arc::new(input.clone());

        ChatContainer {
            client_ref,
            name,
            chat_label,
            chat_box,
            chat_box_ref,
            send_mode,
            input,
            input_ref,
            send_button,
            send_button_ref,
        }
    }

    pub fn set_send_button_handle(&self, client: Arc<Client>) {
        let client_ref = client.clone();
        let chat_box_ref = self.chat_box_ref.clone();
        let input_ref = self.input_ref.clone();
        let send_mode = self.send_mode.clone();
        let name = self.name.clone();

        self.send_button.connect_clicked(move |_| {
            let input_message = input_ref.buffer().text();

            if input_message.is_empty() {
                return;
            }

            input_ref.set_text("");
            let client_ref = client_ref.clone();
            let chat_box_ref = chat_box_ref.clone();
            let outgoing_message = ":".to_string()
                + &client.get_nickname()
                + " "
                + &send_mode.clone()
                + " "
                + &name
                + " "
                + ":"
                + &input_message;

            let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

            let outgoing_m = outgoing_message;
            client_ref
                .send(outgoing_m.clone())
                .expect("No se pudo enviar el mensaje");
            tx.send(outgoing_m).expect("Error enviando!");

            rx.attach(None, move |user_input| {
                let msg = Message::from(user_input.clone());

                if msg.prefix != Some(msg.parameters[0].clone()) {
                    let msg_label = gtk::Label::new(None);
                    let display_msg = sent_display_message(user_input);
                    if !display_msg.is_empty() {
                        msg_label.set_text(&display_msg);
                        msg_label.set_xalign(-0.0);
                        chat_box_ref.add(&msg_label);
                        chat_box_ref.show_all();
                    }
                }

                glib::Continue(true)
            });
        });

        let send_button_ref = self.send_button_ref.clone();
        self.input_ref
            .clone()
            .connect_key_press_event(move |_, key| {
                if (*key.keyval()) == GDK_KEY_Return.try_into().unwrap() {
                    //Si el usuario presiona Enter
                    send_button_ref.emit_clicked();
                }
                Inhibit(false)
            });
    }

    pub fn attach_received_message(&self, message: String, sender: String) {
        if sender == self.name {
            let msg_label = gtk::Label::new(None);
            msg_label.set_text(&message);
            msg_label.set_xalign(-0.0);
            self.chat_box.add(&msg_label);
            self.chat_box.show_all();
        }
    }

    /* pub fn listen_p2p_channel_reicever(self, channel_receiver : Receiver<(String, String)>) {
        let _ = thread::spawn(move || {
            while let Ok(new_msg) = channel_receiver.recv() {
                self.attach_received_message(new_msg.0, new_msg.1);
            }
        });
    } */
}

fn sent_display_message(text: String) -> String {
    let message = Message::from(text.clone());
    let display_msg: String;

    if let Some(sender) = message.prefix {
        // Display message

        if message.command.as_str() == "PRIVMSG" {
            display_msg = sender + " said: " + "\n" + &message.parameters[1] + "\n";

            return display_msg;
        }
    }

    text + "\n"
}
