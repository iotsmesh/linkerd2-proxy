#![deny(warnings, rust_2018_idioms)]

mod accept;
pub mod resolve;

pub use self::{accept::Accept, resolve::Resolve, resolve::Update};
