#![recursion_limit="256"]

#[macro_use]
extern crate log;

#[macro_use]
extern crate serenity;

#[macro_use]
extern crate serde_derive;
extern crate serde;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

#[macro_use]
extern crate lazy_static;

extern crate dotenv;
extern crate env_logger;
extern crate reqwest;
extern crate typemap;
extern crate chrono;
extern crate chrono_humanize;
extern crate rand;
extern crate inflector;
extern crate regex;
extern crate darksky;
extern crate hourglass;
extern crate psutil;
extern crate sys_info;
extern crate parking_lot;

pub use diesel::r2d2;

pub mod schema;
pub mod models;
#[macro_use]
pub mod utils;

#[macro_use]
mod plugins;
mod commands;
mod tasks;
mod handler;
mod database;

use serenity::framework::StandardFramework;
use serenity::framework::standard::help_commands;
use serenity::framework::standard::HelpBehaviour;
use serenity::framework::standard::DispatchError::*;

use serenity::model::Permissions;
use serenity::model::id::UserId;
use serenity::utils::Colour;
use serenity::prelude::*;
use serenity::client::bridge::gateway::ShardManager;

use parking_lot::Mutex;
use std::sync::Arc;

use std::collections::HashSet;
use std::env;
use dotenv::dotenv;

use typemap::Key;
use database::ConnectionPool;

impl Key for ConnectionPool {
    type Value = ConnectionPool;
}

pub struct SerenityShardManager;
impl Key for SerenityShardManager {
    type Value = Arc<Mutex<ShardManager>>;
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
        ).expect("Failed to create a new client");

    {
        let mut data = client.data.lock();
        let pool = database::ConnectionPool::new();

        data.insert::<ConnectionPool>(pool);
        data.insert::<SerenityShardManager>(Arc::clone(&client.shard_manager));
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
                .on_mention(true)
            )
            .on_dispatch_error(|_, msg, error| {
                let mut s = String::new();
                match error {
                    NotEnoughArguments { min, given } => {
                        s = format!("Need {} arguments, but only got {}.", min, given);
                    }
                    TooManyArguments { max, given } => {
                        s = format!("Too many arguments, need {}, but got {}.", max, given);
                    }
                    LackOfPermissions(permissions) => {
                        s = format!(
                            "You do not have permission for this command.  Requires `{:?}`.",
                            permissions
                        );
                    }
                    OnlyForOwners => {
                        s = "no.".to_owned();
                    }
                    OnlyForGuilds => {
                        s = "This command can only be used in guilds.".to_owned();
                    }
                    RateLimited(seconds) => {
                        s = format!("Try this again in {} seconds.", seconds);
                    }
                    BlockedUser => {
                        println!("Blocked user {} attemped to use command.", msg.author.tag());
                    }
                    _ => println!("Unhandled dispatch error."),
                }

                let _ = msg.channel_id.say(&s);

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
                    let s = format!("Error: {}", why.0);

                    let _ = msg.channel_id.say(&s);
                    println!("Error in {}: {:?}", cmd_name, why);

                    // react x whenever an error occurs
                    let _ = msg.react("❌");
                }
            })
            .help(help_commands::with_embeds)
            .customised_help(help_commands::with_embeds, |c| c
                .individual_command_tip("Hello!\n\
                If you want more information about a specific command, just pass the command as argument.")
                .command_not_found_text("Could not find {}, I'm sorry :(")
                .suggestion_text("Did you mean {}?")
                .lacking_permissions(HelpBehaviour::Strike)
                .lacking_role(HelpBehaviour::Strike)
                .wrong_channel(HelpBehaviour::Strike)
                .embed_success_colour(Colour(0x3498db))
                .embed_error_colour(Colour(0xe74c3c))
            )
            .simple_bucket("profile_bucket", 15)
            .group("Profile", |g| g
                .guild_only(true)
                .command("profile", |c| c
                    .desc("Shows your profile.")
                    .bucket("profile_bucket")
                    .known_as("rank")
                    .cmd(commands::levels::profile)
                )
                .command("rep", |c| c
                    .desc("Rep a user.")
                    .cmd(commands::levels::rep)
                )
            )
            .group("Notifications", |g| g
                .command("notification add", |c| c
                    .desc("Adds a notification.")
                    .cmd(commands::notifications::add_notification)
                )
                .command("notification list", |c| c
                    .desc("Lists your set notifications")
                    .cmd(commands::notifications::list_notifications)
                )
                .command("notification delete", |c| c
                    .desc("Deletes a notification")
                    .cmd(commands::notifications::delete_notification)
                )
            )
            .group("Meta", |g| g
                // .command("helpp", |c| c.exec_help(help_commands::plain))
                .command("ping", |c| c
                    .desc("Gets the ping.")
                    .cmd(commands::meta::ping)
                )
                .command("latency", |c| c
                    .desc("Calculates the heartbeat latency between the shard and the gateway.")
                    .cmd(commands::meta::latency)
                )
                .command("events", |c| c
                    .desc("Shows the number of events handled by the bot.")
                    .cmd(commands::meta::events)
                )
                .command("stats", |c| c
                    .desc("Shows bot stats.")
                    .cmd(commands::meta::stats)
                )
            )
            .group("Moderation", |g| g
                .guild_only(true)
                .command("modping", |c| c
                    .desc("Pings a moderator for mod action.")
                    .cmd(commands::moderation::mod_ping::modping)
                )
                .command("reason", |c| c
                    .desc("Edits the reason for moderation action cases.")
                    .required_permissions(Permissions::MANAGE_GUILD)
                    .cmd(commands::moderation::cases::reason)
                )
                .command("ban", |c| c
                    .usage("[mention or id](,mention or id) [reason]")
                    .desc("Bans a user or ID.")
                    .required_permissions(Permissions::BAN_MEMBERS)
                    .cmd(commands::moderation::ban::ban)
                )
                .command("unban", |c| c
                    .usage("[mention or id](,mention or id) [reason]")
                    .desc("Unbans a user or ID.")
                    .required_permissions(Permissions::BAN_MEMBERS)
                    .cmd(commands::moderation::ban::unban)
                )
                .command("mute", |c| c
                    .usage("[mention or id]")
                    .desc("Mutes a member.")
                    .required_permissions(Permissions::BAN_MEMBERS)
                    .cmd(commands::moderation::mute::mute)
                )
                .command("prune", |c| c
                    .usage("[# of messages]")
                    .known_as("bulkdelete")
                    .desc("Bulk deletes messages. Message count given excludes the message used to invoke this command.")
                    .required_permissions(Permissions::MANAGE_MESSAGES)
                    .cmd(commands::moderation::prune::prune)
                )
            )
            .group("Settings", |g| g
                .guild_only(true)
                .command("prefix", |c| c
                    .desc("Gives you the prefix for this guild, or sets a new prefix (Setting prefix requires MANAGE_GUILD).")
                    .cmd(commands::settings::bot::prefix)
                )
                .command("joinmsg", |c| c
                    .desc("Gets the guild's join message or sets one if given.")
                    .required_permissions(Permissions::MANAGE_GUILD)
                    .cmd(commands::settings::messages::joinmsg)
                )
                .command("leavemsg", |c| c
                    .desc("Gets the guild's leave message or sets one if given.")
                    .required_permissions(Permissions::MANAGE_GUILD)
                    .cmd(commands::settings::messages::leavemsg)
                )
                .command("modlog", |c| c
                    .desc("Sets the moderation log channel.")
                    .required_permissions(Permissions::MANAGE_GUILD)
                    .cmd(commands::settings::logs::modlog)
                )
                .command("msglog", |c| c
                    .desc("Sets the message log channel.")
                    .required_permissions(Permissions::MANAGE_GUILD)
                    .cmd(commands::settings::logs::msglog)
                )
                .command("memberlog", |c| c
                    .desc("Sets the member log channel.")
                    .required_permissions(Permissions::MANAGE_GUILD)
                    .cmd(commands::settings::logs::memberlog)
                )
                .command("inviteguard", |c| c
                    .desc("Enables or disables the invite guard.")
                    .required_permissions(Permissions::MANAGE_GUILD)
                    .cmd(commands::settings::chat::inviteguard)
                )
                .command("muterole", |c| c
                    .desc("Sets the mute role.")
                    .required_permissions(Permissions::MANAGE_GUILD)
                    .cmd(commands::settings::roles::mute_role)
                )
                .command("maxmentions", |c| c
                    .desc("Sets the maximum mentions a user can have in a single message before automatically being muted.")
                    .required_permissions(Permissions::MANAGE_GUILD)
                    .cmd(commands::settings::chat::max_mentions)
                )
                .command("listids", |c| c
                    .desc("Lists the server role ids.")
                    .required_permissions(Permissions::MANAGE_GUILD)
                    .cmd(commands::settings::roles::list_ids)
                )
            )
            .group("Gallery", |g| g
                .guild_only(true)
                .required_permissions(Permissions::MANAGE_GUILD)
                .command("gallery list", |c| c
                    .desc("Lists active galleries.")
                    .cmd(commands::settings::gallery::gallery_list)
                )
                .command("gallery add", |c| c
                    .desc("Adds a gallery.")
                    .cmd(commands::settings::gallery::gallery_add)
                )
                .command("gallery delete", |c| c
                    .desc("Deletes a gallery.")
                    .cmd(commands::settings::gallery::gallery_delete)
                )
            )
            .group("Roles", |g| g
                .guild_only(true)
                .required_permissions(Permissions::MANAGE_GUILD)
                .command("roles set", |c| c
                    .desc("Sets the role configuration.")
                    .required_permissions(Permissions::MANAGE_GUILD)
                    .cmd(commands::settings::roles::roles_set)
                )
                .command("roles get", |c| c
                    .desc("Gets the role configuration.")
                    .required_permissions(Permissions::MANAGE_GUILD)
                    .cmd(commands::settings::roles::roles_get)
                )
                .command("roles channel", |c| c
                    .desc("Sets the roles channel.")
                    .required_permissions(Permissions::MANAGE_GUILD)
                    .cmd(commands::settings::roles::roles_channel)
                )
            )
            .group("Misc", |g| g
                .command("play", |c| c
                    .usage("[rust code]")
                    .desc("Evaluates Rust code in the playground.")
                    .min_args(1)
                    .cmd(commands::misc::play)
                )
                .command("crypto", |c| c
                    .usage("(symbol)")
                    .desc("Gets current cryptocurrency prices.")
                    .cmd(commands::crypto::crypto)
                )
                .command("patreon", |c| c
                    .desc("Gets the patreon url. :]")
                    .exec(|_, msg, _| {
                        let url = env::var("PATREON_URL").unwrap_or("N/A".to_owned());
                        let _ = msg.channel_id.say(&format!("You can support me on patreon here: {}\nThanks! :heart:", url))?;
                        Ok(())
                    })
                )
            )
            .group("Reminders", |g| g
                .command("reminder add", |c| c
                    .usage("[time] [description]")
                    .desc("Reminds you to do something after some time.")
                    .cmd(commands::misc::reminder)
                )
                .command("reminder list", |c| c
                    .desc("Shows your pending reminders.")
                    .cmd(commands::misc::reminders)
                )
                
            )
            .group("Search", |g| g
                .command("weather", |c| c
                    .usage("[location]")
                    .desc("Gets the weather of a location")
                    .cmd(commands::search::weather::weather)
                )
                .command("fm", |c| c
                    .usage("[username] or set [username]")
                    .desc("Gets the last played track on last.fm")
                    .cmd(commands::search::lastfm::fm)
                )
            )
            .group("User Info", |g| g
                .command("userinfo", |c| c
                    .usage("[user]")
                    .desc("Gets information about a user.")
                    .cmd(commands::userinfo::userinfo)
                )
                .command("avatar", |c| c
                    .usage("[user]")
                    .desc("Gets the avatar for a user.")
                    .cmd(commands::userinfo::avatar)
                )
            )
            .group("Owner", |g| g
                .command("quit", |c| c
                    .desc("Gracefully shuts down the bot.")
                    .owners_only(true)
                    .cmd(commands::owner::quit)
                    .known_as("shutdown")
                ).command("reset events", |c| c
                    .desc("Resets the events counter.")
                    .owners_only(true)
                    .cmd(commands::meta::reset_events)
                )
                .command("username", |c| c
                    .desc("Changes the bot's username.")
                    .owners_only(true)
                    .cmd(commands::owner::username)
                )
            ),
    );

    client.threadpool.set_num_threads(10);

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}
