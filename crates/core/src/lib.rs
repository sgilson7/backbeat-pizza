//! tool_core: every rule of Backbeat Pizza.
//!
//! Builder requirement: this crate performs no input or output. It takes
//! values and returns values, so `cargo test` reaches every rule without a
//! browser. `tests/boundary.rs` enforces that, along with the dependency
//! allowlist and the reference ban on floats, hash maps and clocks.
#![allow(clippy::needless_range_loop, clippy::collapsible_match)]

pub mod content;
pub mod fx;
pub mod game;
pub mod geom;
pub mod glossary;
pub mod levels;
pub mod mesh;
pub mod puzzle;
pub mod render;
pub mod rng;
pub mod scene;
pub mod solver;
