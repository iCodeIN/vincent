use crate::services::{
    MessageLinkDirection, MessageLinkService, MessageLinkServiceError, UserService, UserServiceError,
};
use carapax::{
    methods::SendMessage,
    types::{ChatId, Message},
    Api, ExecuteError, Ref,
};
use futures_util::future::OptionFuture;
use std::{error::Error, fmt};

const MESSAGE_OK: &str = "OK";
const MESSAGE_NOT_FOUND: &str = "Not found";

pub async fn handle(
    api: Ref<Api>,
    message_link_service: Ref<MessageLinkService>,
    user_service: Ref<UserService>,
    chat_id: ChatId,
    message: Message,
) -> Result<(), BlockError> {
    let link =
        OptionFuture::from(message.reply_to.map(|reply_to| {
            message_link_service.find(reply_to.get_chat_id(), reply_to.id, MessageLinkDirection::Admin)
        }))
        .await
        .transpose()
        .map_err(BlockError::GetLink)?
        .flatten();
    let text = match link {
        Some(link) => {
            if user_service
                .block(link.subscriber_user_id())
                .await
                .map_err(BlockError::SetBlock)?
            {
                MESSAGE_OK
            } else {
                MESSAGE_NOT_FOUND
            }
        }
        None => MESSAGE_NOT_FOUND,
    };
    api.execute(SendMessage::new(chat_id, text).reply_to_message_id(message.id))
        .await
        .map_err(BlockError::SendMessage)?;
    Ok(())
}

#[derive(Debug)]
pub enum BlockError {
    GetLink(MessageLinkServiceError),
    SendMessage(ExecuteError),
    SetBlock(UserServiceError),
}

impl fmt::Display for BlockError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::BlockError::*;
        match self {
            GetLink(err) => err.fmt(out),
            SendMessage(err) => err.fmt(out),
            SetBlock(err) => err.fmt(out),
        }
    }
}

impl Error for BlockError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::BlockError::*;
        Some(match self {
            GetLink(err) => err,
            SendMessage(err) => err,
            SetBlock(err) => err,
        })
    }
}
