//! ML-ODBC Driver – exposes ML model inference as an ODBC data source.
//! Zero external dependencies (vendored ODBC constants).

#![allow(non_snake_case)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::missing_safety_doc)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod odbc_sys;
mod handles;
mod diag;
mod sql_parser;
mod backend;
mod driver;

pub use driver::*;
