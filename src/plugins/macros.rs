/// macro to run multiple plugins in a loop
macro_rules! exec_on_message {
    ( [$ctx:expr, $msg:expr], $( $plugin:ident ),* ) => {
        $(
            $plugin::on_message($ctx, $msg);
        )*
    }
}


macro_rules! exec_on_ready {
    ( [$ctx:expr, $ready:expr], $( $plugin:ident ),* ) => {
        $(
            $plugin::on_ready($ctx, $ready);
        )*
    }
}
