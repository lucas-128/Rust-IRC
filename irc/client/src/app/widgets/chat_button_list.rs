use std::sync::{Arc, Mutex};

use gtk::{
    traits::{ContainerExt, GtkWindowExt, ListBoxExt, WidgetExt},
    Inhibit,
};

use crate::app::client::Client;

use super::{chat_container::ChatContainer, chatting_button::ChattingButton};

pub struct ChattingButtonList {
    pub button_list: gtk::ListBox,
    pub button_array: Vec<ChattingButton>,
}

impl ChattingButtonList {
    pub fn new() -> ChattingButtonList {
        let button_list = gtk::ListBox::new();
        let button_array: Vec<ChattingButton> = Vec::new();

        ChattingButtonList {
            button_list,
            button_array,
        }
    }

    pub fn show_all(&self) {
        self.button_list.show_all();
    }

    pub fn add(&mut self, user_button: ChattingButton) {
        // Wrappeo manualmente el botón en un widget para que no pierda el nombre
        let list_box_row = gtk::ListBoxRow::new();
        list_box_row.add(user_button.chat_button.as_ref());
        let row_widget = gtk::Widget::from(list_box_row);
        row_widget.set_widget_name(&user_button.name);
        self.button_list.add(&row_widget);
        self.button_array.push(user_button);
    }

    pub fn len(&self) -> usize {
        self.button_list.children().len()
    }

    pub fn remove_disconnected_used(
        &mut self,
        new_users: Vec<String>,
        chats: Arc<Mutex<Vec<ChatContainer>>>,
    ) {
        self.button_array
            .retain(|user_chat_button| new_users.contains(&user_chat_button.name));
        for child in self.button_list.children() {
            println!("users {:?}, widget name {}", new_users, child.widget_name());
            if !new_users.contains(&child.widget_name().to_string()) {
                self.button_list.remove(&child);
            }
        }
        let mut chat_containers_lock = chats.lock().unwrap();
        chat_containers_lock.retain(|cc| new_users.contains(&cc.name));

        self.show_all();
    }

    pub fn change_nick(
        &mut self,
        new_users: Vec<String>,
        client: Arc<Client>,
        chats: Arc<Mutex<Vec<ChatContainer>>>,
        chat_vbox: Arc<gtk::Box>,
        //app_ref: Arc<Application>
    ) {
        let mut index_to_change = 0;
        let mut user_to_change = " ".to_string();
        for (i, user) in new_users.iter().enumerate() {
            if *user != self.button_array[i].name {
                index_to_change = i;
                user_to_change = self.button_array[i].name.clone();
            }
        }

        self.button_array[index_to_change].name = new_users[index_to_change].clone();

        let button_to_remove = self
            .button_list
            .row_at_index((index_to_change) as i32)
            .unwrap();

        self.button_list.remove(&button_to_remove);

        let element_to_add = self.button_array[index_to_change].name.clone();

        let user_button = ChattingButton::new(element_to_add.clone());
        if client.get_nickname() == user_to_change {
            *client.nickname.lock().unwrap() = element_to_add.clone();
        }

        let chat_container =
            ChatContainer::new("PRIVMSG".to_string(), element_to_add, client.clone());
        chats.lock().unwrap().remove(index_to_change);
        chat_container.set_send_button_handle(client.clone());

        let chat_container_ref = Arc::new(chat_container.clone());
        chats
            .lock()
            .unwrap()
            .insert(index_to_change, chat_container);

        user_button.set_button_handle(chat_container_ref, chat_vbox, client.clone());

        let nickname = user_button.name.clone();
        user_button.connect_motion_event(move |_, _| {
            let _ = client.send("WHOIS ".to_string() + &nickname);
            Inhibit(false)
        });
        self.button_list
            .insert(user_button.chat_button.as_ref(), index_to_change as i32);

        self.show_all();
    }

    pub fn add_new_user(
        &mut self,
        new_users: Vec<String>,
        client: Arc<Client>,
        chats: Arc<Mutex<Vec<ChatContainer>>>,
        chat_vbox: Arc<gtk::Box>,
        //app_ref: Arc<Application>
    ) {
        let mut current_users: Vec<String> = Vec::new();

        for user_button in self.button_array.clone() {
            if !current_users.contains(&user_button.name) {
                current_users.push(user_button.name.clone());
            }
        }

        let mut element_to_add = " ".to_string();

        for name in new_users {
            if !current_users.contains(&name) {
                element_to_add = name;
                break;
            }
        }
        if element_to_add != *" " {
            let user_button = ChattingButton::new(element_to_add.clone());
            let chat_container =
                ChatContainer::new("PRIVMSG".to_string(), element_to_add, client.clone());
            chat_container.set_send_button_handle(client.clone());
            let chat_container_ref = Arc::new(chat_container.clone());
            chats.lock().unwrap().push(chat_container);
            user_button.set_button_handle(chat_container_ref, chat_vbox, client.clone());
            let nickname = user_button.name.clone();
            user_button.connect_motion_event(move |_, _| {
                let _ = client.send("WHOIS ".to_string() + &nickname);
                Inhibit(false)
            });
            self.add(user_button);
        }

        self.show_all();
    }

    pub fn update_user_info(
        &mut self,
        nickname: String,
        username: String,
        realname: String,
        hostname: String,
    ) {
        let button_opt = self.button_array.iter().find(|b| b.name == nickname);
        if let Some(button) = button_opt {
            let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
            let nickname_label = gtk::Label::new(Some(format!("Nickname: {nickname}").as_str()));
            main_vbox.add(&nickname_label);
            let username_label = gtk::Label::new(Some(format!("Username: {username}").as_str()));
            main_vbox.add(&username_label);
            let realname_label = gtk::Label::new(Some(format!("Realname: {realname}").as_str()));
            main_vbox.add(&realname_label);
            let hostname_label = gtk::Label::new(Some(format!("Hostname: {hostname}").as_str()));
            main_vbox.add(&hostname_label);
            main_vbox.show_all();
            let tooltip_window = gtk::Window::new(gtk::WindowType::Toplevel);
            tooltip_window.add(&main_vbox);
            tooltip_window.set_title("User info");
            tooltip_window.set_position(gtk::WindowPosition::Mouse);
            button.set_tooltip_window(tooltip_window)
        }
    }
}
