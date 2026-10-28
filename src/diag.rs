//! ODBC Diagnostics

use crate::odbc_sys::{SqlReturn, SQL_SUCCESS, SQL_SUCCESS_WITH_INFO};

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub sql_state: [u8; 6],
    pub native_error: i32,
    pub message: String,
}

impl Diagnostic {
    pub fn new(sql_state: &str, native: i32, message: impl Into<String>) -> Self {
        let mut state = [0u8; 6];
        let bytes = sql_state.as_bytes();
        let len = bytes.len().min(5);
        state[..len].copy_from_slice(&bytes[..len]);
        Self {
            sql_state: state,
            native_error: native,
            message: message.into(),
        }
    }

    pub fn general_error(msg: impl Into<String>) -> Self {
        Self::new("HY000", -1, msg)
    }

    pub fn function_sequence_error() -> Self {
        Self::new("HY010", 0, "Function sequence error")
    }
}

pub unsafe fn write_diag_rec(
    diag: &Diagnostic,
    sql_state: *mut u8,
    native_error: *mut i32,
    message_text: *mut u8,
    buffer_length: i16,
    text_length: *mut i16,
) -> SqlReturn {
    if !sql_state.is_null() {
        std::ptr::copy_nonoverlapping(diag.sql_state.as_ptr(), sql_state, 6);
    }
    if !native_error.is_null() {
        *native_error = diag.native_error;
    }
    let msg_bytes = diag.message.as_bytes();
    let needed = msg_bytes.len() as i16;
    if !text_length.is_null() {
        *text_length = needed;
    }
    if message_text.is_null() || buffer_length <= 0 {
        return if buffer_length == 0 {
            SQL_SUCCESS_WITH_INFO
        } else {
            SQL_SUCCESS
        };
    }
    let copy_len = (buffer_length as usize - 1).min(msg_bytes.len());
    std::ptr::copy_nonoverlapping(msg_bytes.as_ptr(), message_text, copy_len);
    *message_text.add(copy_len) = 0;
    if copy_len < msg_bytes.len() {
        SQL_SUCCESS_WITH_INFO
    } else {
        SQL_SUCCESS
    }
}
