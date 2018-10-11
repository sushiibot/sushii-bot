use serenity::framework::standard::CommandError;
use serenity::model::gateway::GameType;
use serenity::model::id::UserId;
use serenity::utils::Colour;

use inflector::Inflector;

use chrono::{DateTime, Utc};
use timeago;
use utils::user::get_id;
use utils::config::get_pool;

command!(userinfo(ctx, msg, args) {
    // gets the user provided or returns author's id if no user given
    let user = match args.single::<String>().ok().and_then(|x| get_id(&x)) {
        Some(val) => val,
        None => msg.author.id.0,
    };

    let default_color = Colour::default();
    let now = Utc::now();

    debug!("got args");
    
    if let Some(guild) = msg.guild() {
        let (member, presence) = {
            let guild = guild.read();
            debug!("read guild");

            let member = guild.member(user);

            let presence = guild.presences.get(&UserId(user)).cloned();

            (member, presence)
        };

        debug!("found member");

        if let Ok(member) = member {
            let user = {
                member.user.read().clone()
            };
            debug!("read user");

            let pool = get_pool(ctx);
            let last_message = pool.get_user_last_message(user.id.0);

            debug!("got lastmsg");

            let created = {
                let created = DateTime::<Utc>::from_utc(user.created_at(), Utc);

                // precise humanized time 
                let mut f = timeago::Formatter::new();
                f.num_items(4);
                f.ago("");

                let ht = f.convert_chrono(created, now);

                format!("{}\n{}", created.format("%Y-%m-%d %H:%M:%S UTC"), ht)
            };

            let joined = if let Some(joined_date) = member.joined_at {
                let mut f = timeago::Formatter::new();
                f.num_items(4);
                f.ago("");

                let ht = f.convert_chrono(joined_date, now);
                Some(format!("{}\n{}", joined_date.format("%Y-%m-%d %H:%M:%S UTC"), ht))
            } else {
                None
            };

            if let Err(e) = msg.channel_id.send_message(|m| 
                m.embed(|e| {
                    let mut e = e.field("ID", user.id, true);


                    e = e.field("Created At", &created, true);

                    if let Some(joined) = joined {
                        e = e.field("Joined At", &joined, true);
                    }

                    if let Some(last_msg) = last_message {
                        e = e.field("Last Message", last_msg.format("%Y-%m-%d %H:%M:%S UTC"), true);
                    }

                    if let Some(presence) = presence {
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

                    debug!("got status");


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

                    debug!("got user name");

                    e = e.author(|a| a
                        .name(&author_name)
                        .icon_url(&user.face())
                    );

                    debug!("author");

                    e = e.thumbnail(&user.face());

                    debug!("thumbnail");

                    // roles
                    let roles = if let Some(mut roles) = member.roles().clone() {
                        debug!("found roles");
                        
                        // sort roles by position
                        roles.sort_by(|a, b| b.position.cmp(&a.position));
                        debug!("sorted roles");
            
                        // set the color of embed to highest role non-default color
                        if let Some(color) = roles
                            .iter()
                            .find(|r| r.colour.0 != default_color.0)
                            .map(|r| r.colour) {

                            e = e.color(color);
                        }
                            
                        debug!("role color");
                        

                        // convert roles to string
                        let roles_str = roles.iter().map(|role| {
                            role.name.clone()
                        }).collect::<Vec<String>>().join(", ");

                        if roles_str.is_empty() {
                            "N/A".to_owned()
                        } else {
                            roles_str
                        }
                    } else {
                        "N/A".to_owned()
                    };

                    debug!("got roles");

                    e = e.field("Roles", roles, false);

                    // return embed
                    e
                })
            ) {
                warn!("Error while sending userinfo message: {}", e);
            }

            debug!("sent message");
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

    if let Ok(user) = UserId(id).to_user() {
        let _ = msg.channel_id.send_message(|m| m
            .embed(|e| e
                .author(|a| a
                    .name(&format!("{}'s avatar", user.tag()))
                    .url(&user.face().replace("webp", "jpg"))
                )
                .color(0x3498db)
                .image(&user.face().replace("webp", "jpg"))
            ));
    } else {
        return Err(CommandError("Can't find user.".to_owned()));
    }
});
