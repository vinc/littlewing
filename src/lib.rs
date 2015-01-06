#![feature(globs)]
#![feature(phase)]

#[phase(plugin)]
extern crate lazy_static;

pub use littlewing::protocols;
pub use littlewing::protocols::cli;

pub mod littlewing;
