use regex::Regex;
use utils::config::get_pool;
use serenity::framework::standard::CommandError;
use serenity::model::id::ChannelId;
use serenity::model::channel::EmbedAuthor;
use serenity::builder::CreateEmbed;

use std::fmt::Write;

command!(reason(ctx, msg, args) {
    let cases = args.single::<String>()?;
    let given_reason = args.full();

    if given_reason.is_empty() {
        return Err(CommandError::from("Please provide a reason."));
    }

    let pool = get_pool(&ctx);

    let guild_id = match msg.guild_id() {
        Some(id) => id.0,
        None => return Err(CommandError::from("No guild.")),
    };

    let re = Regex::new(r"(\d+)\-(\d+)").unwrap();
    let caps = re.captures(&cases);

    let mut first_case: i32;
    let mut second_case: i32;

    // given a range of cases 1234-5678
    if let Some(range) = caps {
        first_case = range.get(1).unwrap().as_str().parse()?;
        second_case = range.get(2).unwrap().as_str().parse()?;
    } else if cases == "latest" {
        first_case = pool.get_latest_mod_action(guild_id);
        second_case = first_case;

        // no cases found
        if first_case == 0 {
            return Err(CommandError::from("No cases available to edit."));
        }
    } else {
        // if only 1 number
        first_case = cases.parse()?;
        second_case = first_case;
    }

    // check if numbers are valid
    if first_case > second_case {
        return Err(CommandError::from("Second case can't be higher than the first."));
    }

    // get the cases
    if let Some(cases) = pool.fetch_mod_actions(guild_id, first_case, second_case) {
        let config = pool.get_guild_config(guild_id);
        let channel = match config.log_mod {
            Some(channel) => ChannelId(channel as u64),
            None => return Err(CommandError::from("There doesn't seem to be a mod log channel set.")),
        };

        // keep track of errored edits, (case number, error message)
        let mut errored = Vec::<(i32, &str)>::new();

        // loop through each case
        for mut case in cases {
            if let Some(msg_id) = case.msg_id {
                // edit message
                let mut message = match channel.message(msg_id as u64) {
                    Ok(msg) => msg,
                    Err(_) => {
                        errored.push((case.case_id, "Failed to fetch message. \
                            Maybe it doesn't exist or I can't access it."));
                        // go to next case
                        continue;
                    }
                };

                let mut embed = match message.embeds.get(0) {
                    Some(val) => val.clone(),
                    None => {
                        errored.push((case.case_id, "Message doesn't seem to have an embed."));
                        // go to next case
                        continue;
                    }
                };

                // edit author
                embed.author = Some(EmbedAuthor {
                    icon_url: Some(msg.author.face()),
                    name: msg.author.tag(),
                    proxy_icon_url: None,
                    url: None,
                });

                // edit reason
                for mut field in &mut embed.fields {
                    if field.name == "Reason" {
                        field.value = given_reason.to_string();
                    }
                }

                // edit the case message embed
                let _ = message.edit(|m| m.embed(|_| CreateEmbed::from(embed.clone())));


                // edit database entry
                case.reason = Some(given_reason.to_owned());
                case.executor_id = Some(msg.author.id.0 as i64);
                pool.update_mod_action(case);
            }
        }
        let mut s = "Finished updating case reasons.".to_owned();
        // check if there were errors
        if !errored.is_empty() {
            let _ = match errored.len() {
                1 => write!(s, "\n\nThere was 1 error while updating cases:\n```\n"),
                _ => write!(s, "\n\nThere were {} errors while updating cases:\n```\n", errored.len()),
            };

            for error in errored {
                let _ = write!(s, "Case #{}: {}\n", error.0, error.1);
            }

            let _ = write!(s, "```");
        }

        // finished editing
        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError::from("I can't seem to find any cases in this range."));
    }

});