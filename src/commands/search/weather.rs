use serenity::framework::standard::CommandError;
use reqwest;
use serde_json::Value;

use darksky::DarkskyReqwestRequester;
use darksky::Unit;
use darksky::models::Icon;
use darksky::Block;
use env;

use utils::time::now_utc;
use utils::config::get_pool;
use hourglass::Timezone;

const GOOGLE_MAPS_URL: &str = "https://maps.googleapis.com/maps/api/geocode/json?address={ADDRESS}&key={KEY}";

command!(weather(ctx, msg, args) {
    let darksky_key = env::var("DARK_SKY_KEY").expect("Expected DARK_SKY_KEY to be set in environment");
    let google_maps_key = env::var("GOOGLE_MAPS_KEY").expect("Expected GOOGLE_MAPS_KEY to be set in environment");

    let pool = get_pool(ctx);
    let lat;
    let lng;
    let address;
    let mut should_save = false;

    let _ = msg.channel_id.broadcast_typing();

    // check database for a saved location
    if args.is_empty() {
        let saved = match pool.get_weather_location(msg.author.id.0) {
            Some((a, b, c)) => (a, b, c),
            None => return Err(CommandError::from(get_msg!("error/weather_none_given"))),
        };

        if let Some(saved_lat) = saved.0 {
            lat = saved_lat;
        } else {
            return Err(CommandError::from(get_msg!("error/weather_invalid_save")));
        }

        if let Some(saved_lng) = saved.1 {
            lng = saved_lng;
        } else {
            return Err(CommandError::from(get_msg!("error/weather_invalid_save")));
        }

        if let Some(saved_address) = saved.2 {
            address = saved_address;
        } else {
            return Err(CommandError::from(get_msg!("error/weather_invalid_save")));
        }
    } else {
        if let Ok(save) = args.single_n::<String>() {
            if save == "save" {
                should_save = true;
                // remove the save
                let _ = args.skip();
            }
        }

        let location = args.full().replace(' ', "+");

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

        address = match maps_data.pointer("/results/0/formatted_address").and_then(|x| x.as_str()) {
            Some(val) => val.to_string(),
            None => return Err(CommandError::from(get_msg!("error/google_maps_missing_address"))),
        };

        lat = match maps_data.pointer("/results/0/geometry/location/lat").and_then(|x| x.as_f64()) {
            Some(val) => val,
            None => return Err(CommandError::from(get_msg!("error/google_maps_missing_coord"))),
        };

        lng = match maps_data.pointer("/results/0/geometry/location/lng").and_then(|x| x.as_f64()) {
            Some(val) => val,
            None => return Err(CommandError::from(get_msg!("error/google_maps_missing_coord"))),
        };

        if should_save {
            pool.save_weather_location(msg.author.id.0, lat, lng, &address);
        }
    }
    

    // partially derived from 
    // https://github.com/zeyla/nanobot/blob/0b543692e810344a097f4511e90b414d4184140c/src/bot/plugins/misc.rs
    // get darksky data

    let client = reqwest::Client::new();

    let forecast = match client.get_forecast_with_options(&darksky_key, lat, lng, |o| o.unit(Unit::Si).exclude(vec![Block::Hourly, Block::Minutely])) {
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

    let alerts = forecast.alerts;

    let today = match daily.data {
        Some(val) => match val.first() {
            Some(val) => val.clone(),
            None => return Err(CommandError::from(get_msg!("error/forecast_fetch_failed"))),
        },
        None => return Err(CommandError::from(get_msg!("error/forecast_fetch_failed"))),
    };

    let icon = get_icon(&currently.icon);
    let icon_weekly = get_icon(&daily.icon);

    // timezone info
    let tz = Timezone::new(&forecast.timezone).unwrap_or_else(|_| Timezone::utc());

    // temperatures
    let temp = get_temp(&currently.temperature);

    let temp_high = get_temp(&today.temperature_high);
    let temp_high_time = get_time(&tz, &today.temperature_high_time);

    let temp_low = get_temp(&today.temperature_low);
    let temp_low_time = get_time(&tz, &today.temperature_low_time);

    let sunrise_time = get_time(&tz, &today.sunrise_time);
    let sunset_time = get_time(&tz, &today.sunset_time);

    let pressure = if let Some(pressure) = today.pressure {
        format!("{} mbar", pressure)
    } else {
        "N/A".to_owned()
    };

    let apparent_temp = get_temp(&currently.apparent_temperature);
    let dew_point = get_temp(&currently.dew_point);
    let wind_speed = currently.wind_speed.map_or("N/A".to_owned(), |s| s.to_string());
    let wind_gust = currently.wind_gust.map_or("N/A".to_owned(), |s| s.to_string());
    let wind_direction = if let Some(bearing) = currently.wind_bearing {
        let dirs = vec!["N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE",
                        "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW"];
        let emojis = vec![":arrow_up:", ":arrow_upper_right:", ":arrow_right:", ":arrow_lower_right:", 
                            ":arrow_down:", ":arrow_lower_left:", ":arrow_left:", ":arrow_upper_left:"];
        
        let dir = ((bearing as f64 + 11.25) / 22.5) as usize;

        format!("{} {} ({}°)", emojis[(dir / 2) % 7], dirs[dir % 15].to_owned(), bearing)
    } else {
        "N/A".to_owned()
    };

    let humidity = if let Some(humidity) = currently.humidity {
        ((humidity * 100.0) as u8).to_string()
    } else {
        "N/A".to_owned()
    };

    let cloud_cover = if let Some(cloud_cover) = currently.cloud_cover {
        ((cloud_cover * 100.0) as u8).to_string()
    } else {
        "N/A".to_owned()
    };

    let visibility = if let Some(visibility) = currently.visibility {
        format!("{} mi", visibility).to_string()
    } else {
        "N/A".to_owned()
    };

    let moon_phase = if let Some(moon_phase) = today.moon_phase {
        let lunations = vec!["new", "waxing crescent", "first quarter", "waxing gibbous",
                            "full", "waning gibbous", "last quarter", "waning crescent"];

        let emojis = vec![":new_moon:", ":waxing_crescent_moon:", ":first_quarter_moon:", ":waxing_gibbous_moon:",
                        ":full_moon:", ":waning_gibbous_moon:", ":last_quarter_moon:", ":waning_crescent_moon:"];

        let phase = ((moon_phase + 0.0625) / 0.125) as usize;

        format!("{} {}", emojis[phase % 7], lunations[phase % 7])
    } else {
        "N/A".to_owned()
    };

    let color = get_color(&currently.icon);

    let summary = currently.summary.unwrap_or_else(|| "Summary unavailable".to_owned());
    let summary_weekly = daily.summary.unwrap_or_else(|| "Summary unavailable".to_owned());

    let precip_probability = currently.precip_probability.map_or(0u8, |v| v as u8);

    let _ = msg.channel_id.send_message(|m| {
        let mut m = m;
        if should_save {
            m = m.content(get_msg!("info/weather_saved_location"));
        }
        m.embed(|e| {
            let mut e = e.author(|a| a
                .name(&address)
                .icon_url("https://darksky.net/images/darkskylogo.png")
                .url("https://darksky.net/poweredby/")
            )
            .color(color)
            .field("Currently", &format!("{} {}", icon, summary), false)
            .field("This Week", &format!("{} {}", icon_weekly, summary_weekly), false)
            
            // temperature row
            .field("Temperature", &format!("{}\nFeels like {}", temp, apparent_temp), true)
            .field("Temperature High", &format!("{}\nat {}", temp_high, temp_high_time), true)
            .field("Temperature Low", &format!("{}\nat {}", temp_low, temp_low_time), true)

            // sunrise row
            .field("Sunrise Time", &sunrise_time, true)
            .field("Sunset Time", &sunset_time, true)
            .field("Pressure", &pressure, true)

            // other row
            .field("Precipitation %", &format!("{}%", precip_probability), true)
            .field("Humidity", &format!("{}%", humidity), true)
            .field("Dew Point", &dew_point, true)

            // wind row
            .field("Wind Direction", &wind_direction, true)
            .field("Wind Gust", &format!("{} m/s", wind_gust), true)
            .field("Wind Speed", &format!("{} m/s", wind_speed), true)

            // other row
            .field("Visibility", &visibility, true)
            .field("Cloud Cover", &format!("{}%", cloud_cover), true)
            .field("Moon Phase", &moon_phase, true);

            if let Some(alert) = alerts.first() {
                let more_alerts = if alerts.len() == 2 {
                    "(and 1 more alert)".to_owned()
                } else if alerts.len() > 2 {
                    format!("(and {} more alerts)", alerts.len() - 1)
                } else {
                    "".to_owned()
                };

                // check if description is over limit,
                // giving generous limits to url, etc before since lazy
                let desc = if alert.description.len() > 700 {
                    format!("{}...", &alert.description[..700])
                } else {
                    alert.description[..].to_owned()
                };

                e = e.field(format!("⚠ {} (Expires: {})", alert.title, get_time(&tz, &alert.expires)),
                    &format!("{}\n[More Information]({}) {}", desc, alert.uri, more_alerts),
                    false
                )
            }

            e.footer(|f| f
                .text("Powered by Dark Sky")
            )
            .timestamp(now_utc().format("%Y-%m-%dT%H:%M:%S").to_string())
       })
    });
});


fn get_temp(temp: &Option<f64>) -> String {
    if let Some(temp) = *temp {
        let temp_f = (((temp * 9f64) / 5f64) + 32f64) as i16;
                
        format!("{}°C ({}°F)", temp as i16, temp_f)
    } else {
        "N/A".to_owned()
    }
}

fn get_time(tz: &Timezone, time: &Option<u64>) -> String {
    if let Some(time) = *time {
        tz.unix(time as i64, 0).ok().and_then(|x| x.format("%H:%M:%S %Z").ok()).unwrap_or_else(|| "N/A".to_owned())
    } else {
        "N/A".to_owned()
    }
}

fn get_icon(icon: &Option<Icon>) -> String {
    match *icon {
        Some(icon) => match icon {
            Icon::ClearDay => ":sunny:",
            Icon::ClearNight => ":night_with_stars:",
            Icon::Fog => ":foggy:",
            Icon::Hail | Icon::Sleet | Icon::Snow => ":cloud_snow:",
            Icon::PartlyCloudyDay => ":partly_sunny:",
            Icon::Cloudy | Icon::PartlyCloudyNight => ":cloud:",
            Icon::Rain => ":cloud_rain:",
            Icon::Thunderstorm => ":thunder_cloud_rain:",
            Icon::Tornado => ":cloud_tornado:",
            Icon::Wind => ":wind_blowing_face:",
        },
        None => "N/A",
    }.to_owned()
}

fn get_color(icon: &Option<Icon>) -> u32 {
    match *icon {
        Some(icon) => match icon {
            Icon::ClearDay | Icon::PartlyCloudyDay => 0xffac33,
            Icon::ClearNight => 0x226699,
            Icon::Cloudy | Icon::Fog | Icon::Tornado | Icon::Wind | 
            Icon::Hail | Icon::Sleet | Icon::Snow => 0xe1e8ed,
            Icon::PartlyCloudyNight => 0xb2bcc3,
            Icon::Rain | Icon::Thunderstorm => 0x5dadec,
        },
        None => 0xe1e8ed,
    }.to_owned()
}