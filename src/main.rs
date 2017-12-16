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
extern crate chrono;
extern crate rand;

pub mod schema;
pub mod models;

mod commands;
#[macro_use]
mod plugins;
mod handler;
mod database;

use serenity::framework::StandardFramework;
use serenity::framework::standard::help_commands;
use serenity::framework::standard::DispatchError::*;

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
            .configure(|c| c.owners(owners).prefix("-"))
            .on_dispatch_error(|_, msg, error| {
                // react x whenever an error occurs
                let _ = msg.react("❌");
                match error {
                    NotEnoughArguments { min, given } => {
                        let s = format!("Need {} arguments, but only got {}.", min, given);

                        let _ = msg.channel_id.say(&s);
                    }
                    TooManyArguments { max, given } => {
                        let s = format!("Too many arguments, need {}, but got {}.", max, given);

                        let _ = msg.channel_id.say(&s);
                    }
                    LackOfPermissions(permissions) => {
                        let s = format!(
                            "You do not have permission for this command.  Requires `{:?}`.",
                            permissions
                        );

                        let _ = msg.channel_id.say(&s);
                    }
                    OnlyForGuilds => {
                        let s = format!("This command can only be used in guilds.");

                        let _ = msg.channel_id.say(&s);
                    }
                    RateLimited(seconds) => {
                        let s = format!("Try this again in {} seconds.", seconds);

                        let _ = msg.channel_id.say(&s);
                    }
                    _ => println!("Unhandled dispatch error."),
                }
            })
            .after(|_ctx, msg, cmd_name, error| {
                //  Print out an error if it happened
                if let Err(why) = error {
                    // react x whenever an error occurs
                    let _ = msg.react("❌");
                    let s = format!("Error: {}", why.0);

                    let _ = msg.channel_id.say(&s);
                    println!("Error in {}: {:?}", cmd_name, why);
                }
            })
            .group("Ranking", |g| {
                g.command("rank", |c| {
                    c.desc("Shows your current rank.").exec(
                        commands::levels::rank,
                    )
                })
            })
            .group("Meta", |g| {
                g.command("help", |c| c.exec_help(help_commands::with_embeds))
                    .command("ping", |c| c.exec_str("Pong!"))
                    .command("latency", |c| {
                        c.desc(
                            "Calculates the heartbeat latency between the shard and the gateway.",
                        ).exec(commands::meta::latency)
                    })
                    .command("events", |c| {
                        c.desc("Shows the number of events handled by the bot.")
                            .exec(commands::meta::events)
                    })
            })
            .group("Misc", |g| {
                g.command("play", |c| {
                    c.usage("[rust code]")
                        .desc("Evaluates Rust code in the playground.")
                        .min_args(1)
                        .exec(commands::misc::play)
                })
            })
            .group("Owner", |g| {
                g.command("quit", |c| {
                    c.desc("Gracefully shuts down the bot.")
                        .owners_only(true)
                        .exec(commands::owner::quit)
                }).command("reset events", |c| {
                        c.desc("Resets the events counter.")
                            .owners_only(true)
                            .exec(commands::meta::reset_events)
                    })
                    .command("username", |c| {
                        c.desc("Changes the bot's username.")
                            .owners_only(true)
                            .exec(commands::owner::username)
                    })
            }),
    );

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}
