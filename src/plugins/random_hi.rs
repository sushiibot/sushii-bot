use serenity::model::Message;
use serenity::prelude::Context;
use rand::{thread_rng, Rng};

pub fn on_message(_: &Context, msg: &Message) {
    // ignore messages other than "hi" and ignore bots
    if msg.content != "hi" || msg.author.bot {
        return ();
    }

    // generate a ranodm number
    let mut rng = thread_rng();
    let n: u32 = rng.gen_range(0, 20);

    // say hi if equals to 1
    if n == 1 {
        let _ = msg.channel_id.say("hi");
    }
}
