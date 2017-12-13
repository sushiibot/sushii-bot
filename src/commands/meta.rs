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
        for event in events {
            let mut s = "```ruby\n".to_string();
            let _ = write!(s, "{}: {}\n", event.name, event.count);
            let _ = write!(s, "```");

            let _ = msg.channel_id.say(&s);
        }
    } else {
        return Err(CommandError("Failed to get events.".to_string()));
    }
});
