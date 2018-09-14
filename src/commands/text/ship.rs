use serenity::framework::standard::CommandError;

use chrono::Utc;
use chrono::Timelike;
use regex::Regex;
use rand::{thread_rng, Rng};

const EMOJIS: &[&str] = &[
    ":nauseated_face:",
    ":rage:",
    ":angry:",
    ":thinking:",
    ":expressionless:",
    ":neutral_face:",
    ":slight_smile:",
    ":smile:",
    ":blush:",
    ":heart_eyes:",
    ":heart_eyes:", // second for potential index of 10, just use same emoji
];

const HEARTS: &[&str] = &[
    ":heart:",
    ":heartpulse:",
    ":revolving_hearts:",
    ":revolving_hearts:",
];

command!(ship(_ctx, msg, args) {
    let ship_str = args.full();

    if ship_str.is_empty() {
        return Err(CommandError::from(get_msg!("text/ship/error/no_ship_given")));
    }

    lazy_static! {
        static ref RE: Regex = Regex::new(r"<@(\d+)>").unwrap();
    }

    let mut rng = thread_rng();
    let count = ship_str.matches("\u{200b}").count();

    let mut percentage = RE
        .captures_iter(&ship_str)
        .fold(0, |sum, capture| sum + (capture
            .get(1) // id
            .and_then(|group| group // id capture group
                .as_str()
                .parse::<usize>()
                .ok()
            )
            .unwrap_or(0) % 100) // sum all ids % 100
        );
    
    percentage = (percentage * Utc::now().hour() as usize) % 101; // multiply by current hour

    if percentage == 0 { // no ids found
        percentage = rng.gen_range(0, 101); // inclusive low, exclusive high
    }    

    // definitely not rigged
    if count == 1 {
        percentage = 100;
    } else if count == 2 {
        percentage = 0;
    };

    let length = percentage / 10;
    let emoji = EMOJIS[length];
    let back = if length == 10 { // prevent underflow
        0
    } else {
        9 - length
    };

    let heart = if length > 6 {
        HEARTS[length - 7]
    } else {
        ""
    };

    let response = format!("Ship for {}: {}% {}\n\
        {:▬^first$}{}{:▬^second$}",
        ship_str, percentage, heart,
        "", emoji, "",
        first = length, second = back);
    
    // dont need to clean content since in embed
    let _ = msg.channel_id.send_message(|m| m
        .embed(|e| e
            .colour(0x3498db)
            .description(&response)
        )
    );
});
