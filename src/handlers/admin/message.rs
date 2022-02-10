use crate::services::{MessageLink, MessageLinkDirection, MessageLinkService, MessageLinkServiceError};
use carapax::{methods::CopyMessage, types::Message, Api, ExecuteError, Ref};
use std::{error::Error, fmt};

pub async fn handle(
    api: Ref<Api>,
    message_link_service: Ref<MessageLinkService>,
    message: Message,
) -> Result<(), MessageError> {
    if let Some(reply_to) = message.reply_to {
        let link = message_link_service
            .find(reply_to.get_chat_id(), reply_to.id, MessageLinkDirection::Admin)
            .await
            .map_err(MessageError::FindLink)?;
        if let Some(link) = link {
            let subscriber_chat_id = link.subscriber_chat_id();
            let admin_chat_id = link.admin_chat_id();
            let subscriber_message_id = api
                .execute(
                    CopyMessage::new(subscriber_chat_id, admin_chat_id, message.id)
                        .reply_to_message_id(link.subscriber_message_id()),
                )
                .await
                .map_err(MessageError::ExecuteCopy)?
                .message_id;
            message_link_service
                .create(MessageLink::new(
                    subscriber_chat_id,
                    subscriber_message_id,
                    admin_chat_id,
                    message.id,
                ))
                .await
                .map_err(MessageError::CreateLink)?;
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum MessageError {
    CreateLink(MessageLinkServiceError),
    ExecuteCopy(ExecuteError),
    FindLink(MessageLinkServiceError),
}

impl fmt::Display for MessageError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::MessageError::*;
        match self {
            CreateLink(err) => err.fmt(out),
            ExecuteCopy(err) => write!(out, "could not copy message: {}", err),
            FindLink(err) => err.fmt(out),
        }
    }
}

impl Error for MessageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::MessageError::*;
        Some(match self {
            CreateLink(err) => err,
            ExecuteCopy(err) => err,
            FindLink(err) => err,
        })
    }
}
