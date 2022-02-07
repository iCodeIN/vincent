use carapax::{
    types::{ChatId, Command, Message},
    Chain, CommandExt,
};

pub fn setup() -> Chain {
    Chain::default().add(handle_start.command("/start")).add(handle_message)
}

async fn handle_start(chat_id: ChatId, command: Command) {
    log::info!("Got /start command from subscriber: {} {:?}", chat_id, command);
}

async fn handle_message(chat_id: ChatId, message: Message) {
    log::info!("Got a message from subscriber: {} {:?}", chat_id, message);
}
