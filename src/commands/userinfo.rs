use serenity::framework::standard::CommandError;
use serenity::model::gateway::GameType;
use serenity::model::id::UserId;

use inflector::Inflector;

use utils::user::get_id;
use utils::config::get_pool;

command!(userinfo(ctx, msg, args) {
    // gets the user provided or returns author's id if no user given
    let user = match args.single::<String>() {
        Ok(val) => val,
        Err(_) => msg.author.id.0.to_string(),
    };
    println!("got args");
    
    if let Some(guild) = msg.guild() {
        let guild = guild.read();
        println!("read guild");

        let member = match get_id(&user) {
            Some(val) => guild.member(val),
            None => return Err(CommandError::from("No user found.")),
        };
        println!("found member");

        if let Ok(member) = member {
            let user = member.user.read();
            println!("read user");

            let pool = get_pool(&ctx);
            let last_message = pool.get_user_last_message(user.id.0);

            println!("got lastmsg");

            if let Err(e) = msg.channel_id.send_message(|m| 
                m.embed(|e| {
                    let mut e = e.field("ID", user.id, true);


                    e = e.field("Created At", user.created_at().format("%Y-%m-%d %H:%M:%S UTC"), true);

                    if let Some(joined_date) = member.joined_at {
                        e = e.field("Joined At", joined_date.naive_utc().format("%Y-%m-%d %H:%M:%S UTC"), true);
                    }

                    if let Some(last_msg) = last_message {
                        e = e.field("Last Message", last_msg.format("%Y-%m-%d %H:%M:%S UTC"), true);
                    }

                    if let Some(presence) = guild.presences.get(&user.id) {
                        let mut full_status = presence.status.name().to_owned().to_sentence_case();

                        if let Some(ref game) = presence.game {
                            let kind = match game.kind {
                                GameType::Playing => "Playing",
                                GameType::Streaming => "Streaming",
                                GameType::Listening => "Listening to"
                            };

                            let game = match game.url {
                                Some(ref url) => format!("{} {}\n{}", kind, game.name, url),
                                None => format!("{} {}", kind, game.name),
                            };

                            full_status = format!("{} - {}", full_status, game);
                        }

                        e = e.field("Status", full_status, false);
                    } else {
                        e = e.field("Status", "Offline", false);
                    }

                    println!("got status");


                    // AUTHOR - nick - tag [bot]

                    let mut author_name;

                    // check if user has a nickname
                    if let Some(ref nick) = member.nick {
                        author_name = format!("{} - {}", nick, user.tag());
                    } else {
                        author_name = user.tag();
                    }

                    // append [BOT] to author if bot
                    if user.bot {
                        author_name = format!("{} [BOT]", author_name);
                    }

                    println!("got user name");

                    e = e.author(|a| a
                        .name(&author_name)
                        .icon_url(&user.face())
                    );

                    println!("author");

                    e = e.thumbnail(&user.face());

                    println!("thumbnail");

                    // roles
                    let roles = match member.roles() {
                        Some(roles) => {
                            println!("found roles");
                            let mut roles = roles.clone();
                            println!("clone roles");
                            
                            // sort roles by position
                            roles.sort_by(|a, b| b.position.cmp(&a.position));
                            println!("sorted roles");
                
                            // set the color of embed to highest role color
                            if roles.len() > 0 {
                                e = e.color(roles[0].colour);
                            }
                            println!("role color");
                            

                            // convert roles to string
                            let roles_str = roles.iter().map(|role| {
                                role.name.clone()
                            }).collect::<Vec<String>>().join(", ");

                            if roles_str.is_empty() {
                                "N/A".to_owned()
                            } else {
                                roles_str
                            }
                        },
                        None => "N/A".to_owned(),
                    };

                    println!("got roles");

                    e = e.field("Roles", roles, false);

                    // return embed
                    e
                })
            ) {
                warn!("Error while sending userinfo message: {}", e);
            }

            println!("sent message");
        } else {
            // user not found
            return Err(CommandError::from("I cant find that user."));
        }
    }
});

command!(avatar(_ctx, msg, args) {
    let name = match args.single::<String>() {
        Ok(val) => val,
        Err(_) => msg.author.id.0.to_string(),
    };

    let id = match get_id(&name) {
        Some(id) => id,
        None => return Err(CommandError("Invalid mention.".to_owned())),
    };

    if let Ok(user) = UserId(id).get() {
        let _ = msg.channel_id.send_message(|m| m
            .embed(|e| e
                .author(|a| a
                    .name(&format!("{}'s avatar", user.tag()))
                    .url(&user.face())
                )
                .color(0x3498db)
                .image(&user.face())
            ));
    } else {
        return Err(CommandError("Can't find user.".to_owned()));
    }
});
