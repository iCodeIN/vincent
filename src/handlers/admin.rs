use crate::services::{
    MessageLink, MessageLinkDirection, MessageLinkService, MessageLinkServiceError, UserInfoList, UserService,
    UserServiceError,
};
use carapax::{
    methods::{AnswerCallbackQuery, CopyMessage, EditMessageText, SendMessage},
    types::{
        CallbackQuery, CallbackQueryError, ChatId, Command, InlineKeyboardButton, InlineKeyboardError, Integer,
        Message, ParseMode,
    },
    Api, Chain, CommandExt, ExecuteError, Ref, TryFromInput,
};
use futures_util::future::BoxFuture;
use serde::{Deserialize, Serialize};

use std::{error::Error, fmt};

pub fn setup() -> Chain {
    Chain::once()
        .add(on_users_page_changed)
        .add(handle_users.command("/users"))
        .add(handle_ban.command("/ban"))
        .add(handle_unban.command("/unban"))
        .add(handle_message)
}

#[derive(Serialize, Deserialize)]
struct Page {
    number: i64,
}

struct PageQuery {
    id: String,
    chat_id: Integer,
    message_id: Integer,
    number: i64,
}

impl TryFrom<CallbackQuery> for PageQuery {
    type Error = PageQueryError;

    fn try_from(query: CallbackQuery) -> Result<Self, Self::Error> {
        let number = query
            .parse_data()
            .map_err(PageQueryError::ParseData)
            .and_then(|page: Option<Page>| page.ok_or(PageQueryError::NoData))?
            .number;
        let message = query.message.ok_or(PageQueryError::NoMessage)?;
        Ok(Self {
            id: query.id,
            chat_id: message.get_chat_id(),
            message_id: message.id,
            number,
        })
    }
}

#[derive(Debug)]
enum PageQueryError {
    NoData,
    NoMessage,
    ParseData(CallbackQueryError),
}

impl fmt::Display for PageQueryError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::PageQueryError::*;
        match self {
            NoData => write!(out, "callback query has no data"),
            NoMessage => write!(out, "callback query has no message"),
            ParseData(err) => write!(out, "could not parse query data: {}", err),
        }
    }
}

impl Error for PageQueryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::PageQueryError::*;
        match self {
            NoData => None,
            NoMessage => None,
            ParseData(err) => Some(err),
        }
    }
}

impl TryFromInput for PageQuery {
    type Error = PageQueryError;

    type Future = BoxFuture<'static, Result<Option<Self>, Self::Error>>;

    fn try_from_input(input: carapax::HandlerInput) -> Self::Future {
        Box::pin(async move {
            CallbackQuery::try_from_input(input)
                .await
                .ok()
                .flatten()
                .map(TryFrom::try_from)
                .transpose()
        })
    }
}

fn build_keyboard(list: &UserInfoList) -> Result<Vec<Vec<InlineKeyboardButton>>, InlineKeyboardError> {
    let mut row = Vec::new();
    let current_page = list.page_number();
    let total_pages = list.total_pages();
    let total_items = list.total_items();
    if current_page != 1 {
        row.push(InlineKeyboardButton::with_callback_data_struct(
            "<<",
            &Page { number: 1 },
        )?)
    }
    if current_page > 2 {
        row.push(InlineKeyboardButton::with_callback_data_struct(
            "<",
            &Page {
                number: current_page - 1,
            },
        )?);
    }
    row.push(InlineKeyboardButton::with_callback_data_struct(
        format!("{}/{} ({})", current_page, total_pages, total_items),
        &Page { number: current_page },
    )?);
    if current_page < total_pages - 1 {
        row.push(InlineKeyboardButton::with_callback_data_struct(
            ">",
            &Page {
                number: current_page + 1,
            },
        )?);
    }
    if current_page < total_pages {
        row.push(InlineKeyboardButton::with_callback_data_struct(
            ">>",
            &Page { number: total_pages },
        )?)
    }
    Ok(vec![row])
}

async fn on_users_page_changed(
    api: Ref<Api>,
    user_service: Ref<UserService>,
    query: PageQuery,
) -> Result<(), AdminError> {
    let users = user_service
        .get_list(query.number)
        .await
        .map_err(AdminError::GetUsers)?;
    let keyboard = build_keyboard(&users).unwrap();
    api.execute(
        EditMessageText::new(query.chat_id, query.message_id, users.to_string())
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard),
    )
    .await
    .map_err(AdminError::SendUsers)?;
    api.execute(AnswerCallbackQuery::new(query.id)).await.unwrap();
    Ok(())
}

async fn handle_users(api: Ref<Api>, user_service: Ref<UserService>, chat_id: ChatId) -> Result<(), AdminError> {
    let users = user_service.get_list(1).await.map_err(AdminError::GetUsers)?;
    let keyboard = build_keyboard(&users).unwrap();
    api.execute(
        SendMessage::new(chat_id, users.to_string())
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard),
    )
    .await
    .map_err(AdminError::SendUsers)?;
    Ok(())
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
    GetUsers(UserServiceError),
    SendUsers(ExecuteError),
}

impl fmt::Display for AdminError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::AdminError::*;
        match self {
            CopyMessage(err) => write!(out, "Could not copy message: {}", err),
            CreateLink(err) => err.fmt(out),
            FindLink(err) => err.fmt(out),
            GetUsers(err) => err.fmt(out),
            SendUsers(err) => write!(out, "Could not send a list of users: {}", err),
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
            GetUsers(err) => err,
            SendUsers(err) => err,
        })
    }
}
