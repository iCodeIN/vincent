use crate::services::{UserBlockFilter, UserInfoList, UserService, UserServiceError};
use carapax::{
    methods::{AnswerCallbackQuery, EditMessageText, SendMessage},
    types::{
        CallbackQuery, CallbackQueryError, ChatId, Command, InlineKeyboardButton, InlineKeyboardError, Integer,
        ParseMode,
    },
    Api, ExecuteError, HandlerInput, Ref, TryFromInput,
};
use futures_util::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

pub async fn handle_list(
    api: Ref<Api>,
    user_service: Ref<UserService>,
    chat_id: ChatId,
    command: Command,
) -> Result<(), UsersError> {
    let block_filter: UserBlockFilter = match command.get_args().first().try_into() {
        Ok(value) => value,
        Err(err) => {
            api.execute(SendMessage::new(chat_id, err.to_string()))
                .await
                .map_err(UsersError::SendMessage)?;
            return Ok(());
        }
    };
    let users = user_service
        .get_list(1, block_filter)
        .await
        .map_err(UsersError::GetList)?;
    let keyboard = build_keyboard(&users).map_err(UsersError::BuildKeyboard)?;
    api.execute(
        SendMessage::new(chat_id, users.to_string())
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard),
    )
    .await
    .map_err(UsersError::SendMessage)?;
    Ok(())
}

pub async fn handle_page_changed(
    api: Ref<Api>,
    user_service: Ref<UserService>,
    query: PageQuery,
) -> Result<(), UsersError> {
    let users = user_service
        .get_list(query.number, query.block_filter)
        .await
        .map_err(UsersError::GetList)?;
    let keyboard = build_keyboard(&users).map_err(UsersError::BuildKeyboard)?;
    api.execute(
        EditMessageText::new(query.chat_id, query.message_id, users.to_string())
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard),
    )
    .await
    .map_err(UsersError::SendMessage)?;
    api.execute(AnswerCallbackQuery::new(query.id))
        .await
        .map_err(UsersError::AnswerCallbackQuery)?;
    Ok(())
}

fn build_keyboard(list: &UserInfoList) -> Result<Vec<Vec<InlineKeyboardButton>>, InlineKeyboardError> {
    let mut row = Vec::new();
    let current_page = list.page_number();
    let total_pages = list.total_pages();
    let total_items = list.total_items();
    let block_filter = list.block_filter();
    if current_page != 1 {
        row.push(InlineKeyboardButton::with_callback_data_struct(
            "<<",
            &Page {
                number: 1,
                block_filter,
            },
        )?)
    }
    if current_page > 2 {
        row.push(InlineKeyboardButton::with_callback_data_struct(
            "<",
            &Page {
                number: current_page - 1,
                block_filter,
            },
        )?);
    }
    row.push(InlineKeyboardButton::with_callback_data_struct(
        format!("{}/{} ({})", current_page, total_pages, total_items),
        &Page {
            number: current_page,
            block_filter,
        },
    )?);
    if current_page < total_pages - 1 {
        row.push(InlineKeyboardButton::with_callback_data_struct(
            ">",
            &Page {
                number: current_page + 1,
                block_filter,
            },
        )?);
    }
    if current_page < total_pages {
        row.push(InlineKeyboardButton::with_callback_data_struct(
            ">>",
            &Page {
                number: total_pages,
                block_filter,
            },
        )?)
    }
    Ok(vec![row])
}

#[derive(Serialize, Deserialize)]
struct Page {
    number: i64,
    block_filter: UserBlockFilter,
}

pub struct PageQuery {
    id: String,
    chat_id: Integer,
    message_id: Integer,
    number: i64,
    block_filter: UserBlockFilter,
}

impl TryFrom<CallbackQuery> for PageQuery {
    type Error = PageQueryError;

    fn try_from(query: CallbackQuery) -> Result<Self, Self::Error> {
        let Page { number, block_filter } = query
            .parse_data()
            .map_err(PageQueryError::ParseData)
            .and_then(|page: Option<Page>| page.ok_or(PageQueryError::NoData))?;
        let message = query.message.ok_or(PageQueryError::NoMessage)?;
        Ok(Self {
            id: query.id,
            chat_id: message.get_chat_id(),
            message_id: message.id,
            number,
            block_filter,
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

    fn try_from_input(input: HandlerInput) -> Self::Future {
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
    AnswerCallbackQuery(ExecuteError),
    BuildKeyboard(InlineKeyboardError),
    GetList(UserServiceError),
    SendMessage(ExecuteError),
}

impl fmt::Display for UsersError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::UsersError::*;
        match self {
            AnswerCallbackQuery(err) => err.fmt(out),
            BuildKeyboard(err) => write!(out, "could not build inline keyboard: {}", err),
            GetList(err) => err.fmt(out),
            SendMessage(err) => err.fmt(out),
        }
    }
}

impl Error for UsersError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::UsersError::*;
        Some(match self {
            AnswerCallbackQuery(err) => err,
            BuildKeyboard(err) => err,
            GetList(err) => err,
            SendMessage(err) => err,
        })
    }
}
