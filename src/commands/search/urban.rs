use serenity::framework::standard::CommandError;
use utils::config::*;
use urbandictionary::ReqwestUrbanDictionaryRequester;

command!(urban(ctx, msg, args) {
    if args.is_empty() {
        return Err(CommandError::from(get_msg!("error/no_urban_word_given")));
    }

    let mut msg = match msg.channel_id.say("Searching for definition...") {
        Ok(msg) => msg,
        Err(_) => return Ok(()),
    };

    let query = args.rest();

    let client = get_reqwest_client(&ctx);
    let mut response = match client.definitions(&query[..]) {
        Ok(response) => response,
        Err(why) => {
            warn_discord!(format!("Err retrieving word '{}': {:?}", query, why));

            return Err(CommandError::from(get_msg!("error/urban_failed_retrieve")));
        },
    };

    let definition = match response.definitions.get_mut(0) {
        Some(definition) => definition,
        None => {
            let _ = msg.edit(|m| m.content("No definition found."));

            return Ok(());
        },
    };

    if definition.definition.len() > 2048 {
        definition.definition.truncate(2045);
        definition.definition.push_str("...");
    }

    if definition.example.len() > 2048 {
        definition.example.truncate(2045);
        definition.example.push_str("...");
    }

    let _ = msg.edit(|m| m
        .content("")
        .embed(|e| e
            .author(|a| a
                .name(&format!("Definition for {}", definition.word))
                .url(&definition.permalink)
                .icon_url("https://i.imgur.com/jkF8UJN.jpg")
            )
            .colour(0x1D2439)
            .description(&definition.definition)
            .field("Example", &definition.example, false)
            .field(":+1:", &definition.thumbs_up.to_string(), true)
            .field(":-1:", &definition.thumbs_down.to_string(), true)
            .footer(|f| f
                .text(&format!("#{} - Submitted by {}", definition.id, definition.author))
            )
        )
    );
});
