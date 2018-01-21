// gets the value of an Option<T>, or returns early if None
macro_rules! check_opt {
    ($expr:expr) => {
        match $expr {
            Some(v) => v,
            None => return,
        }
    }
}
