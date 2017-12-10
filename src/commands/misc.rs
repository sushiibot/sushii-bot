use reqwest;
use reqwest::header::ContentType;

#[derive(Deserialize)]
struct Response {
    stderr: String,
    stdout: String,
}

command!(play(_ctx, msg, args) {
    let mut code = args.full();

    // check if using code block
    if !code.starts_with("```") || !code.ends_with("```") {
        let _ = msg.channel_id.say("Missing code block");
        let _ = msg.react("❌");
        return Ok(());
    }

    let _ = msg.react("👌");

    // clean up input
    code = code.replace("```rust", "");
    code = code.replacen("```", "", 2); // 2 in case rust in top of code block isn't used
    code = code.replace("\"", "\\\"");  // escape quotes
    code = code.replace("\n", "\\n");   // escape new lines

    // create json data
    let mut data = r#"{"channel":"stable","mode":"debug","crateType":"bin","tests":false,"code": "{CODE}"}"#.to_string();
    data = data.replace("{CODE}", &code);

    // send data
    let client = reqwest::Client::new();
    let res = client.post("http://play.integer32.com/execute")
        .body(data)
        .header(ContentType::json())
        .send()?.error_for_status();

    // check response
    match res {
        Ok(mut val) => {
            let res_obj: Response = val.json()?;

            let mut clean = res_obj.stdout.replace("@", "@\u{200B}");
            clean = clean.replace("`", "'");

            let _ = msg.channel_id.say(format!("```rust\n{}\n{}\n```", res_obj.stderr, clean));
        },
        Err(e) => {
            let _ = msg.channel_id.say(format!("Error: {}", e));
        }
    }
});
