use serenity::framework::standard::CommandError;
use serenity::client::CACHE;
use serenity::client::bridge::gateway::ShardId;
use SerenityShardManager;
use Uptime;

use chrono::Utc;
use chrono::Duration;
use chrono_humanize::HumanTime;
use std::env;

use psutil;
use sys_info;

use std::fmt::Write;
use utils::config::get_pool;


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
        Some(val) => format!("{:.3} ms", val.as_secs() as f64 / 1000.0 + f64::from(val.subsec_nanos()) * 1e-6),
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
        let end_ms = i64::from(end.timestamp_subsec_millis());
        let start_ms = i64::from(start.timestamp_subsec_millis());

        end_ms - start_ms
    };
    let diff = ((end.timestamp() - start.timestamp()) * 1000) + ms;

    let _ = msg.edit(|m| m.content(&format!("Pong! `[{}ms]`", diff)));
});

command!(events(ctx, msg) {
    let pool = get_pool(ctx);

    if let Ok(evts) = pool.get_events() {
        let mut s = "```ruby\n".to_owned();
        let mut total = 0;
        // go through each events, add to string and sum total
        for event in evts {
            let _ = write!(s, "{}: {}\n", event.name, event.count);
            total += event.count;
        }

        let _ = write!(s, "\nTOTAL: {}\n", total);
        let _ = write!(s, "```");
        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError("Failed to get events.".to_owned()));
    }
});

command!(reset_events(ctx, msg) {
    let pool = get_pool(ctx);

    if let Ok(()) = pool.reset_events() {
        let _ = msg.channel_id.say("Events have been reset.");
    }
});

// https://github.com/zeyla/nanobot/blob/master/src/commands/owner.rs#L213-L260
command!(stats(ctx, msg) {
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
    const K_TO_GB: u64 = 1024 * 1024; // same as B_TO_MB but more clear i guess
    const K_TO_GB_F: f64 = 1024.0 * 1024.0;

    let mem_rss = memory.resident / B_TO_MB;
    let mem_share = memory.share / B_TO_MB;
    let mem_total = memory.size / B_TO_MB;
    let memory = format!("Resident: {} MB\nShared: {} MB\nTotal: {} MB", mem_rss, mem_share, mem_total);

    let cache = CACHE.read();
    let guilds_count = cache.guilds.len();
    let channels_count = cache.channels.len();
    let users_count = cache.guilds
        .values()
        .fold(0, |acc, x| acc + x.read().member_count);
    let users_count_unique = cache.users.len();

    let current_time = Utc::now();
    let start_time = {
        let data = ctx.data.lock();
        match data.get::<Uptime>() {
            Some(val) => *val,
            None => return Err(CommandError::from("There was a problem getting the shard manager")),
        }
    };
    let uptime = current_time.signed_duration_since(start_time);
    let uptime_humanized = format!("{:#}", HumanTime::from(uptime)).replace("in ", "");

    let system_uptime_sec = psutil::system::uptime();
    let system_uptime_duration = Duration::seconds(system_uptime_sec as i64);
    let system_uptime_diff = current_time - system_uptime_duration;
    let system_uptime = current_time.signed_duration_since(system_uptime_diff);
    let system_uptime_humanized = format!("{:#}", HumanTime::from(system_uptime)).replace("in ", "");

    let cpu_num = if let Ok(num) = sys_info::cpu_num() {
        num.to_string()
    } else {
        "N/A".to_owned()
    };

    let cpu_speed = if let Ok(num) = sys_info::cpu_speed() {
        (num as f64 / 1000.0).to_string()
    } else {
        "N/A".to_owned()
    };

    let disk_info = if let Ok(disk) = sys_info::disk_info() {
        let disk_total = disk.total / K_TO_GB;
        let disk_free = disk.free / K_TO_GB;
        format!("{} / {} GB",  disk_total - disk_free, disk_total)
    } else {
        "N/A".to_owned()
    };

    let loadavg = if let Ok(load) = sys_info::loadavg() {
        format!("[{}, {}, {}]", load.one, load.five, load.fifteen)
    } else {
        "N/A".to_owned()
    };

    let system_memory = if let Ok(load) = sys_info::mem_info() {
        let mem_total = load.total as f64 / K_TO_GB_F;
        let mem_free = load.free as f64 / K_TO_GB_F;
        let mem_avail = load.avail as f64 / K_TO_GB_F;
        let mem_buffers = load.buffers as f64 / K_TO_GB_F;
        let mem_cached = load.cached as f64 / K_TO_GB_F;
        format!("Total: {:.3} / {:.3} GB\nAvail: {:.3} GB\nBuffers: {:.3} GB\nCached: {:.3} GB", 
            mem_total - mem_free, mem_total, mem_avail, mem_buffers, mem_cached)
    } else {
        "N/A".to_owned()
    };

    let os_release = if let Ok(release) = sys_info::os_release() {
        release
    } else {
        "N/A".to_owned()
    };

    let os_type = if let Ok(os_type) = sys_info::os_type() {
        os_type
    } else {
        "N/A".to_owned()
    };

    let bot_version = env!("CARGO_PKG_VERSION");

    let owner_tag = env::var("OWNER_TAG").unwrap_or_else(|_| "N/A".to_owned());

    let _ = msg.channel_id.send_message(|m|
        m.embed(|e| e
            .color(0x3498db)
            .title(&format!("sushii v{}", bot_version))
            .url("https://sushii.xyz")
            .field("Author", &owner_tag, true)
            .field("Library", "[serenity-rs](https://github.com/zeyla/serenity/) v0.5.3", true)
            .field("Guilds", &guilds_count.to_string(), true)
            .field("Channels", &channels_count.to_string(), true)
            .field("Users", &format!("{} Total\n{} Unique (cached)",
                users_count, users_count_unique), true)
            .field("Bot Threads", process.num_threads.to_string(), true)
            .field("Bot Uptime", &uptime_humanized, false)
            .field("Bot Memory", &memory, true)
            .field("System Memory", &system_memory, true)
            .field("System", &format!("{} {}\n{} cores @ {} GHz\nLoad Average: {}\nDisk Usage: {}", os_type, os_release, cpu_num, cpu_speed, loadavg, disk_info), true)
            .field("System Uptime", &system_uptime_humanized, false)
        )
    );
});
