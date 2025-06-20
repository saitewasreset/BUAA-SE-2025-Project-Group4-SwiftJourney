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
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod models;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Unverified;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Verified;

pub const HOTEL_MAX_BOOKING_DAYS: u32 = 7;
pub const HOTEL_MAX_COMMENT_LENGTH: usize = 8192;

pub const DB_CHUNK_SIZE: usize = 4096;

pub const MAX_CONCURRENT_WEBSOCKET_SESSION_PER_USER: usize = 3;

pub const ORDER_STATUS_UPDATE_INTERVAL_SECONDS: u64 = 60; // seconds
