use carapax::types::Integer;
use std::{collections::HashMap, error::Error, fmt, sync::Arc};
use tokio_postgres::{Client, Error as ClientError, Row};

#[derive(Clone)]
pub struct MessageLinkService {
    client: Arc<Client>,
}

impl MessageLinkService {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    pub async fn create(&self, link: MessageLink) -> Result<(), MessageLinkServiceError> {
        self.client
            .execute(
                r#"
                INSERT INTO message_links
                    (subscriber_user_id, subscriber_chat_id, subscriber_message_id, admin_chat_id, admin_message_id)
                VALUES
                    ($1, $2, $3, $4, $5)
                "#,
                &[
                    &link.subscriber_user_id(),
                    &link.subscriber_chat_id(),
                    &link.subscriber_message_id(),
                    &link.admin_chat_id(),
                    &link.admin_message_id(),
                ],
            )
            .await
            .map_err(|source| MessageLinkServiceError::Create { source, link })?;
        Ok(())
    }

    pub async fn find(
        &self,
        chat_id: Integer,
        message_id: Integer,
        direction: MessageLinkDirection,
    ) -> Result<Option<MessageLink>, MessageLinkServiceError> {
        let row = self
            .client
            .query_opt(
                match direction {
                    MessageLinkDirection::Admin => {
                        "SELECT * FROM message_links WHERE admin_chat_id = $1 AND admin_message_id = $2"
                    }
                    MessageLinkDirection::Subscriber => {
                        "SELECT * FROM message_links WHERE subscriber_chat_id = $1 AND subscriber_message_id = $2"
                    }
                },
                &[&chat_id, &message_id],
            )
            .await
            .map_err(|source| MessageLinkServiceError::Find {
                source,
                chat_id,
                message_id,
                direction,
            })?;
        Ok(row.map(MessageLink::from))
    }
}

#[derive(Debug)]
pub struct MessageLink {
    subscriber_user_id: Integer,
    subscriber_chat_id: Integer,
    subscriber_message_id: Integer,
    admin_chat_id: Integer,
    admin_message_id: Integer,
}

impl MessageLink {
    pub fn new(
        subscriber_user_id: Integer,
        subscriber_chat_id: Integer,
        subscriber_message_id: Integer,
        admin_chat_id: Integer,
        admin_message_id: Integer,
    ) -> Self {
        Self {
            subscriber_user_id,
            subscriber_chat_id,
            subscriber_message_id,
            admin_chat_id,
            admin_message_id,
        }
    }

    pub fn subscriber_user_id(&self) -> Integer {
        self.subscriber_user_id
    }

    pub fn subscriber_chat_id(&self) -> Integer {
        self.subscriber_chat_id
    }

    pub fn subscriber_message_id(&self) -> Integer {
        self.subscriber_message_id
    }

    pub fn admin_chat_id(&self) -> Integer {
        self.admin_chat_id
    }

    pub fn admin_message_id(&self) -> Integer {
        self.admin_message_id
    }
}

impl From<Row> for MessageLink {
    fn from(row: Row) -> Self {
        let indexes: HashMap<&str, usize> = row
            .columns()
            .iter()
            .enumerate()
            .map(|(idx, column)| (column.name(), idx))
            .collect();
        MessageLink::new(
            row.get(indexes["subscriber_user_id"]),
            row.get(indexes["subscriber_chat_id"]),
            row.get(indexes["subscriber_message_id"]),
            row.get(indexes["admin_chat_id"]),
            row.get(indexes["admin_message_id"]),
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MessageLinkDirection {
    Admin,
    Subscriber,
}

impl fmt::Display for MessageLinkDirection {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::MessageLinkDirection::*;
        match self {
            Admin => write!(out, "admin"),
            Subscriber => write!(out, "subscriber"),
        }
    }
}

#[derive(Debug)]
pub enum MessageLinkServiceError {
    Create {
        source: ClientError,
        link: MessageLink,
    },
    Find {
        source: ClientError,
        chat_id: Integer,
        message_id: Integer,
        direction: MessageLinkDirection,
    },
}

impl fmt::Display for MessageLinkServiceError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::MessageLinkServiceError::*;
        match self {
            Create { source, link } => write!(out, "Could not create message link: {} ({:?})", source, link),
            Find {
                source,
                chat_id,
                message_id,
                direction,
            } => write!(
                out,
                "Could not find message link for {}: {} (chat_id={}, message_id={})",
                direction, source, chat_id, message_id
            ),
        }
    }
}

impl Error for MessageLinkServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::MessageLinkServiceError::*;
        Some(match self {
            Create { source, .. } => source,
            Find { source, .. } => source,
        })
    }
}
