#![feature(globs)]
#![feature(phase)]

#[phase(plugin)]
extern crate lazy_static;

pub use littlewing::cmd;

pub mod littlewing;
