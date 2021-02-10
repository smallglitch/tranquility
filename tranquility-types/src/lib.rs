#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::struct_excessive_bools, clippy::must_use_candidate)]

pub mod activitypub;
#[cfg(feature = "mastodon")]
pub mod mastodon;
pub mod webfinger;

#[cfg(test)]
mod tests;
