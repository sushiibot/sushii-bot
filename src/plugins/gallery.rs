use serenity::model::channel::Message;
use serenity::prelude::Context;
use database::ConnectionPool;

use regex::Regex;
use reqwest;
use std::fmt::Write;
use std::collections::HashMap;

pub fn on_message(_ctx: &Context, pool: &ConnectionPool, msg: &Message) {
    if msg.guild_id().is_none() {
        return;
    }

    // ignore bots
    if msg.author.bot {
        return;
    }

    lazy_static! {
        static ref RE: Regex = Regex::new(r"(https?://[^\s]+)").unwrap();
    }

    let mut s = String::new();

    // check content for urls
    for mat in RE.find_iter(&msg.content) {
        let _ = write!(s, "{}\n", mat.as_str());
    }

    // check attachments for urls
    for img in &msg.attachments {
        let _ = write!(s, "{}\n", img.url);
    }

    // return if there's nothing in message to send to gallery
    if s.is_empty() {
        return;
    }

    if let Some(gallery_urls) = pool.get_gallery_webhook(msg.channel_id.0) {
        let mut json = HashMap::new();
        let cleaned_string = s
            .replace("@everyone", "@\u{200b}everyone")
            .replace("@here", "@\u{200b}here");

        json.insert("content", cleaned_string);
        json.insert("username", msg.author.name.clone());
        json.insert("avatar_url", msg.author.face());

        for url in gallery_urls {
            let client = reqwest::Client::new();
            let res = client
                .post(&url)
                .json(&json)
                .send();

            match res {
                Err(e) => error!("[PLUGIN:gallery] Failed to send gallery webhook: {}", e),
                Ok(response) => {
                    if let Err(server_err) = response.error_for_status() {
                        error!("Failed to send info webhook: {}\n{:?}", &server_err, &json);
                    }
                }
            }
        }
    }
}
