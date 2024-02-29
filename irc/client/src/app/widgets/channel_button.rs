use std::sync::Arc;

use crate::app::buttons_handlers::join_button_handle::handle_join_button;
use crate::app::buttons_handlers::member_button_handle::handle_member_button;
use crate::app::buttons_handlers::mode_button_handle::handle_mode_button;
use crate::app::buttons_handlers::topic_button_handle::handle_topic_button;
use crate::app::client::Client;
use gtk::traits::{ButtonExt, ContainerExt, WidgetExt};

use super::chat_container::ChatContainer;

#[derive(Clone)]
pub struct ChannelButton {
    pub client_ref: Arc<Client>,
    pub name: String,
    pub chat_button: gtk::Button,
    pub mode_button: gtk::Button,
    pub topic_button: gtk::Button,
    pub join_button: gtk::Button,
    pub member_button: gtk::Button,
}

impl ChannelButton {
    pub fn new(channel_name: String, client: Arc<Client>) -> ChannelButton {
        let name = channel_name.clone();
        let client_ref = client;
        let chat_button = gtk::Button::with_label(&channel_name);
        let mode_button = gtk::Button::with_label("Operator actions");
        let member_button = gtk::Button::with_label("Member options");
        let topic_button = gtk::Button::with_label("Topic options");
        let join_button = gtk::Button::with_label("Join");

        let boton_tooltip = "Chat with ".to_string() + &channel_name;
        chat_button.set_tooltip_text(Some(&boton_tooltip));
        chat_button.set_widget_name("user_button");

        ChannelButton {
            client_ref,
            name,
            chat_button,
            mode_button,
            topic_button,
            join_button,
            member_button,
        }
    }

    pub fn set_button_handle(&self, user_container: Arc<ChatContainer>, chat_vbox: Arc<gtk::Box>) {
        let channel_name = self.name.clone();
        let client = self.client_ref.clone();

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
        let channel_topic = channel_name.clone();
        let client_topic = client.clone();
        let channel_join = channel_name.clone();
        let client_join = client.clone();
        let channel_member = channel_name.clone();
        let client_member = client.clone();

        self.mode_button.connect_clicked(move |_| {
            handle_mode_button(channel_name.clone(), client.clone());
        });
        self.member_button.connect_clicked(move |_| {
            handle_member_button(channel_member.clone(), client_member.clone());
        });
        self.topic_button.connect_clicked(move |_| {
            handle_topic_button(channel_topic.clone(), client_topic.clone());
        });

        self.join_button.connect_clicked(move |_| {
            handle_join_button(channel_join.clone(), client_join.clone());
        });
    }

    /*pub fn set_tooltip_window(&self, tooltip_window: gtk::Window) {
        self.chat_button.set_tooltip_window(Some(&tooltip_window));
    }

    pub fn connect_motion_event<
        F: Fn(&gtk::Button, &gdk::EventMotion) -> glib::signal::Inhibit + 'static,
    >(&self, f: F) {
        self.chat_button.add_events(gdk::EventMask::POINTER_MOTION_MASK);
        self.chat_button.connect_motion_notify_event(f);
    }*/
}
