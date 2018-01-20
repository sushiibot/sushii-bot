use serenity::model::channel::Message;
use serenity::prelude::Context;
use database::ConnectionPool;

use regex::Regex;
use reqwest;
use reqwest::header::ContentType;
use std::fmt::Write;

pub fn on_message(_ctx: &Context, pool: &ConnectionPool, msg: &Message) {
    if let None = msg.guild_id() {
        return;
    }

    lazy_static! {
        static ref RE: Regex = Regex::new(r"(https?://[^\s]+)").unwrap();
    }

    let mut s = String::new();

    // check content for urls
    for mat in RE.find_iter(&msg.content) {
        let _ = write!(s, "{}", mat.as_str());
    }

    // check attachments for urls
    for img in msg.attachments.iter() {
        let _ = write!(s, "{}", img.url);
    }

    // return if there's nothing in message to send to gallery
    if s.is_empty() {
        return;
    }

    // clean string
    s = s.replace("\n",  "\\n")
         .replace("\'", "\\'")
         .replace("\"",  "\\\"")
         .replace("\r", "\\r")
         .replace("\t", "\\t");

    if let Some(gallery_urls) = pool.get_gallery_webhook(msg.channel_id.0) {
        let mut data =
            r#"{"content": "{CONTENT}", "username": "{USERNAME}", "avatar_url": "{AVATAR_URL}"}"#.to_owned();


        for url in gallery_urls {
            data = data.replace("{CONTENT}", &s);
            data = data.replace("{USERNAME}", &msg.author.name);
            data = data.replace("{AVATAR_URL}", &msg.author.face());

            let client = reqwest::Client::new();
            let res = client
                .post(&url)
                .body(data.clone())
                .header(ContentType::json())
                .send();

            match res {
                Err(e) => error!("[PLUGIN:gallery] Failed to send gallery webhook: {}", e),
                Ok(response) => {
                    if let Err(server_err) = response.error_for_status() {
                        error!("Failed to send info webhook: {}\n{}", &server_err, &data);
                    }
                }
            }
        }
    }
}
