use carapax::types::{Integer, User};
use chrono::Utc;
use std::{error::Error, fmt, sync::Arc};
use tokio_postgres::{Client, Error as ClientError};

#[derive(Clone)]
pub struct UserService {
    client: Arc<Client>,
}

impl UserService {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    pub async fn save(&self, user: User) -> Result<(), UserServiceError> {
        if self.is_exists(user.id).await? {
            self.update(user).await?
        } else {
            self.create(user).await?
        }
        Ok(())
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
}

#[derive(Debug)]
pub enum UserServiceError {
    CheckExists { source: ClientError, user_id: Integer },
    CreateUser { source: ClientError, user: User },
    UpdateUser { source: ClientError, user: User },
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
            CreateUser { source, user } => {
                write!(out, "Could not create a user: {} (user={:?})", source, user)
            }
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
            CreateUser { source, .. } => source,
            UpdateUser { source, .. } => source,
        })
    }
}
