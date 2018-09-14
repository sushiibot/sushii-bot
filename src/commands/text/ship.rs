use serenity::framework::standard::CommandError;

use chrono::Utc;
use chrono::Datelike;
use chrono::Timelike;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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

    // hash string
    let mut hasher = DefaultHasher::new();
    ship_str.hash(&mut hasher);
    let h = hasher.finish() as usize;
    
    let mut percentage = h % 1000;

    // multiply by hour / ordinal to make static per hour
    let now = Utc::now();
    percentage = (
        percentage *
            (now.hour() + 1) as usize *
            now.ordinal() as usize
        ) % 101; // multiply by current hour

    // definitely not rigged
    let count = ship_str.matches("\u{200b}").count();
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
            .footer(|f| f
                .text("% resets on each hour")
            )
        )
    );
});
