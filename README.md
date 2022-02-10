# Vincent

A feedback bot for Telegram

## Installation

Make sure that you have installed PostgreSQL.

Download binary:

```sh
$ curl -L https://github.com/rossnomann/vincent/releases/download/0.1.0/vincent-0.1.0_x86_64-linux-gnu --output vincent
$ chmod +x vincent
```

Create `config.yaml`:

```yaml
token: 'bottoken'  # Token from BotFather
chat_id: -1000000000000  # ID of admin chat
database_url: postgresql://user:password@localhost:5432/database  # PostgreSQL connection
greeting: '<b>HI!!!</b>'  # Welcome message for subscribers
```

See https://core.telegram.org/bots/api#html-style for more information about `greeting` format.

If you want to change log level, use [`RUST_LOG`](https://docs.rs/env_logger/0.9.0/env_logger/) environment variable.

Run migrations:

```sh
$ ./vincent config.yaml migrate
```

Start bot:

```sh
$ ./vincent config.yaml start
````

# Changelog

## 0.1.0 (10.02.2022)

- First release.

# LICENSE

The MIT License (MIT)
