use crate::services::{UserService, UserServiceError};
use carapax::{
    methods::SendMessage,
    types::{ChatId, Command, Integer},
    Api, ExecuteError, Ref,
};
use std::{error::Error, fmt};

const MESSAGE_OK: &str = "OK";
const MESSAGE_NOT_FOUND: &str = "Not found";

pub async fn handle(
    api: Ref<Api>,
    user_service: Ref<UserService>,
    chat_id: ChatId,
    command: Command,
) -> Result<(), UnblockError> {
    let message_id = command.get_message().id;
    let user_id = match command.get_args().first().map(|arg| arg.parse::<Integer>()) {
        Some(Ok(value)) => value,
        Some(Err(_)) => {
            api.execute(SendMessage::new(chat_id, "Invalid User ID").reply_to_message_id(message_id))
                .await
                .map_err(UnblockError::SendMessage)?;
            return Ok(());
        }
        None => {
            api.execute(SendMessage::new(chat_id, "User ID is required").reply_to_message_id(message_id))
                .await
                .map_err(UnblockError::SendMessage)?;
            return Ok(());
        }
    };
    let text = if user_service.unblock(user_id).await.map_err(UnblockError::SetBlock)? {
        MESSAGE_OK
    } else {
        MESSAGE_NOT_FOUND
    };
    api.execute(SendMessage::new(chat_id, text).reply_to_message_id(message_id))
        .await
        .map_err(UnblockError::SendMessage)?;
    Ok(())
}

#[derive(Debug)]
pub enum UnblockError {
    SetBlock(UserServiceError),
    SendMessage(ExecuteError),
}

impl fmt::Display for UnblockError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::UnblockError::*;
        match self {
            SetBlock(err) => err.fmt(out),
            SendMessage(err) => write!(out, "could not send message: {}", err),
        }
    }
}

impl Error for UnblockError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::UnblockError::*;
        Some(match self {
            SetBlock(err) => err,
            SendMessage(err) => err,
        })
    }
}
