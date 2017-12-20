use serenity::framework::standard::CommandError;
use reqwest;

use std::fmt::Write;
use std::error::Error;

#[derive(Deserialize)]
struct Prices {
    BTC: Usd,
    ETH: Usd,
    EUR: Usd,
    XMR: Usd,
}

#[derive(Deserialize)]
struct Usd {
    USD: f64,
}

command!(crypto(_ctx, msg, args) {
    // get data
    let mut data: Prices = match reqwest::get("https://min-api.cryptocompare.com/data/pricemulti?fsyms=BTC,ETH,EUR,XMR&tsyms=USD") {
        Ok(mut result) => {
            match result.json() {
                Ok(json) => json,
                Err(why) => return Err(CommandError(why.description().to_owned())),
            }
        },
        Err(why) => return Err(CommandError(why.description().to_owned())),
    };

    let mut s = "```ruby\n".to_owned();
    let _ = write!(s, "BTC: ${}\n", data.BTC.USD);
    let _ = write!(s, "ETH: ${}\n", data.ETH.USD);
    let _ = write!(s, "EUR: ${}\n", data.EUR.USD);
    let _ = write!(s, "XMR: ${}\n", data.XMR.USD);
    let _ = write!(s, "```");
    let _ = msg.channel_id.say(&s);
});
