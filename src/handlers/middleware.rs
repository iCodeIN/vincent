use crate::services::{UserService, UserServiceError};
use carapax::{types::User, Chain, Ref};

pub fn setup() -> Chain {
    Chain::all().add(track_user)
}

async fn track_user(user_service: Ref<UserService>, user: User) -> Result<(), UserServiceError> {
    user_service.save(user).await
}
