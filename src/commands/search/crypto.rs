#![allow(non_snake_case)]

use serenity::framework::standard::CommandError;
use utils::config::*;
use std::collections::HashMap;

use std::fmt::Write;
use std::error::Error;

const CRYPTO_COMPARE_URL: &str = "https://min-api.cryptocompare.com/data/pricemulti?fsyms={COINS}&tsyms=USD";

#[derive(Deserialize)]
struct Usd {
    USD: f64,
}

command!(crypto(ctx, msg, args) {
    let coins = match args.single::<String>() {
        Ok(val) => val.replace(" ", "").to_uppercase(),
        Err(_) => "BTC,ETH,XRP,BCH,LTC,XLM,NEO".to_owned(),
    };
    
    let _ = msg.channel_id.broadcast_typing();

    // get data
    let client = get_reqwest_client(&ctx);
    let mut data: HashMap<String, Usd> = match client.get(&CRYPTO_COMPARE_URL.replace("{COINS}", &coins)).send() {
        Ok(mut result) => {
            match result.json() {
                Ok(json) => json,
                Err(_) => return Err(CommandError("Not found".to_owned())),
            }
        },
        Err(why) => return Err(CommandError(why.description().to_owned())),
    };

    let mut s = "```ruby\n".to_owned();
    for (name, price) in &data {
        let _ = write!(s, "{}: ${}\n", name, price.USD);
    }
    let _ = write!(s, "```");
    let _ = msg.channel_id.say(&s);
});
