use carapax::types::{Integer, User};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fmt, sync::Arc};
use tokio_postgres::{Client, Error as ClientError, Row};

const ITEMS_PER_PAGE: i64 = 5;

#[derive(Clone)]
pub struct UserService {
    client: Arc<Client>,
}

impl UserService {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    pub async fn get_list(
        &self,
        page_number: i64,
        block_filter: UserBlockFilter,
    ) -> Result<UserInfoList, UserServiceError> {
        let total_items = self.count(block_filter).await?;
        let offset = (page_number * ITEMS_PER_PAGE - ITEMS_PER_PAGE).abs();
        let items = self
            .client
            .query(
                &format!(
                    "SELECT * FROM users {} ORDER BY created_at DESC LIMIT $1 OFFSET $2",
                    block_filter.as_sql()
                ),
                &[&ITEMS_PER_PAGE, &offset],
            )
            .await
            .map_err(|source| UserServiceError::GetList { source, page_number })?
            .into_iter()
            .map(UserInfo::from)
            .collect();
        Ok(UserInfoList::new(items, page_number, total_items, block_filter))
    }

    pub async fn save(&self, user: User) -> Result<(), UserServiceError> {
        if self.is_exists(user.id).await? {
            self.update(user).await?
        } else {
            self.create(user).await?
        }
        Ok(())
    }

    pub async fn block(&self, user_id: Integer) -> Result<bool, UserServiceError> {
        self.set_block(user_id, true).await
    }

    pub async fn unblock(&self, user_id: Integer) -> Result<bool, UserServiceError> {
        self.set_block(user_id, false).await
    }

    async fn count(&self, block_filter: UserBlockFilter) -> Result<i64, UserServiceError> {
        let row = self
            .client
            .query_one(&format!("SELECT COUNT(*) FROM users {}", block_filter.as_sql()), &[])
            .await
            .map_err(|source| UserServiceError::Count { source })?;
        Ok(row.get(0))
    }

    async fn is_exists(&self, user_id: Integer) -> Result<bool, UserServiceError> {
        let row = self
            .client
            .query_one("SELECT COUNT(*) FROM users WHERE id = $1", &[&user_id])
            .await
            .map_err(|source| UserServiceError::CheckExists { source, user_id })?;
        let count: i64 = row.get(0);
        Ok(count > 0)
    }

    async fn create(&self, user: User) -> Result<(), UserServiceError> {
        self.client
            .execute(
                "INSERT INTO users (id, first_name, last_name, username, created_at) VALUES ($1, $2, $3, $4, $5)",
                &[
                    &user.id,
                    &user.first_name,
                    &user.last_name,
                    &user.username,
                    &Utc::now().naive_utc(),
                ],
            )
            .await
            .map_err(|source| UserServiceError::CreateUser { source, user })?;
        Ok(())
    }

    async fn update(&self, user: User) -> Result<(), UserServiceError> {
        self.client
            .execute(
                "UPDATE users SET first_name = $1, last_name = $2, username = $3, updated_at = $4 WHERE id = $5",
                &[
                    &user.first_name,
                    &user.last_name,
                    &user.username,
                    &Utc::now().naive_utc(),
                    &user.id,
                ],
            )
            .await
            .map_err(|source| UserServiceError::UpdateUser { source, user })?;
        Ok(())
    }

    async fn set_block(&self, user_id: Integer, value: bool) -> Result<bool, UserServiceError> {
        let affected_rows = self
            .client
            .execute("UPDATE users SET is_blocked = $1 WHERE id = $2", &[&value, &user_id])
            .await
            .map_err(|source| UserServiceError::SetBlock { source, user_id, value })?;
        Ok(affected_rows != 0)
    }
}

#[derive(Clone, Debug)]
pub struct UserInfoList {
    items: Vec<UserInfo>,
    page_number: i64,
    total_items: i64,
    block_filter: UserBlockFilter,
}

impl UserInfoList {
    fn new(items: Vec<UserInfo>, page_number: i64, total_items: i64, block_filter: UserBlockFilter) -> Self {
        Self {
            items,
            page_number,
            total_items,
            block_filter,
        }
    }

    pub fn page_number(&self) -> i64 {
        self.page_number
    }

    pub fn total_pages(&self) -> i64 {
        (self.total_items as f64 / ITEMS_PER_PAGE as f64).ceil() as i64
    }

    pub fn total_items(&self) -> i64 {
        self.total_items
    }

    pub fn block_filter(&self) -> UserBlockFilter {
        self.block_filter
    }
}

impl fmt::Display for UserInfoList {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        self.items.iter().try_for_each(|item| writeln!(out, "{}", item))
    }
}

#[derive(Clone, Debug)]
struct UserInfo {
    id: Integer,
    first_name: String,
    last_name: Option<String>,
    username: Option<String>,
    created_at: NaiveDateTime,
    updated_at: Option<NaiveDateTime>,
    is_blocked: bool,
}

impl fmt::Display for UserInfo {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        write!(out, "<code>{}</code> ", self.id)?;
        let mut name = self.first_name.clone();
        if let Some(ref last_name) = self.last_name {
            name = format!("{} {}", name, last_name);
        }
        write!(out, r#"<a href="tg://user?id={id}">{name}</a>"#, id = self.id)?;
        if let Some(ref username) = self.username {
            write!(out, " (@{username})")?;
        }
        write!(out, " {}", self.created_at.format("%d/%m/%y %H:%M:%S"))?;
        if let Some(updated_at) = self.updated_at {
            write!(out, " {}", updated_at.format("%d/%m/%y %H:%M:%S"))?;
        }
        if self.is_blocked {
            write!(out, " ❌")?;
        }
        Ok(())
    }
}

impl From<Row> for UserInfo {
    fn from(row: Row) -> UserInfo {
        let indexes: HashMap<&str, usize> = row
            .columns()
            .iter()
            .enumerate()
            .map(|(idx, column)| (column.name(), idx))
            .collect();
        UserInfo {
            id: row.get(indexes["id"]),
            first_name: row.get(indexes["first_name"]),
            last_name: row.get(indexes["last_name"]),
            username: row.get(indexes["username"]),
            created_at: row.get(indexes["created_at"]),
            updated_at: row.get(indexes["updated_at"]),
            is_blocked: row.get(indexes["is_blocked"]),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum UserBlockFilter {
    All,
    False,
    True,
}

impl UserBlockFilter {
    fn as_sql(&self) -> &'static str {
        use self::UserBlockFilter::*;
        match self {
            All => "",
            False => "WHERE is_blocked IS FALSE",
            True => "WHERE is_blocked IS TRUE",
        }
    }
}

impl TryFrom<Option<&String>> for UserBlockFilter {
    type Error = UserBlockFilterError;

    fn try_from(value: Option<&String>) -> Result<Self, Self::Error> {
        Ok(match value.map(String::as_str) {
            Some("blocked") => Self::True,
            Some("!blocked") => Self::False,
            Some(value) => return Err(UserBlockFilterError(value.to_string())),
            None => Self::All,
        })
    }
}

#[derive(Debug)]
pub struct UserBlockFilterError(String);

impl fmt::Display for UserBlockFilterError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        write!(out, "unknown filter value: {}", self.0)
    }
}

impl Error for UserBlockFilterError {}

#[derive(Debug)]
pub enum UserServiceError {
    CheckExists {
        source: ClientError,
        user_id: Integer,
    },
    Count {
        source: ClientError,
    },
    CreateUser {
        source: ClientError,
        user: User,
    },
    GetList {
        source: ClientError,
        page_number: i64,
    },
    SetBlock {
        source: ClientError,
        user_id: Integer,
        value: bool,
    },
    UpdateUser {
        source: ClientError,
        user: User,
    },
}

impl fmt::Display for UserServiceError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::UserServiceError::*;
        match self {
            CheckExists { source, user_id } => {
                write!(
                    out,
                    "Could not check whether user with id {} exists: {}",
                    user_id, source
                )
            }
            Count { source } => write!(out, "Could not count users: {}", source),
            CreateUser { source, user } => {
                write!(out, "Could not create a user: {} (user={:?})", source, user)
            }
            GetList { source, page_number } => write!(
                out,
                "Could not get a list of users: {} (page_number={})",
                source, page_number
            ),
            SetBlock { source, user_id, value } => write!(
                out,
                "Could not set block to {} for user with id {}: {}",
                value, user_id, source
            ),
            UpdateUser { source, user } => {
                write!(out, "Could not create a user: {} (user={:?})", source, user)
            }
        }
    }
}

impl Error for UserServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::UserServiceError::*;
        Some(match self {
            CheckExists { source, .. } => source,
            Count { source, .. } => source,
            CreateUser { source, .. } => source,
            GetList { source, .. } => source,
            SetBlock { source, .. } => source,
            UpdateUser { source, .. } => source,
        })
    }
}
