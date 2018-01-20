use std::env;

use reqwest;
use reqwest::header::ContentType;

pub fn bot_update_info(status: &str) {
    let webhook_url = env::var("INFO_WEBHOOK").expect("Expected INFO_WEBHOOK in the environment.");

    let mut data = r#"{"content": "{STATUS}"}"#.to_owned();
    data = data.replace("{STATUS}", &status);

    let client = reqwest::Client::new();
    let res = client
        .post(&webhook_url)
        .body(data)
        .header(ContentType::json())
        .send();

    match res {
        Err(e) => error!("Failed to send info webhook: {}\nData: {}", e, &status),
        Ok(response) => {
            if let Err(server_err) = response.error_for_status() {
                error!(
                    "Failed to send info webhook: {}\nData: {}",
                    &server_err, &status
                );
            }
        }
    }
}
