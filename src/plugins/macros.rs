/// macro to run multiple plugins in a loop
macro_rules! exec_on_message {
    ( [$ctx:expr, $msg:expr], $( $plugin:ident ),* ) => {
        $(
            $plugin::on_message($ctx, $msg);
        )*
    }
}
