#![feature(array_try_from_fn)]
#![feature(sync_unsafe_cell)]

pub mod definition;

mod entity;
pub use entity::*;

mod lazy;
pub use lazy::*;

// FIXME: Correct type here. Is it a 3xf32, 4xf32 or 3xu8 or 4xu8
pub type Color = u8;

pub mod cs2 {
    #![allow(
        dead_code,
        unused,
        non_upper_case_globals,
        non_snake_case,
        non_camel_case_types
    )]
    include!(concat!(env!("OUT_DIR"), "/cs2_schema.rs"));
}
