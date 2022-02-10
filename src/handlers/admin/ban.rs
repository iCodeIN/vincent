use carapax::types::{ChatId, Command};

pub async fn handle(chat_id: ChatId, command: Command) {
    log::info!("Got /ban command from admin: {} {:?}", chat_id, command);
}
