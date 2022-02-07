use carapax::{
    types::{ChatId, Command, Message},
    Chain, CommandExt,
};

pub fn setup() -> Chain {
    Chain::default()
        .add(handle_users.command("/users"))
        .add(handle_ban.command("/ban"))
        .add(handle_unban.command("/unban"))
        .add(handle_message)
}

async fn handle_users(chat_id: ChatId, command: Command) {
    log::info!("Got /users command from admin: {} {:?}", chat_id, command);
}

async fn handle_ban(chat_id: ChatId, command: Command) {
    log::info!("Got /ban command from admin: {} {:?}", chat_id, command);
}

async fn handle_unban(chat_id: ChatId, command: Command) {
    log::info!("Got /unban command from admin: {} {:?}", chat_id, command);
}

async fn handle_message(chat_id: ChatId, message: Message) {
    log::info!("Got a message from admin: {} {:?}", chat_id, message);
}
