pub fn escape_html(s: &str) -> String {
    let mut escaped = s.to_string();

    escaped = escaped.replace("&", "&amp");
    escaped = escaped.replace("<", "&lt");
    escaped = escaped.replace(">", "&gt");
    escaped = escaped.replace("\"", "&quot");
    escaped = escaped.replace("'", "&#39");

    escaped
}

pub fn clean_url(url: &str) -> String {
    url.replace("(", "%28").replace(")", "%29")
}
