use serenity::framework::StandardFramework;
use serenity::framework::standard::DispatchError::*;
use serenity::framework::standard::{CreateGroup, CommandOrAlias, CommandOptions};
use serenity::model::Permissions;
use serenity::model::id::UserId;

use utils::time::now_utc;
use utils::config::get_pool;
use utils::config::get_config;

use std::collections::HashSet;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use commands;

pub fn get_framework() -> (StandardFramework, HashMap<String, Arc<CommandOptions>>) {
    let owners: HashSet<UserId> = env::var("OWNER")
        .expect("Expected owner IDs in the environment.")
        .split(',')
        .map(|x| UserId(x.parse::<u64>().unwrap()))
        .collect();

    let blocked_users: HashSet<UserId> = match env::var("BLOCKED_USERS") {
        Ok(val) => {
            val.split(',').map(|x| UserId(x.parse::<u64>().unwrap())).collect()
        },
        Err(_) => HashSet::new(),
    };

    let mut commands_list = HashMap::new();
    
    // reserved names for cmds not implemented yet
    let reserved = vec![
        "help",
        "info",
        "about",
        "simulate",
        "gifprofile",
        "upgrades",
        "kitty",
        "cat",
        "guildinfo",
        "serverinfo",
        "leaderboard",
        "emojiinfo",
        "roleinfo",
        "youtube",
        "yt",
        "vlive",
        "wolframalpha",
        "wa",
        "trivia"
    ];

    let default_cmd = Arc::new(CommandOptions::default());
    for name in &reserved {
        commands_list.insert(name.to_string(), default_cmd.clone());
    }

    let default_prefix = env::var("DEFAULT_PREFIX").expect("Expected DEFAULT_PREFIX in the environment.");

    let framework = StandardFramework::new()
        .configure(|c| c
            .owners(owners)
            .prefix(&default_prefix)
            .dynamic_prefix(|ctx, msg| {
                let pool = get_pool(ctx);

                // get guild id
                if let Some(guild_id) = msg.guild_id() {
                    // get guild config prefix
                    if let Ok(config) = get_config(ctx, &pool, guild_id.0) {
                        return config.prefix;
                    }
                }

                None
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
                IgnoredBot => {
                    return;
                }
                _ => println!("Unhandled dispatch error."),
            }

            let _ = msg.channel_id.say(&s);

            // react x whenever an error occurs
            let _ = msg.react("❌");
        })
        .before(|ctx, msg, cmd_name| {
            if let Some(guild) = msg.guild() {
                let guild = guild.read();
                
                let pool = get_pool(ctx);

                // fetch member
                let member = match guild.members.get(&msg.author.id) {
                    Some(member) => member,
                    None => return false
                };
                // check if has perm
                let permissions = match member.permissions() {
                    Ok(val) => val,
                    Err(_) => return false,
                };

                // shouldnt fail but if it does, false negative better than positive?
                // though false positive might be a lot less likely
                let config = match get_config(ctx, &pool, guild.id.0) {
                    Ok(val) => val,
                    Err(_) => return false,
                };

                // only check disabled channel if user doesn't have MANAGE_GUILD perms
                // those with perms bypass disabled channels
                if !permissions.manage_guild() {
                    // disabled channels
                    if let Some(disabled_channels) = config.disabled_channels {
                        if disabled_channels.contains(&(msg.channel_id.0 as i64)) {
                            return false;
                        }
                    }
                }

                // role channel is disabled for all users since may cause conflicts
                if let Some(channel) = config.role_channel {
                    if channel == msg.channel_id.0 as i64 {
                        return false;
                    }
                }
            }

            let now = now_utc();
            println!("[{}] {}: {} ", now.format("%Y-%m-%d %H:%M:%S UTC"), msg.author.tag(), cmd_name);
            true
        })
        .after(|ctx, msg, _cmd_name, error| {
            {
                let pool = get_pool(ctx);
                pool.update_stat("commands", "commands_executed", Some(1), None);
            }

            //  Print out an error if it happened
            if let Err(why) = error {
                let s = format!("Error: {}", why.0);

                let _ = msg.channel_id.say(&s);

                // react x whenever an error occurs
                let _ = msg.react("❌");
            }
        })
        .help(|_, msg, _, _, _| {
            let _ = msg.channel_id.say("You can find a list of commands here: <https://sushii.xyz/commands>");

            Ok(())
        })
        .simple_bucket("profile_bucket", 10)
        .group("Users", |g| { 
            let g = g
            .guild_only(true)
            .command("rank", |c| c
                .batch_known_as(vec!["profile", "rakn", "rnak"])
                .desc("Shows your rank.")
                .bucket("profile_bucket")
                .cmd(commands::users::levels::rank)
            )
            .command("asdfprofile", |c| c
                .owners_only(true)
                .desc("Shows your profile.")
                .bucket("profile_bucket")
                .cmd(commands::users::profile::profile)
            )
            .command("leaderboard", |c| c
                .desc("Get the leaderboard URL for this guild.")
                .cmd(commands::users::levels::leaderboard)
            )
            .command("toplevels", |c| c
                .desc("Shows the top users.")
                .cmd(commands::users::levels::top_levels)
            )
            .command("topreps", |c| c
                .desc("Shows the top users.")
                .cmd(commands::users::levels::top_reps)
            )
            .command("rep", |c| c
                .desc("Rep a user.")
                .cmd(commands::users::levels::rep)
            )
            .command("fishy", |c| c
                .batch_known_as(vec!["foshy", "fwishy"])
                .desc("Go fishing.")
                .cmd(commands::fishy::fishy)
            )
            .command("topfishies", |c| c
                .desc("Top 10 users with most fishies.")
                .cmd(commands::fishy::fishies_top)
            );

            add_command_group(&mut commands_list, g)
        })
        .group("Notifications", |g| {
            let g = g
            .command("notification add", |c| c
                .desc("Adds a notification.")
                .cmd(commands::notifications::add_notification)
            )
            .command("notification list", |c| c
                .known_as("notifications")
                .desc("Lists your set notifications")
                .cmd(commands::notifications::list_notifications)
            )
            .command("notification delete", |c| c
                .desc("Deletes a notification")
                .cmd(commands::notifications::delete_notification)
            );

            add_command_group(&mut commands_list, g)
        })
        .group("Meta", |g| {
            let g = g
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
                .batch_known_as(vec!["about", "info"])
                .desc("Shows bot stats.")
                .cmd(commands::meta::stats)
            );

            add_command_group(&mut commands_list, g)
        })
        .group("Moderation", |g| {
            let g = g
            .guild_only(true)
            .command("modping", |c| c
                .desc("Pings a moderator for mod action.")
                .cmd(commands::moderation::mod_ping::modping)
            )
            .command("reason", |c| c
                .desc("Edits the reason for moderation action cases.")
                .required_permissions(Permissions::BAN_MEMBERS)
                .cmd(commands::moderation::cases::reason)
            )
            .command("history", |c| c
                .desc("Looks up past cases for a user.")
                .required_permissions(Permissions::BAN_MEMBERS)
                .cmd(commands::moderation::cases::history)
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
            );

            add_command_group(&mut commands_list, g)
        })
        .group("Settings", |g| {
            let g = g
            .guild_only(true)
            .command("settings", |c| c
                .desc("Lists the current server settings.")
                .cmd(commands::settings::list_settings::settings)
            )
            .command("prefix", |c| c
                .desc("Gives you the prefix for this guild, or sets a new prefix (Setting prefix requires MANAGE_GUILD).")
                .cmd(commands::settings::bot::prefix)
            )
            .command("joinmsg", |c| c
                .desc("Gets the guild's join message or sets one if given.")
                .required_permissions(Permissions::MANAGE_GUILD)
                .cmd(commands::settings::messages::joinmsg)
            )
            .command("joinreact", |c| c
                .desc("Gets the guild's join react or sets one if given.")
                .required_permissions(Permissions::MANAGE_GUILD)
                .cmd(commands::settings::messages::joinreact)
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
            .command("joinleavechannel", |c| c
                .desc("Sets the channel for join / leave messages.")
                .required_permissions(Permissions::MANAGE_GUILD)
                .cmd(commands::settings::messages::msg_channel)
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
            .command("disablechannel", |c| c
                .desc("Disables a channel for commands.")
                .required_permissions(Permissions::MANAGE_GUILD)
                .cmd(commands::settings::disable_channel::disable_channel)
            )
            .command("enablechannel", |c| c
                .desc("Enables a channel for commands.")
                .required_permissions(Permissions::MANAGE_GUILD)
                .cmd(commands::settings::disable_channel::enable_channel)
            )
            .command("disabledchannels", |c| c
                .desc("Lists the disabled channels.")
                .required_permissions(Permissions::MANAGE_GUILD)
                .cmd(commands::settings::disable_channel::list_disabled_channels)
            )
            .command("clearsetting", |c| c
                .desc("Clears a guild setting.")
                .required_permissions(Permissions::MANAGE_GUILD)
                .cmd(commands::settings::list_settings::clear_setting)                
            );

            add_command_group(&mut commands_list, g)
        })
        .group("Gallery", |g| {
            let g = g
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
            );

            add_command_group(&mut commands_list, g)
        })
        .group("Roles", |g| {
            let g = g
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
            );

            add_command_group(&mut commands_list, g)
        })
        .group("Text", |g| {
            let g = g
            .command("hug", |c| c
                .usage("[target]")
                .desc("Hug someone.")
                .cmd(commands::text::hug::hug_cmd)
            );

            add_command_group(&mut commands_list, g)
        })
        .group("Misc", |g| {
            let g = g
            .command("play", |c| c
                .usage("[rust code]")
                .desc("Evaluates Rust code in the playground.")
                .cmd(commands::misc::play)
            )
            .command("patreon", |c| c
                .desc("Gets the patreon url. :]")
                .exec(|_, msg, _| {
                    let url = env::var("PATREON_URL").unwrap_or_else(|_| "N/A".to_owned());
                    let _ = msg.channel_id.say(&format!("You can support me on patreon here: <{}> Thanks! :heart:", url))?;
                    Ok(())
                })
            );

            add_command_group(&mut commands_list, g)
        })
        .group("Reminders", |g| {
            let g = g
            .command("remind me", |c| c
                .usage("(in) [time] (to) [description]")
                .desc("Reminds you to do something after some time.")
                .cmd(commands::misc::reminder)
            )
            .command("reminders", |c| c
                .known_as("reminder list")
                .desc("Shows your pending reminders.")
                .cmd(commands::misc::reminders)
            );

            add_command_group(&mut commands_list, g)
        })
        .group("Tags", |g| {
            let g = g
            .guild_only(true)
            .command("tag random", |c| c
                .desc("Gets a random tag.")
                .cmd(commands::tags::tag_random)
            )
            .command("tag info", |c| c
                .usage("[tag name]")
                .desc("Gets information about a tag.")
                .cmd(commands::tags::tag_info)
            )
            .command("tag add", |c| c
                .usage("[tag name] [content]")
                .desc("Adds a new tag.")
                .cmd(commands::tags::tag_add)
            )
            .command("tag list", |c| c
                .known_as("tags")
                .desc("Lists available tags.")
                .cmd(commands::tags::tag_list)
            )
            .command("tag top", |c| c
                .desc("Lists top 10 most used tags.")
                .cmd(commands::tags::tag_top)
            )
            .command("tag search", |c| c
                .usage("[search]")
                .desc("Searches for a tag.")
                .cmd(commands::tags::tag_search)
            )
            .command("tag delete", |c| c
                .known_as("tag remove")
                .usage("[tag name]")
                .desc("Deletes a tag.")
                .cmd(commands::tags::tag_delete)
            )
            .command("tag rename", |c| c
                .usage("[tag name] [new tag name]")
                .desc("Renames a tag.")
                .cmd(commands::tags::tag_rename)
            )
            .command("tag edit", |c| c
                .usage("[tag name] [new content]")
                .desc("Edits a tag's content.")
                .cmd(commands::tags::tag_edit)
            );

            add_command_group(&mut commands_list, g)
        })
        .group("Search", |g| {
            let g = g
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
            .command("crypto", |c| c
                .usage("(symbol)")
                .desc("Gets current cryptocurrency prices.")
                .cmd(commands::search::crypto::crypto)
            )
            .command("urban", |c| c
                .known_as("ud")
                .usage("[word]")
                .desc("Looks up a word definition on Urban Dictionary.")
                .cmd(commands::search::urban::urban)
            );

            add_command_group(&mut commands_list, g)
        })
        .group("User Info", |g| {
            let g = g
            .command("userinfo", |c| c
                .usage("(@mention or ID)")
                .desc("Gets information about a user.")
                .cmd(commands::userinfo::userinfo)
            )
            .command("avatar", |c| c
                .usage("(@mention or ID)")
                .desc("Gets the avatar for a user.")
                .cmd(commands::userinfo::avatar)
            );

            add_command_group(&mut commands_list, g)
        })
        .group("Owner", |g| {
            let g = g
            .help_available(false)
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
            .command("setavatar", |c| c
                .desc("Changes the bot's avatar.")
                .owners_only(true)
                .cmd(commands::owner::set_avatar)
            )
            .command("patron", |c| c
                .desc("Adds or removes a patron.")
                .owners_only(true)
                .cmd(commands::owner::patron)
            )
            .command("patronemoji", |c| c
                .desc("Sets a custom emoji for a patron.")
                .owners_only(true)
                .cmd(commands::owner::patron_emoji)
            )
            .command("listservers", |c| c
                .desc("Lists the servers sushii is in.")
                .owners_only(true)
                .cmd(commands::owner::listservers)
            );

            add_command_group(&mut commands_list, g)
        });

    (framework, commands_list)
}

// adds commands to a hashmap for later use for reference of other command names or usage
fn add_command_group(map: &mut HashMap<String, Arc<CommandOptions>>, cmd_group: CreateGroup) -> CreateGroup {
    for (name, cmd) in &cmd_group.0.commands {
        if let CommandOrAlias::Command(ref val) = *cmd {
            let options = val.options();

            // insert command options for each alias
            for alias in &options.aliases {
                map.insert(alias.to_owned(), options.clone());
            }

            // insert command option for base command
            map.insert(name.to_owned(), val.options().clone());
        }
    }

    cmd_group
}
