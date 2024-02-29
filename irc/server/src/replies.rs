use crate::message::Message;

///Mensaje de error que informa que se necesitan más parámetros.
pub fn err_need_more_params(command: String) -> Message {
    let line = format!("461 {} :Not enough parameters", command);
    Message::from(line)
}
///Mensaje de respuesta con el topic de un canal.
pub fn rpl_topic(channel: String, topic: String) -> Message {
    let line = format!("332 {} :{}", channel, topic);
    Message::from(line)
}
///Mensaje de respuesta con los miembros de un canal.
pub fn rpl_name_rply(channel: String, nicks: Vec<String>) -> Message {
    let mut line = format!("353 {} :", channel);
    for nick in nicks {
        line = line + " " + &nick;
    }
    Message::from(line)
}
///Mensaje de fin de respuesta de NAMES.
pub fn rpl_end_of_names(channel: String) -> Message {
    let line = format!("366 {} :End of /NAMES list", channel);
    Message::from(line)
}
///Mensaje de error que informa que no existe el canal pasado.
pub fn err_no_such_channel(channel: String) -> Message {
    let line = format!("403 {} :No such channel", channel);
    Message::from(line)
}
///Mensaje de error que informa que el usuario no se encuentra en el canal.
pub fn err_not_on_channel(channel: String) -> Message {
    let line = format!("442 {} :You're not on that channel", channel);
    Message::from(line)
}
///Mensaje de respuesta inicio mensaje LIST.
pub fn rpl_list_start() -> Message {
    let line = ("321 Channel :Users  Name").to_string();
    Message::from(line)
}
///Mensaje de respuesta con el topic de un canal.
pub fn rpl_list(channel: String, topic: String) -> Message {
    let line = format!("322 {} # :{}", channel, topic);
    Message::from(line)
}
///Mensaje de respuesta fin mensaje LIST.
pub fn rpl_list_end() -> Message {
    let line = ("323 :End of /LIST").to_string();
    Message::from(line)
}
///Mensaje de error que informa que no existe el nick pasado.
pub fn error_no_such_nick(nick: String) -> Message {
    let line = format!("401 {} :No such nick/channel", nick);
    Message::from(line)
}
///Mensaje de error que informa que el usuario ya se encuentra en el canal.
pub fn err_user_on_channel(nick: String, channel: String) -> Message {
    let line = format!("443 {} {} :is already on channel", nick, channel);
    Message::from(line)
}
///Mensaje de error que informa que se necesitan privilegios de operador de canal.
pub fn err_chan_opriv_is_needed(channel: String) -> Message {
    let line = format!("482 {} :You're not channel operator", channel);
    Message::from(line)
}
///Mensaje de respuesta mensaje INVITE.
pub fn rpl_inviting(nick: String, channel: String) -> Message {
    let line = format!("341 {} {}", channel, nick);
    Message::from(line)
}
///Mensaje de error que informa que no se especificó un receptor del mensaje.
pub fn err_no_recpient(command: String) -> Message {
    let line = format!("411 :No recipient given {}", command);
    Message::from(line)
}
///Mensaje de error que informa que no se especificó un texto para el mensaje.
pub fn err_no_text_tosend() -> Message {
    let line = ("412 :No text to send").to_string();
    Message::from(line)
}
///Mensaje de error que informa que no se puede mandar el mensaje al canal.
pub fn err_can_not_send_to_chan(channel: String) -> Message {
    let line = format!("404 {} :Cannot send to channel", channel);
    Message::from(line)
}
///Mensaje de error que informa que el nickname enviado ya está en uso.
pub fn err_nickname_in_use(nick: String) -> Message {
    let line = format!("433 :{} is already in use", nick);
    Message::from(line)
}
///Mensaje de error que informa que no se especificó un nickname.
pub fn err_no_nickname_given() -> Message {
    let line = ("431 :No nickname given").to_string();
    Message::from(line)
}
///Mensaje de respuseta que informa que se recibió permiso de operador.
pub fn rpl_you_are_oper() -> Message {
    let line = ("381 :You are now an IRC operator").to_string();
    Message::from(line)
}
///Mensaje de error que informa que no se es operador.
pub fn err_no_oper_host() -> Message {
    let line = ("491 :No O-lines for your host").to_string();
    Message::from(line)
}
///Mensaje de respuesta que brinda informacion del usuario.
pub fn rpl_whoisuser(info: String) -> Message {
    let mut line = ("311 ").to_string();
    line.push_str(&info);
    Message::from(line)
}

pub fn err_no_such_nick() -> Message {
    let line = ("401 :No such nick").to_string();
    Message::from(line)
}
///Mensaje de respuesta que brinda informacion de usuarios.
pub fn rpl_who_reply(usuarios: String) -> Message {
    let mut line = "352 :".to_string();
    line.push_str(&usuarios);
    Message::from(line)
}
///Mensaje de error que informa que ya se está registrado.
pub fn err_already_registred() -> Message {
    let line = ("462 :You may not reregister").to_string();
    Message::from(line)
}

///Mensaje de respuesta que informa que el canal no tiene topic.
pub fn rpl_no_topic(channel: String) -> Message {
    let line = format!("331 {} :No topic is set", channel);
    Message::from(line)
}
///Mensaje de respuesta que informa que el usuario es seteado como que está AWAY.
pub fn rpl_away() -> Message {
    // 306 = RPL_NOWAWAY
    let line = "306 :You have been marked as being away".to_string();
    Message::from(line)
}
///Mensaje de respuesta que informa que el usuario es seteado como que ya no está AWAY.
pub fn rpl_unaway() -> Message {
    // 305 = RPL_UNAWAY
    let line = "305 :You are no longer marked as being away".to_string();
    Message::from(line)
}
///Mensaje de error que informa que el modo especificado es desconocido.
pub fn err_unknown_mode(char: char) -> Message {
    let line = format!("472 {} :is unknown mode char to me", char);
    Message::from(line)
}
///Mensaje de error que informa que el canal alcanzó el máximo de usuarios.
pub fn err_channel_is_full(channel: String) -> Message {
    let line = format!("471 {} :Cannot join channel (+l)", channel);
    Message::from(line)
}
///Mensaje de respuesta con las ban masks.
pub fn rpl_banlist(channel: String, ban_mask: String) -> Message {
    let line = format!("367 {} {}", channel, ban_mask);
    Message::from(line)
}
///Mensaje de respuesta con el fin de mensaje OPER +b.
pub fn rpl_end_of_ban_list(channel: String) -> Message {
    let line = format!("368 {} :End of channel ban list", channel);
    Message::from(line)
}
///Mensaje de error que informa que ya se estableció una contraseña.
pub fn err_key_set(channel: String) -> Message {
    let line = format!("467 {} ::Channel key already set", channel);
    Message::from(line)
}
///Mensaje de error que informa que no se puede ingresar al canal porque es invite only.
pub fn err_invite_only_chan(channel: String) -> Message {
    let line = format!("473 {} :Cannot join channel (+i)", channel);
    Message::from(line)
}
///Mensaje de error que informa que no se puede ingresar al canal porque está banneado.
pub fn err_banned_from_chan(channel: String) -> Message {
    let line = format!("474 {} :Cannot join channel (+b)", channel);
    Message::from(line)
}
///Mensaje de error que informa que no se puede ingresar al canal porque la contraseña
/// es incorrecta.
pub fn err_bad_channel_key(channel: String) -> Message {
    let line = format!("475 {} :Cannot join channel (+k)", channel);
    Message::from(line)
}
///Mensaje de error que informa que no se cuenta con permisos de operador.
pub fn err_no_privileges() -> Message {
    let line = ("481 :Permission Denied- You're not an IRC operator").to_string();
    Message::from(line)
}
