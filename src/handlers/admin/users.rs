use crate::services::{UserInfoList, UserService, UserServiceError};
use carapax::{
    methods::{AnswerCallbackQuery, EditMessageText, SendMessage},
    types::{CallbackQuery, CallbackQueryError, ChatId, InlineKeyboardButton, InlineKeyboardError, Integer, ParseMode},
    Api, ExecuteError, Ref, TryFromInput,
};
use futures_util::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

pub async fn handle_list(api: Ref<Api>, user_service: Ref<UserService>, chat_id: ChatId) -> Result<(), UsersError> {
    let users = user_service.get_list(1).await.map_err(UsersError::Get)?;
    let keyboard = build_keyboard(&users).map_err(UsersError::BuildKeyboard)?;
    api.execute(
        SendMessage::new(chat_id, users.to_string())
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard),
    )
    .await
    .map_err(UsersError::ExecuteSend)?;
    Ok(())
}

pub async fn handle_page_changed(
    api: Ref<Api>,
    user_service: Ref<UserService>,
    query: PageQuery,
) -> Result<(), UsersError> {
    let users = user_service.get_list(query.number).await.map_err(UsersError::Get)?;
    let keyboard = build_keyboard(&users).map_err(UsersError::BuildKeyboard)?;
    api.execute(
        EditMessageText::new(query.chat_id, query.message_id, users.to_string())
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard),
    )
    .await
    .map_err(UsersError::ExecuteSend)?;
    api.execute(AnswerCallbackQuery::new(query.id))
        .await
        .map_err(UsersError::ExecuteAnswer)?;
    Ok(())
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

#[derive(Serialize, Deserialize)]
struct Page {
    number: i64,
}

pub struct PageQuery {
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
pub enum PageQueryError {
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

#[derive(Debug)]
pub enum UsersError {
    BuildKeyboard(InlineKeyboardError),
    ExecuteAnswer(ExecuteError),
    ExecuteSend(ExecuteError),
    Get(UserServiceError),
}

impl fmt::Display for UsersError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::UsersError::*;
        match self {
            BuildKeyboard(err) => write!(out, "could not build inline keyboard: {}", err),
            ExecuteAnswer(err) => write!(out, "could not answer to a callback query: {}", err),
            ExecuteSend(err) => write!(out, "could not send users list: {}", err),
            Get(err) => err.fmt(out),
        }
    }
}

impl Error for UsersError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::UsersError::*;
        Some(match self {
            BuildKeyboard(err) => err,
            ExecuteAnswer(err) => err,
            ExecuteSend(err) => err,
            Get(err) => err,
        })
    }
}
