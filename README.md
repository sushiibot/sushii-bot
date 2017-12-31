# discord-sbot2
[![Build Status](https://travis-ci.org/drklee3/discord-sbot2.svg?branch=master)](https://travis-ci.org/drklee3/discord-sbot2)

A [Discord](https://discordapp.com) bot written in [Rust](https://www.rust-lang.org/) with [serenity-rs](https://github.com/zeyla/serenity).

Uses a [PostgreSQL](https://www.postgresql.org) database along with [diesel-rs](https://github.com/diesel-rs/diesel) and [r2d2-diesel](https://github.com/diesel-rs/r2d2-diesel).

Work in progress.  Features may be either missing or broken.

# Current Features
* Ranking system based on percentiles of message counts in daily, weekly, monthly, and all time categories
* User 24 hour activity tracker
* Profile image generation for displaying rank and activity graph (with [sbot2-image-server](https://github.com/drklee3/sbot2-image-server))
* Customizable prefix per guild
* Reminders
* Keyword notifications
* Moderation action logs and editable action reasons
* User join and leave messages
* Rust playground code execution
* Discord events counter
* ...and more to be added

# Installation
Currently you will have to build everything yourself.  Later on, SQL migrations may be moved into binary downloads to simplify installation, removing the need for cloning this repository or installing Rust, Cargo, and Diesel CLI.

1. Install dependencies.
    * [PostgreSQL](https://www.postgresql.org)
    * [Diesel CLI](https://github.com/diesel-rs/diesel/tree/master/diesel_cli)
        1. Install [Rust and Cargo](http://doc.crates.io).
            ```bash
            $ curl -sSf https://static.rust-lang.org/rustup.sh | sh
            ```
        2. Install Diesel CLI.
            ```bash
            $ cargo install diesel_cli --no-default-features --features "postgres"
            ```
    * [sbot2-image-server](https://github.com/drklee3/sbot2-image-server) (Used for rank image generation, etc)
        1. Clone repository.
        2. Install sbot2-image-server dependencies.
            ```bash
            $ npm install
            ```
        3. Install [chromium dependencies](https://github.com/GoogleChrome/puppeteer/blob/master/docs/troubleshooting.md#chrome-headless-doesnt-launch).
            ```bash
            $ sudo apt-get install -y gconf-service libasound2 libatk1.0-0 libc6 libcairo2 libcups2 libdbus-1-3 libexpat1 libfontconfig1 libgcc1 libgconf-2-4 libgdk-pixbuf2.0-0 libglib2.0-0 libgtk-3-0 libnspr4 libpango-1.0-0 libpangocairo-1.0-0 libstdc++6 libx11-6 libx11-xcb1 libxcb1 libxcomposite1 libxcursor1 libxdamage1 libxext6 libxfixes3 libxi6 libxrandr2 libxrender1 libxss1 libxtst6 ca-certificates fonts-liberation libappindicator1 libnss3 lsb-release xdg-utils wget
            ```
        4. Start with `npm start` or with a process manager like [PM2](https://github.com/Unitech/pm2)
2. Clone this repository.
3. Edit [`.env.example`](.env.example) and rename to `.env`.
4. Build and the bot.
    ```bash
    $ cargo run --release
    ```
