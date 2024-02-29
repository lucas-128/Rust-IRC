use crate::app::client::Client;
use gtk::traits::{ButtonExt, ContainerExt, EntryExt, GridExt, GtkWindowExt, WidgetExt};
use std::sync::Arc;

pub fn handle_mode_button(channel_name: String, client: Arc<Client>) {
    let grid = gtk::Grid::new();

    let window = Arc::new(gtk::Window::new(gtk::WindowType::Toplevel));

    let password_button = gtk::Button::with_label("Set password for channel");
    let remove_password_button = gtk::Button::with_label("Remove password for channel");

    grid.attach(&password_button, 0, 0, 2, 1);
    grid.attach_next_to(
        &remove_password_button,
        Some(&password_button),
        gtk::PositionType::Right,
        1,
        1,
    );

    let private_button = gtk::Button::with_label("Make channel private");
    let remove_private_button = gtk::Button::with_label("Remove private restriction");
    grid.attach(&private_button, 0, 1, 2, 1);
    grid.attach_next_to(
        &remove_private_button,
        Some(&private_button),
        gtk::PositionType::Right,
        1,
        1,
    );

    let secret_button = gtk::Button::with_label("Make channel secret");
    let remove_secret_button = gtk::Button::with_label("Remove secret restriction");
    grid.attach(&secret_button, 0, 2, 2, 1);
    grid.attach_next_to(
        &remove_secret_button,
        Some(&secret_button),
        gtk::PositionType::Right,
        1,
        1,
    );

    let invite_only_button = gtk::Button::with_label("Make channel invite only");
    let remove_invite_only_button = gtk::Button::with_label("Remove invite only restriction");
    grid.attach(&invite_only_button, 0, 3, 2, 1);
    grid.attach_next_to(
        &remove_invite_only_button,
        Some(&invite_only_button),
        gtk::PositionType::Right,
        1,
        1,
    );

    let topic_button = gtk::Button::with_label("Make topic settable by operator only");
    let remove_topic_button = gtk::Button::with_label("Remove topic restriction");
    grid.attach(&topic_button, 0, 4, 2, 1);
    grid.attach_next_to(
        &remove_topic_button,
        Some(&topic_button),
        gtk::PositionType::Right,
        1,
        1,
    );

    let outside_button = gtk::Button::with_label("Forbid messages from outside");
    let remove_outside_button = gtk::Button::with_label("Allow messages from outside");
    grid.attach(&outside_button, 0, 5, 2, 1);
    grid.attach_next_to(
        &remove_outside_button,
        Some(&outside_button),
        gtk::PositionType::Right,
        1,
        1,
    );

    let moderated_button = gtk::Button::with_label("Make channel moderated");
    let remove_moderated_button = gtk::Button::with_label("Remove moderated restriction");
    grid.attach(&moderated_button, 0, 6, 2, 1);
    grid.attach_next_to(
        &remove_moderated_button,
        Some(&moderated_button),
        gtk::PositionType::Right,
        1,
        1,
    );

    let operator_button = gtk::Button::with_label("Make someone channel operator");
    let remove_operator_button = gtk::Button::with_label("Deop channel member");
    grid.attach(&operator_button, 0, 7, 2, 1);
    grid.attach_next_to(
        &remove_operator_button,
        Some(&operator_button),
        gtk::PositionType::Right,
        1,
        1,
    );

    let speak_button = gtk::Button::with_label("Let someone speak on moderated channel");
    let remove_speak_button = gtk::Button::with_label("Forbid someone speak on moderated channel");
    grid.attach(&speak_button, 0, 8, 2, 1);
    grid.attach_next_to(
        &remove_speak_button,
        Some(&speak_button),
        gtk::PositionType::Right,
        1,
        1,
    );

    let limit_button = gtk::Button::with_label("Set user limit");
    let remove_limit_button = gtk::Button::with_label("Unset user limit");
    grid.attach(&limit_button, 0, 9, 2, 1);
    grid.attach_next_to(
        &remove_limit_button,
        Some(&limit_button),
        gtk::PositionType::Right,
        1,
        1,
    );

    let ban_button = gtk::Button::with_label("Set ban mask");
    let remove_ban_button = gtk::Button::with_label("Unset ban mask");
    grid.attach(&ban_button, 0, 10, 2, 1);
    grid.attach_next_to(
        &remove_ban_button,
        Some(&ban_button),
        gtk::PositionType::Right,
        1,
        1,
    );

    let see_ban_button = gtk::Button::with_label("See set ban masks");
    grid.attach(&see_ban_button, 0, 11, 2, 1);

    let kick_button = gtk::Button::with_label("Kick a channel member");
    grid.attach_next_to(
        &kick_button,
        Some(&see_ban_button),
        gtk::PositionType::Right,
        1,
        1,
    );

    window.add(&grid);

    window.show_all();
    let channel_priv = channel_name.clone();
    let client_ref_priv = client.clone();

    private_button.connect_clicked(move |_| {
        let message = format!("MODE {} +p", channel_priv);
        client_ref_priv
            .send(message)
            .expect("No se pudo enviar el mensaje");
    });
    let channel_privremove = channel_name.clone();
    let client_ref_privremove = client.clone();

    remove_private_button.connect_clicked(move |_| {
        let message = format!("MODE {} -p", channel_privremove);
        client_ref_privremove
            .send(message)
            .expect("No se pudo enviar el mensaje");
    });

    let channel_secret = channel_name.clone();
    let client_ref_secret = client.clone();
    secret_button.connect_clicked(move |_| {
        let message = format!("MODE {} +s", channel_secret);
        client_ref_secret
            .send(message)
            .expect("No se pudo enviar el mensaje");
    });
    let channel_secretremove = channel_name.clone();
    let client_ref_secretremove = client.clone();
    remove_secret_button.connect_clicked(move |_| {
        let message = format!("MODE {} -s", channel_secretremove);
        client_ref_secretremove
            .send(message)
            .expect("No se pudo enviar el mensaje");
    });

    let channel_invite = channel_name.clone();
    let client_invite = client.clone();
    invite_only_button.connect_clicked(move |_| {
        let message = format!("MODE {} +i", channel_invite);
        client_invite
            .send(message)
            .expect("No se pudo enviar el mensaje");
    });
    let channel_inviteremove = channel_name.clone();
    let client_inviteremove = client.clone();
    remove_invite_only_button.connect_clicked(move |_| {
        let message = format!("MODE {} -i", channel_inviteremove);
        client_inviteremove
            .send(message)
            .expect("No se pudo enviar el mensaje");
    });

    let channel_topic = channel_name.clone();
    let client_topic = client.clone();
    topic_button.connect_clicked(move |_| {
        let message = format!("MODE {} +t", channel_topic);
        client_topic
            .send(message)
            .expect("No se pudo enviar el mensaje");
    });
    let channel_topicremove = channel_name.clone();
    let client_topicremove = client.clone();
    remove_topic_button.connect_clicked(move |_| {
        let message = format!("MODE {} -t", channel_topicremove);
        client_topicremove
            .send(message)
            .expect("No se pudo enviar el mensaje");
    });
    let channel_outside = channel_name.clone();
    let client_outside = client.clone();
    outside_button.connect_clicked(move |_| {
        let message = format!("MODE {} +n", channel_outside);
        client_outside
            .send(message)
            .expect("No se pudo enviar el mensaje");
    });
    let channel_outsideremove = channel_name.clone();
    let client_outsideremove = client.clone();
    remove_outside_button.connect_clicked(move |_| {
        let message = format!("MODE {} -n", channel_outsideremove);
        client_outsideremove
            .send(message)
            .expect("No se pudo enviar el mensaje");
    });

    let channel_moderated = channel_name.clone();
    let client_moderated = client.clone();
    moderated_button.connect_clicked(move |_| {
        let message = format!("MODE {} +m", channel_moderated);
        client_moderated
            .send(message)
            .expect("No se pudo enviar el mensaje");
    });
    let channel_moderatedremove = channel_name.clone();
    let client_moderatedremove = client.clone();
    remove_moderated_button.connect_clicked(move |_| {
        let message = format!("MODE {} -m", channel_moderatedremove);
        client_moderatedremove
            .send(message)
            .expect("No se pudo enviar el mensaje");
    });

    let channel_password = channel_name.clone();
    let client_password = client.clone();
    password_button.connect_clicked(move |_| {
        handle_mode_input(
            channel_password.clone(),
            client_password.clone(),
            "Password: ",
            "+k".to_string(),
        );
    });
    let channel_passwordremove = channel_name.clone();
    let client_passwordremove = client.clone();
    remove_password_button.connect_clicked(move |_| {
        let message = format!("MODE {} -k", channel_passwordremove);
        client_passwordremove
            .send(message)
            .expect("No se pudo enviar el mensaje");
    });

    let channel_operator = channel_name.clone();
    let client_operator = client.clone();
    operator_button.connect_clicked(move |_| {
        handle_mode_input(
            channel_operator.clone(),
            client_operator.clone(),
            "Nickname of new operator: ",
            "+o".to_string(),
        );
    });
    let channel_operatorremove = channel_name.clone();
    let client_operatorremove = client.clone();
    remove_operator_button.connect_clicked(move |_| {
        handle_mode_input(
            channel_operatorremove.clone(),
            client_operatorremove.clone(),
            "Nickname of removed operator: ",
            "-o".to_string(),
        );
    });
    let channel_speak = channel_name.clone();
    let client_speak = client.clone();
    speak_button.connect_clicked(move |_| {
        handle_mode_input(
            channel_speak.clone(),
            client_speak.clone(),
            "Nickname of new speaker: ",
            "+v".to_string(),
        );
    });
    let channel_speakremove = channel_name.clone();
    let client_speakremove = client.clone();
    remove_speak_button.connect_clicked(move |_| {
        handle_mode_input(
            channel_speakremove.clone(),
            client_speakremove.clone(),
            "Nickname of removed speaker: ",
            "-v".to_string(),
        );
    });

    let channel_limit = channel_name.clone();
    let client_limit = client.clone();
    limit_button.connect_clicked(move |_| {
        handle_mode_input(
            channel_limit.clone(),
            client_limit.clone(),
            "Limit: ",
            "+l".to_string(),
        )
    });
    let channel_limitremove = channel_name.clone();
    let client_limitremove = client.clone();
    remove_limit_button.connect_clicked(move |_| {
        let message = format!("MODE {} -l", channel_limitremove);
        client_limitremove
            .send(message)
            .expect("No se pudo enviar el mensaje");
    });

    let channel_ban = channel_name.clone();
    let client_ban = client.clone();
    ban_button.connect_clicked(move |_| {
        handle_mode_input(
            channel_ban.clone(),
            client_ban.clone(),
            "Ban mask: ",
            "+b".to_string(),
        )
    });
    let channel_banremove = channel_name.clone();
    let client_banremove = client.clone();
    remove_ban_button.connect_clicked(move |_| {
        handle_mode_input(
            channel_banremove.clone(),
            client_banremove.clone(),
            "Ban mask: ",
            "-b".to_string(),
        )
    });

    let channel_seeban = channel_name.clone();
    let client_seeban = client.clone();
    see_ban_button.connect_clicked(move |_| {
        let message = format!("MODE {} +b", channel_seeban);
        client_seeban
            .send(message)
            .expect("No se pudo enviar el mensaje");
    });

    let channel_kick = channel_name;
    let client_kick = client;
    kick_button.connect_clicked(move |_| {
        handle_kick_input(
            channel_kick.clone(),
            client_kick.clone(),
            "Member to kick: ",
        )
    });
}

pub fn handle_mode_input(
    channel_name: String,
    client: Arc<Client>,
    input_label: &str,
    mode: String,
) {
    let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let window = Arc::new(gtk::Window::new(gtk::WindowType::Toplevel));

    let input_label = gtk::Label::new(Some(input_label));
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
        let message = format!("MODE {} {} {}", channel_name, mode.clone(), input);
        client.send(message).expect("No se pudo enviar el mensaje");

        window.close();
    });
}
pub fn handle_kick_input(channel_name: String, client: Arc<Client>, input_label: &str) {
    let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let window = Arc::new(gtk::Window::new(gtk::WindowType::Toplevel));

    let input_label = gtk::Label::new(Some(input_label));
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
        let message = format!("KICK {} {}", channel_name, input);
        client.send(message).expect("No se pudo enviar el mensaje");

        window.close();
    });
}
