use crate::app::client::Client;
use std::sync::Arc;

pub fn handle_join_button(channel_name: String, client: Arc<Client>) {
    let message = format!("JOIN {}", channel_name);
    client.send(message).expect("No se pudo enviar el mensaje");
}
