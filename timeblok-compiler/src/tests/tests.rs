use crate::{compile_deterministic, ir::ExactDateTime};
use insta::assert_snapshot;

macro_rules! assert_compile_snapshot {
    ($input:expr, $base_time:expr) => {
        assert_snapshot!(compile_deterministic($input, $base_time.clone()).unwrap());
    };

    ($input:expr) => {
        assert_compile_snapshot!($input, ExactDateTime::from_ymd_hms(2023, 1, 1, 0, 0, 0));
    };
}

#[test]
fn test_sanity() {
    assert_compile_snapshot!(r###"2023-4-4
10am wake up and eat breakfast
11am go to work
    "###);
}

#[test]
fn test_filters(){
    assert_compile_snapshot!(r###"2023--
{mon or tue or thu}
10am wakeup
{sat}
20:00 wekly review
    "###);

    assert_compile_snapshot!(r###"2024--
{workday}
6am wake up
{weekend}
10pm sleep
    "###);
}
