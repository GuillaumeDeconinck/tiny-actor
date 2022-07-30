#![doc = include_str!("../README.md")]

pub(crate) mod address;
pub(crate) mod child;
pub(crate) mod inbox;
pub(crate) mod spawning;
pub(crate) mod shared;

pub use address::*;
pub use child::*;
pub use inbox::*;
pub use spawning::{spawn, spawn_many};
pub use actor_channel::*;
pub use shared::*;



