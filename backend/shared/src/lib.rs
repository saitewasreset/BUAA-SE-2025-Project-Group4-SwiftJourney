use phf::{Set, phf_set};
use regex::Regex;
use std::sync::LazyLock;

pub static PHONE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^1[3-9]\d{9}$").expect("Failed to create phone validation regex")
});

pub static PHONE_PREFIX_SET: Set<&'static str> = phf_set! {
"134", "135", "136", "137", "138", "139", "144", "147", "148", "150", "151",
"152", "157", "158", "159", "165", "170", "172", "178", "182", "183", "184",
"187", "188", "195", "197", "198", "130", "131", "132", "140", "145", "146",
"155", "156", "166", "167", "171", "175", "176", "185", "186", "196", "133",
"141", "149", "153", "162", "173", "174", "177", "180", "181", "189", "190",
"191", "193", "199", "192"};

pub const API_SUCCESS_CODE: u32 = 200;
pub const API_SUCCESS_MESSAGE: &str = "For Super Earth!";

pub const API_FORBIDDEN_MESSAGE_TEMPLATE: &str =
    "Sorry, but this was meant to be a private game: {}";
pub const API_BAD_REQUEST_MESSAGE_TEMPLATE: &str = "{}";

pub const API_INTERNAL_SERVER_ERROR_MESSAGE: &str =
    "Multiplayer Session Ended: an internal server error has occurred";
