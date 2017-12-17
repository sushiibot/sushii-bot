use serenity::framework::standard::CommandError;
use serenity::model::GameType;
use serenity::utils::parse_username;
use serenity::model::UserId;

use inflector::Inflector;

use database;

command!(userinfo(ctx, msg, args) {
    // gets the user provided or returns author's id if no user given
    let mut name = match args.single::<String>() {
        Ok(val) => val,
        Err(_) => msg.author.tag(),
    };

    let user_from_id = match parse_username(&name) {
        Some(id) => {
            match UserId(id).get() {
                Ok(user) => {
                    Some(format!("{}#{}", user.name, user.discriminator))
                },
                Err(_) => {
                    None
                }
            }
        },
        None => {
            None
        }
    };


    // replace name if it's a mention
    if let Some(name_from_id) = user_from_id {
        name = name_from_id;
    }

    // from https://github.com/zeyla/serenity/blob/7fa4df324bcc68f9c0c1c1322eb94931aa267cf0/src/model/guild/mod.rs#L727-L737
    // as a workaround for currently non-working guild.member_named() function
    let (name, discrim) = if let Some(pos) = name.find('#') {
        let split = name.split_at(pos);

        // [1..] is to remove the #
        match split.1[1..].parse::<u16>() {
            Ok(discrim_int) => (split.0, Some(discrim_int)),
            Err(_) => (&name[..], None),
        }
    } else {
        (&name[..], None)
    };
    
    if let Some(guild) = msg.guild() {
        let guild = guild.read().unwrap();

        // fetch user by name containing
        let members = guild.members_containing(&name, false, true);

        let member = members.iter().find(|member| {
                let name_matches = member.user.read().unwrap().name == name;
                let discrim_matches = match discrim {
                    Some(discrim) => member.user.read().unwrap().discriminator == discrim,
                    None => true,
                };

                name_matches && discrim_matches
            })
            .or_else(|| {
                members.iter().find(|member| {
                    member.nick.as_ref().map_or(false, |nick| nick == name)
                })
        });

        if let Some(member) = member {
            let user = member.user.read().unwrap();

            // user id but not in server
            let _ = msg.channel_id.send_message(|m| 
                m.embed(|e| {
                    let mut e = e
                        .field(|f| f
                            .name("ID")
                            .value(user.id)
                        )
                        .field(|f| f
                            .name("Created At")
                            .value(user.created_at().format("%Y-%m-%d %H:%M:%S UTC"))
                        );

                    if let Some(joined_date) = member.joined_at {
                        e = e.field(|f| f
                            .name("Joined At")
                            .value(joined_date.naive_utc().format("%Y-%m-%d %H:%M:%S UTC")));
                    }

                    if let Some(presence) = guild.presences.get(&user.id) {
                        let mut full_status = presence.status.name().to_owned().to_sentence_case();

                        if let Some(ref game) = presence.game {
                            let kind = match game.kind {
                                GameType::Playing => "Playing",
                                GameType::Streaming => "Streaming",
                            };

                            let game = match game.url {
                                Some(ref url) => format!("{} {}\n{}", kind, game.name, url),
                                None => format!("{} {}", kind, game.name),
                            };

                            full_status = format!("{} - {}", full_status, game);
                        }

                        e = e.field(|f| f
                            .name("Status")
                            .value(full_status));
                    } else {
                        e = e.field(|f| f
                            .name("Status")
                            .value("Offline"));
                    }

                    if let Some(ref nick) = member.nick {
                        e = e.author(|a|
                            a.name(&format!("{} - {}", nick, user.tag()))
                            .icon_url(&user.face())
                        );
                    } else {
                        e = e.author(|a|
                            a.name(&user.tag())
                            .icon_url(&user.face())
                        );
                    }

                    e
                })
            );
        } else {
            // member not found
            let s = format!("I cant find a member named `{}`.", name);
            return Err(CommandError(s.to_owned()));
        }
    }
});
