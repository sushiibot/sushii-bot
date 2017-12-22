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
#[macro_use]
extern crate r2d2_diesel;

#[macro_use]
extern crate diesel_infer_schema;

extern crate dotenv;
extern crate env_logger;
extern crate reqwest;
extern crate typemap;
extern crate chrono;
extern crate chrono_humanize;
extern crate rand;
extern crate inflector;
extern crate regex;

pub mod schema;
pub mod models;
pub mod util;

#[macro_use]
mod plugins;
mod commands;
mod tasks;
mod handler;
mod database;

use serenity::framework::StandardFramework;
use serenity::framework::standard::help_commands;
use serenity::framework::standard::DispatchError::*;

use serenity::model::permissions::Permissions;
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

    let blocked_users: HashSet<UserId> = match env::var("BLOCKED_USERS") {
        Ok(val) => {
            val.split(",").map(|x| UserId(x.parse::<u64>().unwrap())).collect()
        },
        Err(_) => HashSet::new(),
    };


    client.with_framework(
        StandardFramework::new()
            .configure(|c| c
                .owners(owners)
                .dynamic_prefix(|ctx, msg| {
                    let mut data = ctx.data.lock();
                    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

                    // get guild id
                    if let Some(guild_id) = msg.guild_id() {
                        // get guild config prefix
                        if let Some(prefix) = pool.get_prefix(guild_id.0) {
                            return Some(prefix);
                        }
                    }

                    // either no guild found or no prefix set for guild, use default
                    let default_prefix = env::var("DEFAULT_PREFIX").expect("Expected DEFAULT_PREFIX in the environment.");
                    Some(default_prefix)
                })
                .blocked_users(blocked_users)
                .allow_whitespace(true)
            )
            .on_dispatch_error(|_, msg, error| {
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
                    OnlyForOwners => {
                        let _ = msg.channel_id.say("no.");
                    }
                    OnlyForGuilds => {
                        let _ = msg.channel_id.say("This command can only be used in guilds.");
                    }
                    RateLimited(seconds) => {
                        let s = format!("Try this again in {} seconds.", seconds);

                        let _ = msg.channel_id.say(&s);
                    }
                    BlockedUser => {
                        println!("Blocked user {} attemped to use command.", msg.author.tag());
                    }
                    _ => println!("Unhandled dispatch error."),
                }

                // react x whenever an error occurs
                let _ = msg.react("❌");
            })
            .before(|_ctx, msg, cmd_name| {
                println!("{}: {} ", msg.author.tag(), cmd_name);
                true
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
            .group("Settings", |g| {
                g.command("prefix", |c| {
                    c.desc("Gives you the prefix for this guild, or sets a new prefix (Setting prefix requires MANAGE_GUILD).")
                        .exec(commands::settings::prefix)
                    })
                    .command("joinmsg", |c| {
                        c.desc("Gets the guild's join message or sets one if given.")
                            .required_permissions(Permissions::MANAGE_GUILD)
                            .exec(commands::settings::joinmsg)
                    })
                    .command("leavemsg", |c| {
                        c.desc("Gets the guild's leave message or sets one if given.")
                            .required_permissions(Permissions::MANAGE_GUILD)
                            .exec(commands::settings::leavemsg)
                    })
                    .command("modlog", |c| {
                        c.desc("Sets the moderation log channel.")
                            .required_permissions(Permissions::MANAGE_GUILD)
                            .exec(commands::settings::modlog)
                    })
                    .command("msglog", |c| {
                        c.desc("Sets the message log channel.")
                            .required_permissions(Permissions::MANAGE_GUILD)
                            .exec(commands::settings::msglog)
                    })
                    .command("memberlog", |c| {
                        c.desc("Sets the member log channel.")
                            .required_permissions(Permissions::MANAGE_GUILD)
                            .exec(commands::settings::memberlog)
                    })
            })
            .group("Misc", |g| {
                g.command("play", |c| {
                    c.usage("[rust code]")
                        .desc("Evaluates Rust code in the playground.")
                        .min_args(1)
                        .exec(commands::misc::play)
                })
                .command("reminder", |c| {
                    c.usage("[time] [description]")
                        .desc("Reminds you to do something after some time.")
                        .exec(commands::misc::reminder)
                })
                .command("reminders", |c| {
                    c.desc("Shows your pending reminders.")
                        .exec(commands::misc::reminders)
                })
                .command("crypto", |c| {
                    c.usage("(symbol)")
                        .desc("Gets current cryptocurrency prices.")
                        .exec(commands::crypto::crypto)
                })
            })
            .group("User Info", |g| {
                g.command("userinfo", |c| {
                    c.usage("[user]")
                        .desc("Gets information about a user.")
                        .exec(commands::userinfo::userinfo)
                })
                .command("avatar", |c| {
                    c.usage("[user]")
                        .desc("Gets the avatar for a user.")
                        .exec(commands::userinfo::avatar)
                })
            })
            .group("Owner", |g| {
                g.command("quit", |c| {
                    c.desc("Gracefully shuts down the bot.")
                        .owners_only(true)
                        .exec(commands::owner::quit)
                        .known_as("shutdown")
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
