//! Minimal subset of odbc-sys types and constants used by this driver.
//! Allows building with zero external crates.

#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::os::raw::{c_void, c_short, c_ushort, c_int, c_longlong};

pub type Handle = *mut c_void;
pub type HandleType = c_short;
pub type SQLSMALLINT = c_short;
pub type SQLUSMALLINT = c_ushort;
pub type SQLINTEGER = c_int;
pub type SQLUINTEGER = u32;
pub type SQLLEN = isize;
pub type SQLULEN = usize;
pub type SQLPOINTER = *mut c_void;
pub type SQLCHAR = u8;
pub type SQLHWND = *mut c_void;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SqlReturn(pub i16);

impl SqlReturn {
    pub const SUCCESS: SqlReturn = SqlReturn(0);
    pub const SUCCESS_WITH_INFO: SqlReturn = SqlReturn(1);
    pub const ERROR: SqlReturn = SqlReturn(-1);
    pub const INVALID_HANDLE: SqlReturn = SqlReturn(-2);
    pub const NO_DATA: SqlReturn = SqlReturn(100);
}

// Allow comparing with the associated constants used as free names below
pub use SqlReturn as SqlReturnType;

pub const SQL_SUCCESS: SqlReturn = SqlReturn(0);
pub const SQL_SUCCESS_WITH_INFO: SqlReturn = SqlReturn(1);
pub const SQL_ERROR: SqlReturn = SqlReturn(-1);
pub const SQL_INVALID_HANDLE: SqlReturn = SqlReturn(-2);
pub const SQL_NO_DATA: SqlReturn = SqlReturn(100);

pub const SQL_HANDLE_ENV: HandleType = 1;
pub const SQL_HANDLE_DBC: HandleType = 2;
pub const SQL_HANDLE_STMT: HandleType = 3;

pub const SQL_NTS: SQLSMALLINT = -3;

pub const SQL_FETCH_NEXT: SQLSMALLINT = 1;

pub const SQL_ATTR_ROW_ARRAY_SIZE: SQLINTEGER = 27;
pub const SQL_ATTR_AUTOCOMMIT: SQLINTEGER = 102;
pub const SQL_AUTOCOMMIT_ON: SQLUINTEGER = 1;

pub const SQL_INTEGER: i16 = 4;
pub const SQL_BIGINT: i16 = -5;
pub const SQL_DOUBLE: i16 = 8;
pub const SQL_VARCHAR: i16 = 12;
pub const SQL_BIT: i16 = -7;

pub const SQL_NULLABLE: SQLSMALLINT = 1;
pub const SQL_NO_NULLS: SQLSMALLINT = 0;

pub const SQL_DESC_NAME: SQLUSMALLINT = 1011;
pub const SQL_DESC_TYPE: SQLUSMALLINT = 1002;
pub const SQL_DESC_NULLABLE: SQLUSMALLINT = 1008;
pub const SQL_DESC_LENGTH: SQLUSMALLINT = 1003;
pub const SQL_DESC_PRECISION: SQLUSMALLINT = 1005;
pub const SQL_DESC_SCALE: SQLUSMALLINT = 1006;
pub const SQL_DESC_DISPLAY_SIZE: SQLUSMALLINT = 6;
