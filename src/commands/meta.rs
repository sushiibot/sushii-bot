use serenity::framework::standard::CommandError;
use serenity::client::CACHE;
use serenity::client::bridge::gateway::ShardId;
use chrono::Utc;
use psutil;
use utils::config::get_pool;

use SerenityShardManager;
use std::fmt::Write;

command!(latency(ctx, msg) {
    let data = ctx.data.lock();
    let shard_manager = match data.get::<SerenityShardManager>() {
        Some(v) => v,
        None => return Err(CommandError::from("There was a problem getting the shard manager")),
    };

    let manager = shard_manager.lock();
    let runners = manager.runners.lock();

    // Shards are backed by a "shard runner" responsible for processing events
    // over the shard, so we'll get the information about the shard runner for
    // the shard this command was sent over.
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => return Err(CommandError::from("No shard found")),
    };

    let runner_latency = match runner.latency {
        Some(val) => format!("{} ms", val.as_secs() as f64 / 1000.0 + val.subsec_nanos() as f64 * 1e-6),
        None => "N/A".to_owned(),
    };

    let _ = msg.channel_id.say(&format!("The shard latency is {}", runner_latency));
});

command!(ping(_ctx, msg) {
    let start = Utc::now();
    let mut msg = match msg.channel_id.say("Ping!") {
        Ok(val) => val,
        Err(e) => {
            warn!("[CMD:ping] Error sending message: {}", e);
            
            return Ok(());
        },
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
    let pool = get_pool(&ctx);

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
    let pool = get_pool(&ctx);

    if let Ok(()) = pool.reset_events() {
        let _ = msg.channel_id.say("Events have been reset.");
    }
});

// https://github.com/zeyla/nanobot/blob/master/src/commands/owner.rs#L213-L260
command!(stats(_ctx, msg) {
    let processes = match psutil::process::all() {
        Ok(processes) => processes,
        Err(why) => {
            warn!("[CMD:stats] Error getting processes: {:?}", why);

            return Err(CommandError::from("Error getting process list"));
        },
    };

    let process = match processes.iter().find(|p| p.pid == psutil::getpid()) {
        Some(process) => process,
        None => {
            warn!("[CMD:stats] Error getting process stats");
            return Err(CommandError::from("Error getting process stats"));
        },
    };

    let memory = match process.memory() {
        Ok(memory) => memory,
        Err(why) => {
            warn!("[CMD:stats] Error getting process memory: {:?}", why);

            return Err(CommandError::from("Error getting process memory"));
        },
    };

    const B_TO_MB: u64 = 1024 * 1024;

    let mem_total = memory.size / B_TO_MB;
    let mem_rss = memory.resident / B_TO_MB;
    let memory = format!("{}MB/{}MB (RSS/Total)", mem_rss, mem_total);
    let guilds = CACHE.read().guilds.len();

    let _ = msg.channel_id.send_message(|m|
        m.embed(|e| e
            .title("Stats")
            .field("Version", "0.1.2", true)
            .field("Guilds", &guilds.to_string(), true)
            .field("Memory Used", &memory, true)
        )
    );
});
