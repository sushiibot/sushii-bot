# sushii bot

[![Build Status](https://travis-ci.org/drklee3/sushii-bot.svg?branch=master)](https://travis-ci.org/drklee3/sushii-bot)

![sushii](assets/sushii.png)

A [Discord](https://discordapp.com) bot written in [Rust](https://www.rust-lang.org/) with [serenity-rs](https://github.com/zeyla/serenity).  Uses a [PostgreSQL](https://www.postgresql.org) database along with [diesel-rs](https://github.com/diesel-rs/diesel) and [r2d2-diesel](https://github.com/diesel-rs/r2d2-diesel).

Work in progress.  Features may be either missing, incomplete, or broken.

# Features
* Ranking system based on message counts in daily, weekly, monthly, and all time categories
* User 24 hour activity tracker
* Profile image generation for displaying rank and activity graph (with [sushii-image-server](https://github.com/drklee3/sushii-image-server))
* Configurable self assigning role system with multiple categories and limits
* Configurable prefix per guild
* Moderation action logs and editable action reasons
* Mute evasion prevention
* Mass mention auto-mutes
* User created tags (custom commands-ish)
* Channel galleries (sends links & images from a channel to a webhook)
* Reminders
* Keyword notifications
* User join and leave messages
* Rust playground code execution
* Discord events counter
* ...and more to be added

# Installation
1. Download the latest version from the [releases](releases) page, currently only supporting x86_64-unknown-linux-gnu.
2. Allow the file to be executed.
    ```bash
    $ chmod +x x86_64-unknown-linux-gnu
    ```
3. Create an `.env` file in the same directory and update according to [`.env.example`](.env.example).  All variables must exist except for `BLOCKED_USERS` or the bot will panic.

4. Run with `./x86_64-unknown-linux-gnu` or with a process manager like [Supervisor](http://supervisord.org).

# Building from Source

1. Install dependencies.
    * [PostgreSQL](https://www.postgresql.org) (9.4+)
    * [Rust / Cargo](http://doc.crates.io)
        ```bash
        $ curl -sSf https://static.rust-lang.org/rustup.sh | sh
        ```
    * [sushii-image-server](https://github.com/drklee3/sushii-image-server) (Used for rank image generation, etc)
2. Clone this repository and enter the directory.
    ```bash
    $ git clone https://github.com/drklee3/sushii-bot.git
    $ cd sushii-bot
    ```
3. Edit [`.env.example`](.env.example) and rename to `.env`.  Removing any key or leaving them blank will result in panics.
4. Build and run the bot.
    ```bash
    $ cargo run --release
    ```
