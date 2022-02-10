use carapax::{Chain, CommandExt};

mod block;
mod message;
mod unblock;
mod users;

pub fn setup() -> Chain {
    Chain::once()
        .add(users::handle_list.command("/users"))
        .add(users::handle_page_changed)
        .add(block::handle.command("/block"))
        .add(unblock::handle.command("/unblock"))
        .add(message::handle)
}
