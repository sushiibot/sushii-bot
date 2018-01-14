use serenity::framework::standard::CommandError;
use reqwest;
use serde_json::Value;

use darksky::DarkskyReqwestRequester;
use darksky::Unit;
use darksky::models::Icon;
use env;

use utils::time::now_utc;

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

    let address = match maps_data.pointer("/results/0/formatted_address").and_then(|x| x.as_str()) {
        Some(val) => val,
        None => return Err(CommandError::from(get_msg!("error/google_maps_missing_address"))),
    };

    let lat = match maps_data.pointer("/results/0/geometry/location/lat").and_then(|x| x.as_f64()) {
        Some(val) => val,
        None => return Err(CommandError::from(get_msg!("error/google_maps_missing_coord"))),
    };

    let lng = match maps_data.pointer("/results/0/geometry/location/lng").and_then(|x| x.as_f64()) {
        Some(val) => val,
        None => return Err(CommandError::from(get_msg!("error/google_maps_missing_coord"))),
    };

    // partially derived from 
    // https://github.com/zeyla/nanobot/blob/0b543692e810344a097f4511e90b414d4184140c/src/bot/plugins/misc.rs
    // get darksky data

    let client = reqwest::Client::new();

    let forecast = match client.get_forecast_with_options(&darksky_key, lat, lng, |o| o.unit(Unit::Si)) {
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

    let daily = match forecast.daily {
        Some(val) =>  val,
        None => return Err(CommandError::from(get_msg!("error/forecast_fetch_failed"))),
    };

    let today = match daily.data {
        Some(val) => match val.first() {
            Some(val) => val.clone(),
            None => return Err(CommandError::from(get_msg!("error/forecast_fetch_failed"))),
        },
        None => return Err(CommandError::from(get_msg!("error/forecast_fetch_failed"))),
    };

    let icon = get_icon(&currently.icon);
    let icon_weekly = get_icon(&daily.icon);
    
    let temp = get_temp(currently.temperature);
    let apparent_temp = get_temp(currently.apparent_temperature);
    let dew_point = get_temp(currently.dew_point);
    let wind_speed = currently.wind_speed.map_or("N/A".to_owned(), |s| s.to_string());
    let wind_direction = if let Some(bearing) = currently.wind_bearing {
        let dirs = vec!["N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE",
                        "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW"];
        
        let dir = ((bearing as f64 + 11.25) / 22.5) as usize;

        format!("{} ({}°)", dirs[dir % 16].to_owned(), bearing)
    } else {
        "N/A".to_owned()
    };

    let humidity = if let Some(humidity) = currently.humidity {
        ((humidity * 100.0) as u8).to_string()
    } else {
        "N/A".to_owned()
    };

    let cloud_cover = if let Some(cloud_cover) = currently.cloud_cover {
        (cloud_cover * 100.0).to_string()
    } else {
        "N/A".to_owned()
    };

    let moon_phase = if let Some(moon_phase) = today.moon_phase {
        let lunations = vec!["new", "waxing crescent", "first quarter", "waxing gibbous",
                            "full", "waning gibbous", "last quarter", "waning crescent"];

        let emojis = vec![":new_moon:", ":waxing_crescent_moon:", ":first_quarter_moon:", ":waxing_gibbous_moon:",
                        ":full_moon:", ":waning_gibbous_moon:", ":last_quarter_moon:", ":waning_crescent_moon:"];

        let phase = ((moon_phase + 0.0625) / 0.125) as usize;

        format!("{} {}", emojis[phase], lunations[phase])
    } else {
        "N/A".to_owned()
    };

    let color = get_color(&currently.icon);

    let summary = currently.summary.unwrap_or("Summary unavailable".to_owned());
    let summary_weekly = daily.summary.unwrap_or("Summary unavailable".to_owned());

    let precip_probability = currently.precip_probability.map_or(0u8, |v| v as u8);

    let _ = msg.channel_id.send_message(|m| m
       .embed(|e| e
            .author(|a| a
                .name(&address)
                .icon_url("https://darksky.net/images/darkskylogo.png")
                .url("https://darksky.net/poweredby/")
            )
            .color(color)
            .field(|f| f
                .name("Currently")
                .value(&format!("{} {}", icon, summary))
                .inline(false)
            )
            .field(|f| f
                .name("This Week")
                .value(&format!("{} {}", icon_weekly, summary_weekly))
                .inline(false)
            )
            .field(|f| f
                .name("Temperature")
                .value(&temp)
            )
            .field(|f| f
                .name("Apparent Temperature")
                .value(&apparent_temp)
            )
            .field(|f| f
                .name("Precipitation %")
                .value(&format!("{}%", precip_probability))
            )
            .field(|f| f
                .name("Humidity")
                .value(&format!("{}%", humidity))
            )
            .field(|f| f
                .name("Dew Point")
                .value(&dew_point)
            )
            .field(|f| f
                .name("Wind Speed")
                .value(&format!("{} m/s", wind_speed))
            )
            .field(|f| f
                .name("Wind Direction")
                .value(&wind_direction)
            )
            .field(|f| f
                .name("Cloud Cover")
                .value(&format!("{}%", cloud_cover))
            )
            .field(|f| f
                .name("Moon Phase")
                .value(&moon_phase)
            )
            .footer(|f| f
                .text("Powered by Dark Sky")
            )
            .timestamp(now_utc().format("%Y-%m-%dT%H:%M:%S").to_string())
       )
    );
});


fn get_temp(temp: Option<f64>) -> String {
    if let Some(temp) = temp {
        let temp_f = (((temp * 9f64) / 5f64) + 32f64) as i16;
                
        format!("{}°C ({}°F)", temp as i16, temp_f)
    } else {
        "N/A".to_owned()
    }
}

fn get_icon(icon: &Option<Icon>) -> String {
    match icon {
        &Some(icon) => match icon {
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
        &None => "N/A",
    }.to_owned()
}

fn get_color(icon: &Option<Icon>) -> u32 {
    match icon {
        &Some(icon) => match icon {
            Icon::ClearDay | Icon::PartlyCloudyDay => 0xffac33,
            Icon::ClearNight => 0x226699,
            Icon::Cloudy | Icon::Fog | Icon::Tornado | Icon::Wind => 0xe1e8ed,
            Icon::Hail | Icon::Sleet | Icon::Snow => 0xe1e8ed,
            Icon::PartlyCloudyNight => 0xb2bcc3,
            Icon::Rain | Icon::Thunderstorm => 0x5dadec,
        },
        &None => 0xe1e8ed,
    }.to_owned()
}