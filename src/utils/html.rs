
pub fn escape_html(s: &str) -> String {
    let mut escaped = s.to_string();

    escaped = escaped.replace("&", "&amp");
    escaped = escaped.replace("<", "&lt");
    escaped = escaped.replace(">", "&gt");
    escaped = escaped.replace("\"", "&quot");
    escaped = escaped.replace("'", "&#39");

    escaped
}
