macro_rules! align_up {
    ($val: expr, $align:expr) => {
        ($val) + ((!($val) + 1) & (($align) - 1));
    };
}
