use crate::migrations::types;
use barrel::{backend::Pg, Migration};

macro_rules! version {
    ($builder:ident) => {
        Version::new(stringify!($builder), $builder)
    };
}

pub fn build() -> Vec<Version> {
    vec![version!(create_users), version!(create_message_links)]
}

pub struct Version {
    name: String,
    builder: Builder,
}

impl Version {
    fn new<N>(name: N, builder: Builder) -> Self
    where
        N: Into<String>,
    {
        Self {
            name: name.into(),
            builder,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn build(&self) -> String {
        let builder = self.builder;
        let migration = builder();
        migration.make::<Pg>()
    }
}

type Builder = fn() -> Migration;

fn create_users() -> Migration {
    let mut migration = Migration::new();
    migration.create_table("users", |table| {
        table.add_column("id", types::bigint().primary(true));
        table.add_column("first_name", types::varchar(255));
        table.add_column("last_name", types::varchar(255).nullable(true));
        table.add_column("username", types::varchar(255).nullable(true));
        table.add_column("created_at", types::utc_timestamp());
        table.add_column("updated_at", types::utc_timestamp().nullable(true));
        table.add_column("is_blocked", types::boolean().default(false));
    });
    migration
}

fn create_message_links() -> Migration {
    let mut migration = Migration::new();
    migration.create_table("message_links", |table| {
        table.add_column("id", types::primary());
        table.add_column("subscriber_chat_id", types::bigint());
        table.add_column("subscriber_message_id", types::bigint());
        table.add_column("admin_chat_id", types::bigint());
        table.add_column("admin_message_id", types::bigint());
        table.add_index(
            "message_links_subscriber_idx",
            types::index(&["subscriber_chat_id", "subscriber_message_id"]),
        );
        table.add_index(
            "message_links_admin_idx",
            types::index(&["admin_chat_id", "admin_message_id"]),
        );
    });
    migration
}
