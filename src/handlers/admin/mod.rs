use carapax::{Chain, CommandExt};

mod ban;
mod message;
mod unban;
mod users;

pub fn setup() -> Chain {
    Chain::once()
        .add(users::handle_list.command("/users"))
        .add(users::handle_page_changed)
        .add(ban::handle.command("/ban"))
        .add(unban::handle.command("/unban"))
        .add(message::handle)
}
