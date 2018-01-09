use serde_json;
use serde_json::Value;

lazy_static! {
    pub static ref LOCALE: Value = init_locale();
}


fn init_locale() -> Value {
    let locale_file = include_str!("../../assets/locale.json");

    serde_json::from_str(&locale_file).expect("Failed to parse locale JSON")
}

macro_rules! get_msg {
    ($ptr:expr $(, $replace:expr )* ) => {
        {
            use utils::locale::LOCALE;

            let default = format!(
                "Unknown error or info message, please fix in local.json `({})`",
                $ptr
            );

            let mut s = match LOCALE.pointer(&["/", $ptr].join("")) {
                Some(val) => match val.as_str() {
                    Some(string) => string.to_owned(),
                    None => default,
                },
                None => default,
            };

            $(
                s = s.replace("{}", &format!("{}", $replace));
            )*


            s
        }
    }
}
