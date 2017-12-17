use serenity::framework::standard::CommandError;

use database;

command!(userinfo(ctx, msg, args) {
    // gets the user provided or returns author's id if no user given
    let name = match args.single::<String>() {
        Ok(val) => val,
        Err(_) => msg.author.tag(),
    };

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
                        .author(|a|
                            a.name(&user.tag())
                            .icon_url(&user.face())
                        )
                        .field(|f| f
                            .name("ID")
                            .value(user.id)
                        )
                        .field(|f| f
                            .name("Created At")
                            .value(user.created_at().format("%Y-%m-%d %H:%M:%S"))
                        );

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
