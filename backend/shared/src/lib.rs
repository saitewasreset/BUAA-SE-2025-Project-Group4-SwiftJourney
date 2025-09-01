/*
 * Super Earth.
 * Our home.
 * Prosperity.
 * Liberty.
 * (Hi there)
 * (Hey)
 * Democracy.
 * Our way of life.
 * (Hello)
 * But freedom doesn't come free.
 * No...
 * sweet Liberty...
 * NOOOO!
 * (laughs) Look familiar?
 * Scenes like these are happening all over the galaxy, right now!
 * You could be next.
 * That is, unless you make the most important decision of your life.
 * Prove to yourself that you have the strength and the courage to be free.
 * Join...the Helldivers.
 *  Become part of an elite peacekeeping force!
 * See exotic new lifeforms.
 * And spread Managed Democracy throughout the galaxy.
 * Become a HERO.
 * Become a LEGEND.
 * Become a Helldiver!
 */
pub mod api;
pub mod application_error;
pub mod data;
pub mod domain;
pub mod event;
pub mod internal;
pub mod macros;
pub mod utils;

use phf::{Set, phf_set};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Unverified;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Verified;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MicroService {
    User,
    Geo,
    Train,
    Hotel,
    Dish,
    Order,
    ObjectStorage,
    Message,
}

impl MicroService {
    /// 返回枚举成员的小写字符串表示。
    pub fn name(&self) -> &'static str {
        match self {
            MicroService::User => "user",
            MicroService::Geo => "geo",
            MicroService::Train => "train",
            MicroService::Hotel => "hotel",
            MicroService::Dish => "dish",
            MicroService::Order => "order",
            MicroService::ObjectStorage => "object_storage",
            MicroService::Message => "message",
        }
    }
}

// 手动实现 Display 特征
impl std::fmt::Display for MicroService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 直接调用辅助方法，将小写名称写入格式化器
        write!(f, "{}", self.name())
    }
}

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

pub const API_FORBIDDEN_CODE: u32 = 403;
pub const API_SUCCESS_MESSAGE: &str = "For Super Earth!";

pub const API_FORBIDDEN_MESSAGE_TEMPLATE: &str =
    "Sorry, but this was meant to be a private game: {}";
pub const API_BAD_REQUEST_MESSAGE_TEMPLATE: &str = "{}";

pub const API_NOT_FOUND_MESSAGE_TEMPLATE: &str =
    "Sorry, but this was meant to be a private game: {}";

pub const API_INTERNAL_SERVER_ERROR_MESSAGE: &str =
    "Multiplayer Session Ended: an internal server error has occurred";
