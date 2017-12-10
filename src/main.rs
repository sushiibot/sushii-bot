#![recursion_limit="128"]

#[macro_use]
extern crate log;

#[macro_use]
extern crate serenity;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate diesel;
extern crate r2d2;
extern crate r2d2_diesel;

#[macro_use]
extern crate diesel_infer_schema;

extern crate dotenv;
extern crate env_logger;
extern crate reqwest;
extern crate typemap;

pub mod schema;
pub mod models;

mod commands;
mod plugins;
mod handler;
mod database;

use serenity::framework::StandardFramework;
use serenity::model::UserId;
use serenity::prelude::*;

use std::collections::HashSet;
use std::env;
use dotenv::dotenv;

use typemap::Key;
use database::ConnectionPool;

impl Key for ConnectionPool {
    type Value = ConnectionPool;
}


fn main() {
    dotenv().ok();

    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to debug`.
    env_logger::init().expect("Failed to initialize env_logger");

    let mut client =
        Client::new(
            &env::var("DISCORD_TOKEN").expect("Expected a discord token in the environment."),
            handler::Handler,
        );

    {
        let mut data = client.data.lock();
        let pool = database::init();

        data.insert::<ConnectionPool>(pool);
    }

    let owners: HashSet<UserId> = env::var("OWNER")
        .expect("Expected owner IDs in the environment.")
        .split(",")
        .map(|x| UserId(x.parse::<u64>().unwrap()))
        .collect();

    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.owners(owners).prefix("~"))
            .group("Meta", |g| {
                g.command("ping", |c| c.exec_str("Pong!"))
                    .command("latency", |c| {
                        c.desc(
                            "Calculates the heartbeat latency between the shard and the gateway.",
                        ).exec(commands::meta::latency)
                    })
                    .command("quit", |c| {
                        c.desc("Gracefully shuts down the bot.")
                            .owners_only(true)
                            .exec(commands::owner::quit)
                    })
            })
            .group("Misc", |g| {
                g.command("play", |c| {
                    c.usage("[rust code]")
                        .desc("Evaluates Rust code in the playground.")
                        .exec(commands::misc::play)
                })
            }),
    );

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}
