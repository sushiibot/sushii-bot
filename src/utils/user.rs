use serenity::model::guild::{Member, Guild};

pub fn get_id(value: &str) -> Option<u64> {
    // check if it's already an ID
    if let Ok(id) = value.parse::<u64>() {
        return Some(id);
    }

    // Derived from https://docs.rs/serenity/0.4.5/src/serenity/utils/mod.rs.html#158-172
    if value.starts_with("<@!") {
        let len = value.len() - 1;
        value[3..len].parse::<u64>().ok()
    } else if value.starts_with("<@") {
        let len = value.len() - 1;
        value[2..len].parse::<u64>().ok()
    } else {
        None
    }
}

/// Searches for a member from either an ID, mention, or username string
pub fn find_member(value: &str, guild: &Guild) -> Option<Member> {
    if let Some(user) = get_id(value) {
        // is an ID or mention
        return match guild.member(user) {
            Ok(val) => Some(val),
            Err(_) => None,
        };
    } else {
        // is a name
        // from https://github.com/zeyla/serenity/blob/7fa4df324bcc68f9c0c1c1322eb94931aa267cf0/src/model/guild/mod.rs#L727-L737
        // as a workaround for currently non-working guild.member_named() function
        
        // convert username to lowercase for case insensitive search
        let value = value.to_lowercase();
        let (name, discrim) = if let Some(pos) = value.find('#') {
            let split = value.split_at(pos);

            // [1..] is to remove the #
            match split.1[1..].parse::<u16>() {
                Ok(discrim_int) => (split.0, Some(discrim_int)),
                Err(_) => (&value[..], None),
            }
        } else {
            (&value[..], None)
        };
        

        // fetch user by name containing
        let members = guild.members_containing(&name, false, false);

        let member = members.iter().find(|member| {
                let name_matches = member.user.read().name.to_lowercase() == name;
                let discrim_matches = match discrim {
                    Some(discrim) => member.user.read().discriminator == discrim,
                    None => true,
                };

                name_matches && discrim_matches
            })
            .or_else(|| {
                members.iter().find(|member| {
                    member.nick.as_ref().map_or(false, |nick| nick.to_lowercase() == name)
                })
        });

        if let Some(member) = member {
            Some((*member).clone())
        } else {
            None
        }
    }
}

