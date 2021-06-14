macro_rules! runtest {
    ($expr:expr) => {
        if (!($expr)) {
            panic!(concat!("runtest failed: ", stringify!($expr)));
        }
    };
}

macro_rules! align_up {
    ($val: expr, $align:expr) => {
        ($val) + ((!($val) + 1) & (($align) - 1));
    };
}
