use serenity::framework::standard::CommandError;
use reqwest;
use serde_json::Value;

use darksky::DarkskyReqwestRequester;
use darksky::models::Icon;
use env;

const GOOGLE_MAPS_URL: &str = "https://maps.googleapis.com/maps/api/geocode/json?address={ADDRESS}&key={KEY}";

command!(weather(_ctx, msg, args) {
    let location = args.full().replace(' ', "+");

    if location.is_empty() {
        return Err(CommandError::from(get_msg!("error/weather_none_given")));
    }

    let _ = msg.channel_id.broadcast_typing();

    let darksky_key = env::var("DARK_SKY_KEY").expect("Expected DARK_SKY_KEY to be set in environment");
    let google_maps_key = env::var("GOOGLE_MAPS_KEY").expect("Expected GOOGLE_MAPS_KEY to be set in environment");

    let url = GOOGLE_MAPS_URL.replace("{ADDRESS}", &location).replace("{KEY}", &google_maps_key);

    let mut resp = match reqwest::get(&url) {
        Ok(val) => val,
        Err(e) => {
            error!("Error getting geocode: {}", e);
            return Err(CommandError::from(get_msg!("error/google_maps_fetch_failed")));
        }
    };

    let maps_data: Value = match resp.json() {
        Ok(val) => val,
        Err(e) => {
            error!("Error deserializing geocode data: {}", e);
            return Err(CommandError::from(get_msg!("error/google_maps_deserialize_failed")));
        }
    };

    let lat = match maps_data.pointer("/results/0/geometry/location/lat").and_then(|x| x.as_f64()) {
        Some(val) => val,
        None => return Err(CommandError::from(get_msg!("error/google_maps_missing_coord"))),
    };

    let lng = match maps_data.pointer("/results/0/geometry/location/lng").and_then(|x| x.as_f64()) {
        Some(val) => val,
        None => return Err(CommandError::from(get_msg!("error/google_maps_missing_coord"))),
    };


    // get darksky data

    let client = reqwest::Client::new();

    let forecast = match client.get_forecast(&darksky_key, lat, lng) {
        Ok(val) => val,
        Err(e) => {
            error!("Error getting forecast: {}", e);
            return Err(CommandError::from(get_msg!("error/forecast_fetch_failed")));
        }
    };

    let currently = match forecast.currently {
        Some(val) => val,
        None => return Err(CommandError::from(get_msg!("error/forecast_fetch_failed"))),
    };

    // derived from 
    // https://github.com/zeyla/nanobot/blob/0b543692e810344a097f4511e90b414d4184140c/src/bot/plugins/misc.rs
    
    let icon = match currently.icon {
        Some(icon) => match icon {
            Icon::ClearDay => ":sunny:",
            Icon::ClearNight => ":night_with_stars:",
            Icon::Cloudy => ":cloud:",
            Icon::Fog => ":foggy:",
            Icon::Hail | Icon::Sleet | Icon::Snow => ":cloud_snow:",
            Icon::PartlyCloudyDay => ":partly_sunny:",
            Icon::PartlyCloudyNight => ":cloud:",
            Icon::Rain => ":cloud_rain:",
            Icon::Thunderstorm => ":thunder_cloud_rain:",
            Icon::Tornado => ":cloud_tornado:",
            Icon::Wind => ":wind_blowing_face:",
        },
        None => "N/A",
    };

    let temp = if let Some(temp) = currently.temperature {
        let temp_f = (((temp * 9f64) / 5f64) + 32f64) as i16;
            
        format!("{}C ({}F)", temp as i16, temp_f)
    } else {
        "N/A".to_owned()
    };

    let summary = currently.summary.unwrap_or("Summary available".to_owned());

    let s = format!("{} {} {}", icon, summary, temp);

    let _ = msg.channel_id.say(&s);
});
