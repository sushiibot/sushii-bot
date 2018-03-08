// gets the value of an Option<T>, or returns early if None
macro_rules! check_opt {
    ( $expr:expr ) => {
        match $expr {
            Some(v) => v,
            None => return,
        }
    }
}

macro_rules! check_res {
    ( $expr:expr ) => {
        match $expr {
            Ok(v) => v,
            Err(e) => {
                warn_discord!("Error: {}", e);
                return
            },
        }
    }
}

macro_rules! check_res_msg {
    ( $expr:expr ) => {
        match $expr {
            Ok(v) => v,
            Err(e) => {
                warn_discord!("Error: {}", e);
                return Err(CommandError::from(get_msg!("error/unknown_error")))
            },
        }
    }
}

macro_rules! warn_discord {
    ( $expr:expr $(, $replace:expr )* ) => {
        {
            use utils::info::bot_update_info;
            use utils::time::now_utc;

            let now = now_utc().format("%Y-%m-%d %H:%M:%S UTC");

            #[allow(unused_mut)]
            let mut s = $expr.to_owned();

            $(
                s = s.replacen("{}", &format!("{}", &$replace), 1);
            )*

            warn!("[{}] {}", &now, &s);
            bot_update_info(&format!("`[{}] WARN: {}`", &now, &s));
        }
    }
}

macro_rules! info_discord {
    ( $expr:expr $(, $replace:expr )* ) => {
        {
            use utils::info::bot_update_info;
            use utils::time::now_utc;

            let now = now_utc().format("%Y-%m-%d %H:%M:%S UTC");

            #[allow(unused_mut)]
            let mut s = $expr.to_owned();

            $(
                s = s.replacen("{}", &format!("{}", &$replace), 1);
            )*

            info!("[{}] {}", &now, &s);
            bot_update_info(&format!("`[{}] INFO: {}`", &now, &s));
        }
    }
}
