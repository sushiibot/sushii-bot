use utils::config::get_pool;
use utils::user::get_id;
use serenity::framework::standard::CommandError;
use serenity::model::UserId;
use serenity::Error;
use serenity::model::ModelError::InvalidPermissions;
use serenity::model::ModelError::DeleteMessageDaysAmount;

use regex::Regex;
use std::fmt::Write;

command!(ban(ctx, msg, args) {
    // get the target
    let raw_users = args.single::<String>()?;
    let split = raw_users.split(",");
    let mut users = Vec::new();

    // loop through each one and parse the user id
    for user in split {
        match get_id(&user) {
            Some(val) => users.push(val),
            None => return Err(CommandError::from("Malformed mention or ID.")),
        };
    }

    // get the guild
    let guild = match msg.guild() {
        Some(val) => val.read().unwrap().clone(),
        None => return Err(CommandError::from("No guild.")),
    };

    // get the reason
    let reason_raw = args.full();
    let reason;

    let re = Regex::new(r"\d{17,18}[a-zA-Z ]+").unwrap();
    if re.is_match(&reason_raw) && !reason_raw.starts_with("\\") {
        return Err(CommandError::from("There seems to be a user ID in the beginning of your reason. \
                If you're banning multiple users at once, be sure to not leave spaces between the commas, IDs or mentions.  \
                If you actually wanted an ID in your reason, prefix your reason with a backslash (\\\\):\n\
                Example: `-ban 12345678910,456789123 \\10987654321 is his best friend but he's really smelly`"));
    } else if reason_raw.starts_with("\\") && !reason_raw.is_empty() {
        reason = Some(&reason_raw[1..]);
    } else if !reason_raw.is_empty() {
        reason = Some(&reason_raw[..]);
    } else {
        reason = None;
    }

    let mut bans = match guild.bans() {
        Ok(val) => val.iter().map(|x| x.user.id.0).collect(),
        Err(_) => Vec::new(),
    };

    // log the ban in the database
    let pool = get_pool(&ctx);
    let mut s = String::new();

    let _ = write!(s, "```ruby\n");

    let _ = write!(s, "Attempted to ban {} users:\n\n", users.len());

    for u in users {
        // fetch the user for tag
        let user = match UserId(u).get() {
            Ok(val) => val,
            Err(e) => {
                let _ = write!(s, "{} - Error: Failed to fetch user: {}\n", u, &e);
                continue;
            }
        };

        // format a tag (id) string for the user
        let user_tag_id = format!("{} ({})", user.tag(), user.id.0);

        // check if already banned in server ban list or in current currently
        if bans.contains(&u) {
            let _ = write!(s, "{} - Error: User is already banned\n", user_tag_id);
            continue;
        }


        // potentially if banning a user who's already banned, it will make another case for them as
        // it won't create an error.  check before if user is already banned?
        let case_id = pool.add_mod_action("ban", guild.id.0, &user, reason, true, Some(msg.author.id.0)).case_id;

        // ban the user
        let _ = match guild.ban(u, 7) {
            Err(Error::Model(InvalidPermissions(permissions))) => {
                let e = format!("I don't have permission to ban this user, requires: `{:?}`.", permissions);
                let _ = write!(s, "{} - Error: {}\n", &user_tag_id, &e);
                pool.remove_mod_action(guild.id.0, &user, case_id);
            },
            Err(Error::Model(DeleteMessageDaysAmount(num))) => {
                let e = format!("The number of days worth of messages to delete is over the maximum: ({}).", num);
                let _ = write!(s, "{} - Error: {}\n", &user_tag_id, &e);
                pool.remove_mod_action(guild.id.0, &user, case_id);
            }
            Err(_) => {
                let e = "There was an unknown error trying to ban this user.";
                let _ = write!(s, "{} - Error: {}\n", &user_tag_id, &e);
                pool.remove_mod_action(guild.id.0, &user, case_id);
            },
            Ok(_) => {
                let _ = write!(s, "{} - Successfully banned.\n", &user_tag_id);
                // add the ban to the vec to prevent dupe bans
                bans.push(u);
            },
        };
    }

    let _ = write!(s, "```");

    
    let _ = msg.channel_id.say(&s);
});
