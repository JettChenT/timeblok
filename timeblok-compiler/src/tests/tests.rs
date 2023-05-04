use crate::{compile_deterministic, ir::ExactDateTime};
use insta::{assert_snapshot, glob};
use std::fs;

macro_rules! assert_compile_snapshot {
    ($input:expr, $base_time:expr) => {
        assert_snapshot!(compile_deterministic($input, $base_time.clone()).unwrap());
    };

    ($input:expr) => {
        assert_compile_snapshot!($input, ExactDateTime::from_ymd_hms(2023, 1, 1, 0, 0, 0));
    };
}


#[cfg(not(target_family = "wasm"))]
#[test]
fn run_tests(){
    glob!("bloks/*.tb", |path| {
        let input = fs::read_to_string(path).unwrap();
        assert_compile_snapshot!(&input);
    });
}

#[cfg(target_family = "wasm")]
#[test]
fn run_tests(){
    glob!("wsm-bloks/*.tb", |path| {
        let input = fs::read_to_string(path).unwrap();
        assert_compile_snapshot!(&input);
    });
}