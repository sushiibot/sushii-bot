use serenity::model::channel::Message;
use serenity::prelude::Context;
use serenity::CACHE;

use database::ConnectionPool;

pub fn on_message(_ctx: &Context, _pool: &ConnectionPool, msg: &Message) {
    let cache = CACHE.read();

    // return if self, prevents messages by self
    if msg.author.id.0 == cache.user.id.0 {
        return;
    }

    if msg.is_private() {
        let content = &msg.content;

        // check if user is searching for help or info
        if content == "help" ||
            content == "info" ||
            content == "about" {

            let _ = msg.channel_id.say("Hi!  You can find information and a list of commands here: <https://sushii.xyz>\n\
                The default command prefix is `-` if you want to use any commands here!");
        }

        let s = format!("DM from {} ({}):\nMessage: {}\nAttachments: {:?}", msg.author.tag(),
            msg.author.id, content, msg.attachments.iter().map(|x| &x.url).collect::<Vec<&String>>());
        info_discord!(&s);
    }
}
