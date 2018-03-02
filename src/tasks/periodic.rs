use std::{thread, time};

use serenity::prelude::Context;
use serenity::model::id::ChannelId;
use serenity::model::gateway::Ready;

use std::sync::{Once, ONCE_INIT};

use utils::time::now_utc;


static INIT: Once = ONCE_INIT;

pub fn on_ready(_ctx: &Context, _: &Ready) {
    INIT.call_once(|| {
        thread::spawn(move || loop {
            let one_min = time::Duration::from_secs(300);

            let now = now_utc();

            let _ = ChannelId(167165367058300928).say(&now.format("%Y-%m-%d %H:%M:%S UTC"));

            thread::sleep(one_min);
        });
    });
}
