use crate::services::{UserService, UserServiceError};
use carapax::{access::AccessPolicy, types::Integer, HandlerInput};
use futures_util::future::{BoxFuture, OptionFuture};

#[derive(Clone)]
pub struct SubscriberAccessPolicy {
    user_service: UserService,
    admin_chat_id: Integer,
}

impl SubscriberAccessPolicy {
    pub fn new(user_service: UserService, admin_chat_id: Integer) -> Self {
        Self {
            user_service,
            admin_chat_id,
        }
    }
}

impl AccessPolicy for SubscriberAccessPolicy {
    type Error = UserServiceError;
    type Future = BoxFuture<'static, Result<bool, Self::Error>>;

    fn is_granted(&self, input: HandlerInput) -> Self::Future {
        let user_service = self.user_service.clone();
        let admin_chat_id = self.admin_chat_id;
        Box::pin(async move {
            Ok(
                if input
                    .update
                    .get_chat_id()
                    .map(|chat_id| chat_id == admin_chat_id)
                    .unwrap_or(false)
                {
                    // admins has no access to subscriber handlers
                    false
                } else {
                    OptionFuture::from(
                        input
                            .update
                            .get_user_id()
                            .map(|user_id| user_service.is_blocked(user_id)),
                    )
                    .await
                    .transpose()?
                    .map(|is_blocked| !is_blocked)
                    .unwrap_or(true) // allow for all other users by default
                },
            )
        })
    }
}
