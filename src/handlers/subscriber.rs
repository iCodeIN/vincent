use crate::{
    config::Config,
    services::{MessageLink, MessageLinkDirection, MessageLinkService, MessageLinkServiceError},
};
use carapax::{
    methods::CopyMessage,
    types::{ChatId, Command, InlineKeyboardButton, Message},
    Api, Chain, CommandExt, ExecuteError, Ref,
};
use std::{error::Error, fmt};

pub fn setup() -> Chain {
    Chain::once().add(handle_start.command("/start")).add(handle_message)
}

async fn handle_start(chat_id: ChatId, command: Command) {
    log::info!("Got /start command from subscriber: {} {:?}", chat_id, command);
}

async fn handle_message(
    api: Ref<Api>,
    message_link_service: Ref<MessageLinkService>,
    config: Ref<Config>,
    subscriber_message: Message,
) -> Result<(), SubscriberError> {
    let admin_chat_id = config.chat_id;
    let subscriber_chat_id = subscriber_message.get_chat_id();

    let mut method = CopyMessage::new(admin_chat_id, subscriber_chat_id, subscriber_message.id);
    if let Some(user) = subscriber_message.get_user() {
        let name = user.get_full_name();
        let url = match user.username {
            Some(ref username) => format!("t.me/{}", username),
            None => user.get_link(),
        };
        method = method.reply_markup(vec![vec![InlineKeyboardButton::with_url(name, url)]])
    }
    if let Some(reply_to) = subscriber_message.reply_to {
        if let Some(link) = message_link_service
            .find(reply_to.get_chat_id(), reply_to.id, MessageLinkDirection::Subscriber)
            .await
            .map_err(SubscriberError::FindLink)?
            .filter(|link| link.admin_chat_id() == admin_chat_id)
        {
            method = method.reply_to_message_id(link.admin_message_id());
        }
    }

    let admin_message_id = api
        .execute(method)
        .await
        .map_err(SubscriberError::CopyMessage)?
        .message_id;

    message_link_service
        .create(MessageLink::new(
            subscriber_chat_id,
            subscriber_message.id,
            admin_chat_id,
            admin_message_id,
        ))
        .await
        .map_err(SubscriberError::CreateLink)?;

    Ok(())
}

#[derive(Debug)]
enum SubscriberError {
    CopyMessage(ExecuteError),
    CreateLink(MessageLinkServiceError),
    FindLink(MessageLinkServiceError),
}

impl fmt::Display for SubscriberError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::SubscriberError::*;
        match self {
            CopyMessage(err) => write!(out, "Could not copy message: {}", err),
            CreateLink(err) => err.fmt(out),
            FindLink(err) => err.fmt(out),
        }
    }
}

impl Error for SubscriberError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::SubscriberError::*;
        Some(match self {
            CopyMessage(err) => err,
            CreateLink(err) => err,
            FindLink(err) => err,
        })
    }
}
