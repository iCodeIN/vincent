use crate::services::{MessageLink, MessageLinkDirection, MessageLinkService, MessageLinkServiceError};
use carapax::{
    methods::CopyMessage,
    types::{ChatId, Command, Message},
    Api, Chain, CommandExt, ExecuteError, Ref,
};

use std::{error::Error, fmt};

pub fn setup() -> Chain {
    Chain::once()
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

async fn handle_message(
    api: Ref<Api>,
    message_link_service: Ref<MessageLinkService>,
    message: Message,
) -> Result<(), AdminError> {
    if let Some(reply_to) = message.reply_to {
        let link = message_link_service
            .find(reply_to.get_chat_id(), reply_to.id, MessageLinkDirection::Admin)
            .await
            .map_err(AdminError::FindLink)?;
        if let Some(link) = link {
            let subscriber_chat_id = link.subscriber_chat_id();
            let admin_chat_id = link.admin_chat_id();
            let subscriber_message_id = api
                .execute(
                    CopyMessage::new(subscriber_chat_id, admin_chat_id, message.id)
                        .reply_to_message_id(link.subscriber_message_id()),
                )
                .await
                .map_err(AdminError::CopyMessage)?
                .message_id;
            message_link_service
                .create(MessageLink::new(
                    subscriber_chat_id,
                    subscriber_message_id,
                    admin_chat_id,
                    message.id,
                ))
                .await
                .map_err(AdminError::CreateLink)?;
        }
    }
    Ok(())
}

#[derive(Debug)]
enum AdminError {
    CopyMessage(ExecuteError),
    CreateLink(MessageLinkServiceError),
    FindLink(MessageLinkServiceError),
}

impl fmt::Display for AdminError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::AdminError::*;
        match self {
            CopyMessage(err) => write!(out, "Could not copy message: {}", err),
            CreateLink(err) => err.fmt(out),
            FindLink(err) => err.fmt(out),
        }
    }
}

impl Error for AdminError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::AdminError::*;
        Some(match self {
            CopyMessage(err) => err,
            CreateLink(err) => err,
            FindLink(err) => err,
        })
    }
}
