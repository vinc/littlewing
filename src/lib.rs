#![feature(globs)]
#![feature(phase)]

#[phase(plugin)]
extern crate lazy_static;

pub use littlewing::game;

pub mod littlewing;
