macro_rules! runtest {
    ($expr:expr) => {
        if (!($expr)) {
            panic!(concat!("runtest failed: ", stringify!($expr)));
        }
    };
}
