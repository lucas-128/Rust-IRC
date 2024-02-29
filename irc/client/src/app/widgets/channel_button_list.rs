use std::sync::{Arc, Mutex};

use gtk::traits::{ContainerExt, WidgetExt};

use crate::app::client::Client;
use crate::app::widgets::channel_button::ChannelButton;

use super::chat_container::ChatContainer;

pub struct ChannelButtonList {
    pub button_list: gtk::ListBox,
    pub button_array: Vec<ChannelButton>,
}

impl ChannelButtonList {
    pub fn new() -> ChannelButtonList {
        let button_list = gtk::ListBox::new();
        let button_array: Vec<ChannelButton> = Vec::new();

        ChannelButtonList {
            button_list,
            button_array,
        }
    }

    pub fn show_all(&self) {
        self.button_list.show_all();
    }

    pub fn add(&mut self, user_button: ChannelButton) {
        let buttons_row = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        buttons_row.add(&user_button.chat_button);
        buttons_row.add(&user_button.mode_button);
        buttons_row.add(&user_button.topic_button);
        buttons_row.add(&user_button.join_button);
        buttons_row.add(&user_button.member_button);

        self.button_list.add(&buttons_row);

        // self.button_list.add(&user_button.chat_button);
        // self.button_list.add(&user_button.mode_button);

        self.button_array.push(user_button);
    }

    pub fn add_channel(
        &mut self,
        channel_name: String,
        client: Arc<Client>,
        chat_vbox: Arc<gtk::Box>,
        chats: Arc<Mutex<Vec<ChatContainer>>>,
    ) {
        for channel in &self.button_array {
            if channel.name == channel_name {
                return; // Channel ya esta en la lista
            }
        }
        let channel_button = ChannelButton::new(channel_name.clone(), client.clone());
        let chat_container =
            ChatContainer::new("PRIVMSG".to_string(), channel_name, client.clone());
        chat_container.set_send_button_handle(client);
        let chat_container_ref = Arc::new(chat_container.clone());
        chats.lock().unwrap().push(chat_container);
        channel_button.set_button_handle(chat_container_ref, chat_vbox);
        self.add(channel_button);
        self.show_all();
    }
}
