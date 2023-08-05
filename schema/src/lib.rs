pub mod definition;

mod schema;
pub use schema::*;

/// If you want to explore the schema and all the class relations,
/// please use https://github.com/neverlosecc/source2sdk.
///
/// This crate only provides the class member offsets.
pub mod offsets {
    #![allow(dead_code, unused, non_upper_case_globals, non_snake_case)]
    include!(concat!(env!("OUT_DIR"), "/offsets.rs"));
}