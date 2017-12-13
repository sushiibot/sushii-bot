use serenity::framework::standard::CommandError;

use std::fmt::Write;
use database;

command!(latency(ctx, msg) {
    let latency = ctx.shard.lock()
        .latency()
        .map_or_else(|| "N/A".to_string(), |s| {
            format!("{}.{}s", s.as_secs(), s.subsec_nanos())
        });

    let _ = msg.channel_id.say(latency);
});

command!(events(ctx, msg) {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    if let Ok(events) = pool.get_events() {
        let mut s = "```ruby\n".to_string();
        let mut total = 0;
        // go through each events, add to string and sum total
        for event in events {
            let _ = write!(s, "{}: {}\n", event.name, event.count);
            total = total + event.count;
        }

        let _ = write!(s, "\nTOTAL: {}\n", total);
        let _ = write!(s, "```");
        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError("Failed to get events.".to_string()));
    }
});

command!(reset_events(ctx, msg) {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    if let Ok(()) = pool.reset_events() {
        let _ = msg.channel_id.say("Events have been reset.");
    }
});
