use serenity::framework::standard::CommandError;
use chrono::Utc;

use std::fmt::Write;
use database;

// no .latency() on serenity::client::bridge::gateway::ShardMessenger?
// command!(latency(ctx, msg) {
//     let ltncy = ctx.shard.lock()
//         .latency()
//         .map_or_else(|| "N/A".to_owned(), |s| {
//             format!("{}.{}s", s.as_secs(), s.subsec_nanos())
//         });
// 
//     let _ = msg.channel_id.say(ltncy);
// });

command!(ping(_ctx, msg) {
    let start = Utc::now();
    let mut msg = match msg.channel_id.say("Ping!") {
        Ok(val) => val,
        Err(_) => return Ok(()),
    };

    let end = Utc::now();
    let ms = {
        let end_ms = end.timestamp_subsec_millis() as i64;
        let start_ms = start.timestamp_subsec_millis() as i64;

        end_ms - start_ms
    };
    let diff = ((end.timestamp() - start.timestamp()) * 1000) + ms;

    let _ = msg.edit(|m| m.content(&format!("Pong! `[{}ms]`", diff)));
});

command!(events(ctx, msg) {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    if let Ok(evts) = pool.get_events() {
        let mut s = "```ruby\n".to_owned();
        let mut total = 0;
        // go through each events, add to string and sum total
        for event in evts {
            let _ = write!(s, "{}: {}\n", event.name, event.count);
            total = total + event.count;
        }

        let _ = write!(s, "\nTOTAL: {}\n", total);
        let _ = write!(s, "```");
        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError("Failed to get events.".to_owned()));
    }
});

command!(reset_events(ctx, msg) {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    if let Ok(()) = pool.reset_events() {
        let _ = msg.channel_id.say("Events have been reset.");
    }
});
