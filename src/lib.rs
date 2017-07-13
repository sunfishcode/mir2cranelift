#![feature(
    custom_attribute,
    link_args,
    rustc_private,
)]

#[macro_use]
extern crate rustc;
extern crate rustc_mir;
extern crate syntax;
extern crate rustc_const_math;
extern crate rustc_data_structures;

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate env_logger;

pub mod error;
pub mod trans;
mod monomorphize;
mod traits;
