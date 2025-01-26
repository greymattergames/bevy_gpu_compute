#![feature(f16)]

#[allow(clippy::manual_non_exhaustive)]
// cannot use #[non_exhaustive] in a macro, and we want to force users even intra-crate to use the constructors for the matrix and vector types
mod rust;
pub mod wgsl;

pub use rust::*;
pub mod wgsl_helpers {
    pub use super::wgsl::user_facing_api::*;
}
