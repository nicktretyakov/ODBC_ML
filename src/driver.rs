//! ODBC Driver Manager entry points.
//!
//! All functions follow the classic ODBC C signature and return SqlReturn.

use std::ptr;
use crate::odbc_sys::*;

use crate::handles::*;
use crate::diag::*;
use crate::sql_parser::parse_sql;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn set_diag(handle_id: usize, diag: Diagnostic) {
    if let Some(h) = get_handle(handle_id) {
        let mut g = h.lock().unwrap();
        g.diagnostics.clear();
        g.diagnostics.push(diag);
    }
}

fn push_diag(handle_id: usize, diag: Diagnostic) {
    if let Some(h) = get_handle(handle_id) {
        let mut g = h.lock().unwrap();
        g.diagnostics.push(diag);
    }
}

fn clear_diag(handle_id: usize) {
    if let Some(h) = get_handle(handle_id) {
        h.lock().unwrap().diagnostics.clear();
    }
}

// ---------------------------------------------------------------------------
// SQLAllocHandle
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn SQLAllocHandle(
    handle_type: HandleType,
    input_handle: Handle,
    output_handle: *mut Handle,
) -> SqlReturn {
    if output_handle.is_null() {
        return SQL_ERROR;
    }

    let kind = match handle_type {
        SQL_HANDLE_ENV => HandleKind::Env,
        SQL_HANDLE_DBC => HandleKind::Dbc,
        SQL_HANDLE_STMT => HandleKind::Stmt,
        _ => return SQL_ERROR,
    };

    let parent = if handle_type == SQL_HANDLE_ENV {
        None
    } else {
        match handle_from_ptr(input_handle) {
            Some(id) => Some(id),
            None => return SQL_INVALID_HANDLE,
        }
    };

    match alloc_handle(kind, parent) {
        Ok(id) => {
            *output_handle = ptr_from_id(id);
            ();
            SQL_SUCCESS
        }
        Err(_) => SQL_ERROR,
    }
}

// ---------------------------------------------------------------------------
// SQLFreeHandle
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn SQLFreeHandle(
    handle_type: HandleType,
    handle: Handle,
) -> SqlReturn {
    let id = match handle_from_ptr(handle) {
        Some(id) => id,
        None => return SQL_INVALID_HANDLE,
    };
    ();
    free_handle(id)
}

// ---------------------------------------------------------------------------
// SQLDriverConnect
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn SQLDriverConnect(
    connection_handle: Handle,
    _window_handle: SQLHWND,
    in_connection_string: *const SQLCHAR,
    string_length1: SQLSMALLINT,
    out_connection_string: *mut SQLCHAR,
    buffer_length: SQLSMALLINT,
    string_length2_ptr: *mut SQLSMALLINT,
    _driver_completion: SQLUSMALLINT,
) -> SqlReturn {
    let id = match handle_from_ptr(connection_handle) {
        Some(id) => id,
        None => return SQL_INVALID_HANDLE,
    };
    clear_diag(id);

    let conn_str = if in_connection_string.is_null() {
        String::new()
    } else {
        let len = if string_length1 == SQL_NTS {
            // find null
            let mut l = 0;
            while *in_connection_string.add(l) != 0 {
                l += 1;
            }
            l
        } else {
            string_length1 as usize
        };
        let slice = std::slice::from_raw_parts(in_connection_string, len);
        String::from_utf8_lossy(slice).to_string()
    };

    ();

    let backend = match create_backend(&conn_str) {
        Ok(b) => b,
        Err(e) => {
            set_diag(id, Diagnostic::general_error(e));
            return SQL_ERROR;
        }
    };

    if let Some(h) = get_handle(id) {
        let mut g = h.lock().unwrap();
        if g.kind != HandleKind::Dbc {
            return SQL_INVALID_HANDLE;
        }
        g.connected = true;
        g.connection_string = conn_str.clone();
        g.backend = Some(backend);
    }

    // Copy connection string back if buffer provided
    if !out_connection_string.is_null() && buffer_length > 0 {
        let bytes = conn_str.as_bytes();
        let copy_len = (buffer_length as usize - 1).min(bytes.len());
        ptr::copy_nonoverlapping(bytes.as_ptr(), out_connection_string, copy_len);
        *out_connection_string.add(copy_len) = 0;
        if !string_length2_ptr.is_null() {
            *string_length2_ptr = bytes.len() as SQLSMALLINT;
        }
    }

    SQL_SUCCESS
}

// ---------------------------------------------------------------------------
// SQLDisconnect
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn SQLDisconnect(connection_handle: Handle) -> SqlReturn {
    let id = match handle_from_ptr(connection_handle) {
        Some(id) => id,
        None => return SQL_INVALID_HANDLE,
    };
    if let Some(h) = get_handle(id) {
        let mut g = h.lock().unwrap();
        g.connected = false;
        g.backend = None;
    }
    SQL_SUCCESS
}

// ---------------------------------------------------------------------------
// SQLExecDirect / SQLPrepare / SQLExecute
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn SQLExecDirect(
    statement_handle: Handle,
    statement_text: *const SQLCHAR,
    text_length: SQLINTEGER,
) -> SqlReturn {
    let id = match handle_from_ptr(statement_handle) {
        Some(id) => id,
        None => return SQL_INVALID_HANDLE,
    };
    clear_diag(id);

    let sql = read_sql(statement_text, text_length);
    ();

    execute_sql(id, &sql)
}

#[no_mangle]
pub unsafe extern "C" fn SQLPrepare(
    statement_handle: Handle,
    statement_text: *const SQLCHAR,
    text_length: SQLINTEGER,
) -> SqlReturn {
    let id = match handle_from_ptr(statement_handle) {
        Some(id) => id,
        None => return SQL_INVALID_HANDLE,
    };
    clear_diag(id);

    let sql = read_sql(statement_text, text_length);
    ();

    if let Some(h) = get_handle(id) {
        let mut g = h.lock().unwrap();
        g.sql = Some(sql);
        g.prepared = true;
        g.executed = false;
        g.result_cols.clear();
        g.result_rows.clear();
        g.current_row = -1;
    }
    SQL_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn SQLExecute(statement_handle: Handle) -> SqlReturn {
    let id = match handle_from_ptr(statement_handle) {
        Some(id) => id,
        None => return SQL_INVALID_HANDLE,
    };
    clear_diag(id);

    let sql = {
        let h = match get_handle(id) {
            Some(h) => h,
            None => return SQL_INVALID_HANDLE,
        };
        let g = h.lock().unwrap();
        if !g.prepared {
            set_diag(id, Diagnostic::function_sequence_error());
            return SQL_ERROR;
        }
        g.sql.clone().unwrap_or_default()
    };

    execute_sql(id, &sql)
}

fn read_sql(ptr: *const SQLCHAR, len: SQLINTEGER) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let l = if len == SQL_NTS as SQLINTEGER {
        let mut i = 0;
        unsafe {
            while *ptr.add(i) != 0 {
                i += 1;
            }
        }
        i
    } else {
        len as usize
    };
    let slice = unsafe { std::slice::from_raw_parts(ptr, l) };
    String::from_utf8_lossy(slice).to_string()
}

fn execute_sql(stmt_id: usize, sql: &str) -> SqlReturn {
    // Find parent connection to get the backend
    let parent_id = {
        let h = match get_handle(stmt_id) {
            Some(h) => h,
            None => return SQL_INVALID_HANDLE,
        };
        let parent = h.lock().unwrap().parent;
        parent
    };

    let parent_id = match parent_id {
        Some(p) => p,
        None => {
            set_diag(stmt_id, Diagnostic::general_error("statement has no parent connection"));
            return SQL_ERROR;
        }
    };

    let backend = {
        let h = match get_handle(parent_id) {
            Some(h) => h,
            None => return SQL_INVALID_HANDLE,
        };
        let g = h.lock().unwrap();
        if !g.connected {
            set_diag(stmt_id, Diagnostic::general_error("not connected"));
            return SQL_ERROR;
        }
        // We need to clone the capability; for MVP we re-create from connection string
        // or keep a shared Arc. Here we just re-create for simplicity.
        match create_backend(&g.connection_string) {
            Ok(b) => b,
            Err(e) => {
                set_diag(stmt_id, Diagnostic::general_error(e));
                return SQL_ERROR;
            }
        }
    };

    let parsed = match parse_sql(sql) {
        Ok(p) => p,
        Err(e) => {
            set_diag(stmt_id, Diagnostic::general_error(e.to_string()));
            return SQL_ERROR;
        }
    };

    match backend.execute(&parsed) {
        Ok((cols, rows)) => {
            if let Some(h) = get_handle(stmt_id) {
                let mut g = h.lock().unwrap();
                g.result_cols = cols;
                g.result_rows = rows;
                g.current_row = -1;
                g.executed = true;
                g.prepared = true;
            }
            SQL_SUCCESS
        }
        Err(e) => {
            set_diag(stmt_id, Diagnostic::general_error(e));
            SQL_ERROR
        }
    }
}

// ---------------------------------------------------------------------------
// SQLFetch / SQLFetchScroll
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn SQLFetch(statement_handle: Handle) -> SqlReturn {
    sql_fetch_scroll_impl(statement_handle, SQL_FETCH_NEXT, 0)
}

#[no_mangle]
pub unsafe extern "C" fn SQLFetchScroll(
    statement_handle: Handle,
    fetch_orientation: SQLSMALLINT,
    fetch_offset: SQLLEN,
) -> SqlReturn {
    sql_fetch_scroll_impl(statement_handle, fetch_orientation, fetch_offset)
}

fn sql_fetch_scroll_impl(
    statement_handle: Handle,
    orientation: SQLSMALLINT,
    _offset: SQLLEN,
) -> SqlReturn {
    let id = match handle_from_ptr(statement_handle) {
        Some(id) => id,
        None => return SQL_INVALID_HANDLE,
    };

    let h = match get_handle(id) {
        Some(h) => h,
        None => return SQL_INVALID_HANDLE,
    };
    let mut g = h.lock().unwrap();

    if !g.executed {
        set_diag(id, Diagnostic::function_sequence_error());
        return SQL_ERROR;
    }

    if orientation != SQL_FETCH_NEXT {
        set_diag(id, Diagnostic::general_error("only SQL_FETCH_NEXT is supported in MVP"));
        return SQL_ERROR;
    }

    g.current_row += 1;
    if g.current_row as usize >= g.result_rows.len() {
        g.current_row = g.result_rows.len() as isize; // past end
        return SQL_NO_DATA;
    }

    // If columns are bound, copy data now
    let row_idx = g.current_row as usize;
    let row = &g.result_rows[row_idx];

    for (col_num, bound) in g.bound_cols.iter() {
        let col_idx = (*col_num as usize).saturating_sub(1);
        if col_idx >= row.len() {
            continue;
        }
        let val = &row[col_idx];
        unsafe {
            write_bound_value(val, bound);
        }
    }

    SQL_SUCCESS
}

unsafe fn write_bound_value(val: &Option<String>, bound: &BoundColumn) {
    if val.is_none() {
        if !bound.strlen_or_ind.is_null() {
            *bound.strlen_or_ind = -1; // SQL_NULL_DATA
        }
        return;
    }
    let s = val.as_ref().unwrap();
    match bound.target_type {
        SQL_INTEGER | SQL_BIGINT => {
            if let Ok(n) = s.parse::<i64>() {
                if !bound.target_value.is_null() {
                    *(bound.target_value as *mut i64) = n;
                }
                if !bound.strlen_or_ind.is_null() {
                    *bound.strlen_or_ind = std::mem::size_of::<i64>() as isize;
                }
            }
        }
        SQL_DOUBLE => {
            if let Ok(n) = s.parse::<f64>() {
                if !bound.target_value.is_null() {
                    *(bound.target_value as *mut f64) = n;
                }
                if !bound.strlen_or_ind.is_null() {
                    *bound.strlen_or_ind = std::mem::size_of::<f64>() as isize;
                }
            }
        }
        SQL_BIT => {
            let b: u8 = if s == "1" || s.eq_ignore_ascii_case("true") { 1 } else { 0 };
            if !bound.target_value.is_null() {
                *(bound.target_value as *mut u8) = b;
            }
            if !bound.strlen_or_ind.is_null() {
                *bound.strlen_or_ind = 1;
            }
        }
        _ => {
            // treat as string
            let bytes = s.as_bytes();
            let max = if bound.buffer_length > 0 {
                (bound.buffer_length as usize).saturating_sub(1)
            } else {
                bytes.len()
            };
            let copy_len = bytes.len().min(max);
            if !bound.target_value.is_null() {
                ptr::copy_nonoverlapping(bytes.as_ptr(), bound.target_value as *mut u8, copy_len);
                *((bound.target_value as *mut u8).add(copy_len)) = 0;
            }
            if !bound.strlen_or_ind.is_null() {
                *bound.strlen_or_ind = bytes.len() as isize;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SQLGetData
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn SQLGetData(
    statement_handle: Handle,
    col_or_param_num: SQLUSMALLINT,
    target_type: SQLSMALLINT,
    target_value: SQLPOINTER,
    buffer_length: SQLLEN,
    strlen_or_ind: *mut SQLLEN,
) -> SqlReturn {
    let id = match handle_from_ptr(statement_handle) {
        Some(id) => id,
        None => return SQL_INVALID_HANDLE,
    };

    let h = match get_handle(id) {
        Some(h) => h,
        None => return SQL_INVALID_HANDLE,
    };
    let g = h.lock().unwrap();

    if g.current_row < 0 || g.current_row as usize >= g.result_rows.len() {
        return SQL_NO_DATA;
    }

    let col_idx = (col_or_param_num as usize).saturating_sub(1);
    let row = &g.result_rows[g.current_row as usize];
    if col_idx >= row.len() {
        set_diag(id, Diagnostic::general_error("invalid column number"));
        return SQL_ERROR;
    }

    let val = &row[col_idx];
    let bound = BoundColumn {
        target_type,
        target_value,
        buffer_length,
        strlen_or_ind,
    };
    write_bound_value(val, &bound);
    SQL_SUCCESS
}

// ---------------------------------------------------------------------------
// SQLBindCol
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn SQLBindCol(
    statement_handle: Handle,
    column_number: SQLUSMALLINT,
    target_type: SQLSMALLINT,
    target_value: SQLPOINTER,
    buffer_length: SQLLEN,
    strlen_or_ind: *mut SQLLEN,
) -> SqlReturn {
    let id = match handle_from_ptr(statement_handle) {
        Some(id) => id,
        None => return SQL_INVALID_HANDLE,
    };

    if let Some(h) = get_handle(id) {
        let mut g = h.lock().unwrap();
        if target_value.is_null() {
            g.bound_cols.remove(&column_number);
        } else {
            g.bound_cols.insert(
                column_number,
                BoundColumn {
                    target_type,
                    target_value,
                    buffer_length,
                    strlen_or_ind,
                },
            );
        }
    }
    SQL_SUCCESS
}

// ---------------------------------------------------------------------------
// Metadata: SQLNumResultCols, SQLDescribeCol, SQLColAttribute, SQLRowCount
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn SQLNumResultCols(
    statement_handle: Handle,
    column_count: *mut SQLSMALLINT,
) -> SqlReturn {
    let id = match handle_from_ptr(statement_handle) {
        Some(id) => id,
        None => return SQL_INVALID_HANDLE,
    };
    if column_count.is_null() {
        return SQL_ERROR;
    }
    if let Some(h) = get_handle(id) {
        let g = h.lock().unwrap();
        *column_count = g.result_cols.len() as SQLSMALLINT;
        return SQL_SUCCESS;
    }
    SQL_INVALID_HANDLE
}

#[no_mangle]
pub unsafe extern "C" fn SQLDescribeCol(
    statement_handle: Handle,
    column_number: SQLUSMALLINT,
    column_name: *mut SQLCHAR,
    buffer_length: SQLSMALLINT,
    name_length: *mut SQLSMALLINT,
    data_type: *mut SQLSMALLINT,
    column_size: *mut SQLULEN,
    decimal_digits: *mut SQLSMALLINT,
    nullable: *mut SQLSMALLINT,
) -> SqlReturn {
    let id = match handle_from_ptr(statement_handle) {
        Some(id) => id,
        None => return SQL_INVALID_HANDLE,
    };

    let h = match get_handle(id) {
        Some(h) => h,
        None => return SQL_INVALID_HANDLE,
    };
    let g = h.lock().unwrap();
    let idx = (column_number as usize).saturating_sub(1);
    if idx >= g.result_cols.len() {
        set_diag(id, Diagnostic::general_error("invalid column number"));
        return SQL_ERROR;
    }
    let col = &g.result_cols[idx];

    if !column_name.is_null() && buffer_length > 0 {
        let bytes = col.name.as_bytes();
        let copy = (buffer_length as usize - 1).min(bytes.len());
        ptr::copy_nonoverlapping(bytes.as_ptr(), column_name, copy);
        *column_name.add(copy) = 0;
        if !name_length.is_null() {
            *name_length = bytes.len() as SQLSMALLINT;
        }
    }
    if !data_type.is_null() {
        *data_type = col.sql_type;
    }
    if !column_size.is_null() {
        *column_size = col.display_size as SQLULEN;
    }
    if !decimal_digits.is_null() {
        *decimal_digits = col.scale;
    }
    if !nullable.is_null() {
        *nullable = if col.nullable { SQL_NULLABLE } else { SQL_NO_NULLS };
    }
    SQL_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn SQLColAttribute(
    statement_handle: Handle,
    column_number: SQLUSMALLINT,
    field_identifier: SQLUSMALLINT,
    character_attribute: SQLPOINTER,
    buffer_length: SQLSMALLINT,
    string_length: *mut SQLSMALLINT,
    numeric_attribute: *mut SQLLEN,
) -> SqlReturn {
    let id = match handle_from_ptr(statement_handle) {
        Some(id) => id,
        None => return SQL_INVALID_HANDLE,
    };
    let h = match get_handle(id) {
        Some(h) => h,
        None => return SQL_INVALID_HANDLE,
    };
    let g = h.lock().unwrap();
    let idx = (column_number as usize).saturating_sub(1);
    if idx >= g.result_cols.len() {
        return SQL_ERROR;
    }
    let col = &g.result_cols[idx];

    match field_identifier {
        x if x == SQL_DESC_NAME as u16 => {
            if !character_attribute.is_null() && buffer_length > 0 {
                let bytes = col.name.as_bytes();
                let copy = (buffer_length as usize - 1).min(bytes.len());
                ptr::copy_nonoverlapping(bytes.as_ptr(), character_attribute as *mut u8, copy);
                *((character_attribute as *mut u8).add(copy)) = 0;
                if !string_length.is_null() {
                    *string_length = bytes.len() as SQLSMALLINT;
                }
            }
        }
        x if x == SQL_DESC_TYPE as u16 => {
            if !numeric_attribute.is_null() {
                *numeric_attribute = col.sql_type as SQLLEN;
            }
        }
        x if x == SQL_DESC_NULLABLE as u16 => {
            if !numeric_attribute.is_null() {
                *numeric_attribute = if col.nullable { SQL_NULLABLE as SQLLEN } else { SQL_NO_NULLS as SQLLEN };
            }
        }
        x if x == SQL_DESC_LENGTH as u16 || x == SQL_DESC_DISPLAY_SIZE as u16 => {
            if !numeric_attribute.is_null() {
                *numeric_attribute = col.display_size as SQLLEN;
            }
        }
        x if x == SQL_DESC_PRECISION as u16 => {
            if !numeric_attribute.is_null() {
                *numeric_attribute = col.precision as SQLLEN;
            }
        }
        x if x == SQL_DESC_SCALE as u16 => {
            if !numeric_attribute.is_null() {
                *numeric_attribute = col.scale as SQLLEN;
            }
        }
        _ => {}
    }
    SQL_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn SQLRowCount(
    statement_handle: Handle,
    row_count: *mut SQLLEN,
) -> SqlReturn {
    let id = match handle_from_ptr(statement_handle) {
        Some(id) => id,
        None => return SQL_INVALID_HANDLE,
    };
    if row_count.is_null() {
        return SQL_ERROR;
    }
    if let Some(h) = get_handle(id) {
        let g = h.lock().unwrap();
        *row_count = g.result_rows.len() as SQLLEN;
        return SQL_SUCCESS;
    }
    SQL_INVALID_HANDLE
}

// ---------------------------------------------------------------------------
// Diagnostics
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn SQLGetDiagRec(
    handle_type: HandleType,
    handle: Handle,
    rec_number: SQLSMALLINT,
    sql_state: *mut SQLCHAR,
    native_error: *mut SQLINTEGER,
    message_text: *mut SQLCHAR,
    buffer_length: SQLSMALLINT,
    text_length: *mut SQLSMALLINT,
) -> SqlReturn {
    if rec_number <= 0 {
        return SQL_ERROR;
    }
    let id = match handle_from_ptr(handle) {
        Some(id) => id,
        None => return SQL_INVALID_HANDLE,
    };
    let h = match get_handle(id) {
        Some(h) => h,
        None => return SQL_INVALID_HANDLE,
    };
    let g = h.lock().unwrap();
    let idx = (rec_number - 1) as usize;
    if idx >= g.diagnostics.len() {
        return SQL_NO_DATA;
    }
    let diag = &g.diagnostics[idx];
    write_diag_rec(diag, sql_state, native_error, message_text, buffer_length, text_length)
}

#[no_mangle]
pub unsafe extern "C" fn SQLGetDiagField(
    _handle_type: HandleType,
    _handle: Handle,
    _rec_number: SQLSMALLINT,
    _diag_identifier: SQLSMALLINT,
    _diag_info: SQLPOINTER,
    _buffer_length: SQLSMALLINT,
    _string_length: *mut SQLSMALLINT,
) -> SqlReturn {
    // Minimal stub – most applications use SQLGetDiagRec
    SQL_SUCCESS
}

// ---------------------------------------------------------------------------
// SQLFreeStmt
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn SQLFreeStmt(
    statement_handle: Handle,
    option: SQLUSMALLINT,
) -> SqlReturn {
    let id = match handle_from_ptr(statement_handle) {
        Some(id) => id,
        None => return SQL_INVALID_HANDLE,
    };
    if let Some(h) = get_handle(id) {
        let mut g = h.lock().unwrap();
        // SQL_CLOSE = 0, SQL_UNBIND = 2, SQL_RESET_PARAMS = 3
        if option == 0 || option == 1 {
            g.current_row = -1;
            g.executed = false;
        }
        if option == 2 {
            g.bound_cols.clear();
        }
    }
    SQL_SUCCESS
}

// ---------------------------------------------------------------------------
// Catalog functions – SQLTables / SQLColumns
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn SQLTables(
    statement_handle: Handle,
    _catalog_name: *const SQLCHAR,
    _name_length1: SQLSMALLINT,
    _schema_name: *const SQLCHAR,
    _name_length2: SQLSMALLINT,
    _table_name: *const SQLCHAR,
    _name_length3: SQLSMALLINT,
    _table_type: *const SQLCHAR,
    _name_length4: SQLSMALLINT,
) -> SqlReturn {
    let id = match handle_from_ptr(statement_handle) {
        Some(id) => id,
        None => return SQL_INVALID_HANDLE,
    };
    clear_diag(id);

    let parent_id = {
        let h = match get_handle(id) {
            Some(h) => h,
            None => return SQL_INVALID_HANDLE,
        };
        let parent = h.lock().unwrap().parent;
        parent
    };
    let parent_id = match parent_id {
        Some(p) => p,
        None => return SQL_ERROR,
    };

    let tables = {
        let h = match get_handle(parent_id) {
            Some(h) => h,
            None => return SQL_INVALID_HANDLE,
        };
        let g = h.lock().unwrap();
        match create_backend(&g.connection_string) {
            Ok(b) => b.tables(),
            Err(e) => {
                set_diag(id, Diagnostic::general_error(e));
                return SQL_ERROR;
            }
        }
    };

    // Standard result set for SQLTables:
    // TABLE_CAT, TABLE_SCHEM, TABLE_NAME, TABLE_TYPE, REMARKS
    let cols = vec![
        ColumnDesc { name: "TABLE_CAT".into(), sql_type: SQL_VARCHAR, nullable: true, precision: 0, scale: 0, display_size: 128 },
        ColumnDesc { name: "TABLE_SCHEM".into(), sql_type: SQL_VARCHAR, nullable: true, precision: 0, scale: 0, display_size: 128 },
        ColumnDesc { name: "TABLE_NAME".into(), sql_type: SQL_VARCHAR, nullable: false, precision: 0, scale: 0, display_size: 128 },
        ColumnDesc { name: "TABLE_TYPE".into(), sql_type: SQL_VARCHAR, nullable: false, precision: 0, scale: 0, display_size: 128 },
        ColumnDesc { name: "REMARKS".into(), sql_type: SQL_VARCHAR, nullable: true, precision: 0, scale: 0, display_size: 254 },
    ];

    let rows: Vec<Vec<Option<String>>> = tables
        .into_iter()
        .map(|t| {
            vec![
                None,
                None,
                Some(t),
                Some("TABLE".into()),
                Some("ML model exposed as table".into()),
            ]
        })
        .collect();

    if let Some(h) = get_handle(id) {
        let mut g = h.lock().unwrap();
        g.result_cols = cols;
        g.result_rows = rows;
        g.current_row = -1;
        g.executed = true;
    }
    SQL_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn SQLColumns(
    statement_handle: Handle,
    _catalog_name: *const SQLCHAR,
    _name_length1: SQLSMALLINT,
    _schema_name: *const SQLCHAR,
    _name_length2: SQLSMALLINT,
    table_name: *const SQLCHAR,
    name_length3: SQLSMALLINT,
    _column_name: *const SQLCHAR,
    _name_length4: SQLSMALLINT,
) -> SqlReturn {
    let id = match handle_from_ptr(statement_handle) {
        Some(id) => id,
        None => return SQL_INVALID_HANDLE,
    };
    clear_diag(id);

    let table = read_sql(table_name, name_length3 as SQLINTEGER);

    let parent_id = {
        let h = match get_handle(id) {
            Some(h) => h,
            None => return SQL_INVALID_HANDLE,
        };
        let parent = h.lock().unwrap().parent;
        parent
    };
    let parent_id = match parent_id {
        Some(p) => p,
        None => return SQL_ERROR,
    };

    let model_cols = {
        let h = match get_handle(parent_id) {
            Some(h) => h,
            None => return SQL_INVALID_HANDLE,
        };
        let g = h.lock().unwrap();
        match create_backend(&g.connection_string) {
            Ok(b) => match b.columns(&table) {
                Ok(c) => c,
                Err(e) => {
                    set_diag(id, Diagnostic::general_error(e));
                    return SQL_ERROR;
                }
            },
            Err(e) => {
                set_diag(id, Diagnostic::general_error(e));
                return SQL_ERROR;
            }
        }
    };

    // Standard SQLColumns result set (simplified)
    let meta_cols = vec![
        ColumnDesc { name: "TABLE_CAT".into(), sql_type: SQL_VARCHAR, nullable: true, precision: 0, scale: 0, display_size: 128 },
        ColumnDesc { name: "TABLE_SCHEM".into(), sql_type: SQL_VARCHAR, nullable: true, precision: 0, scale: 0, display_size: 128 },
        ColumnDesc { name: "TABLE_NAME".into(), sql_type: SQL_VARCHAR, nullable: false, precision: 0, scale: 0, display_size: 128 },
        ColumnDesc { name: "COLUMN_NAME".into(), sql_type: SQL_VARCHAR, nullable: false, precision: 0, scale: 0, display_size: 128 },
        ColumnDesc { name: "DATA_TYPE".into(), sql_type: SQL_INTEGER, nullable: false, precision: 10, scale: 0, display_size: 11 },
        ColumnDesc { name: "TYPE_NAME".into(), sql_type: SQL_VARCHAR, nullable: false, precision: 0, scale: 0, display_size: 128 },
        ColumnDesc { name: "COLUMN_SIZE".into(), sql_type: SQL_INTEGER, nullable: true, precision: 10, scale: 0, display_size: 11 },
        ColumnDesc { name: "NULLABLE".into(), sql_type: SQL_INTEGER, nullable: false, precision: 10, scale: 0, display_size: 11 },
    ];

    let rows: Vec<Vec<Option<String>>> = model_cols
        .into_iter()
        .map(|c| {
            let type_name = match c.sql_type {
                SQL_INTEGER => "INTEGER",
                SQL_BIGINT => "BIGINT",
                SQL_DOUBLE => "DOUBLE",
                SQL_BIT => "BIT",
                _ => "VARCHAR",
            };
            vec![
                None,
                None,
                Some(table.clone()),
                Some(c.name),
                Some(c.sql_type.to_string()),
                Some(type_name.into()),
                Some(c.display_size.to_string()),
                Some(if c.nullable { "1" } else { "0" }.into()),
            ]
        })
        .collect();

    if let Some(h) = get_handle(id) {
        let mut g = h.lock().unwrap();
        g.result_cols = meta_cols;
        g.result_rows = rows;
        g.current_row = -1;
        g.executed = true;
    }
    SQL_SUCCESS
}

// ---------------------------------------------------------------------------
// Statement / Connection attributes (minimal)
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn SQLSetStmtAttr(
    statement_handle: Handle,
    attribute: SQLINTEGER,
    value: SQLPOINTER,
    _string_length: SQLINTEGER,
) -> SqlReturn {
    let id = match handle_from_ptr(statement_handle) {
        Some(id) => id,
        None => return SQL_INVALID_HANDLE,
    };
    if attribute == SQL_ATTR_ROW_ARRAY_SIZE as i32 {
        if let Some(h) = get_handle(id) {
            let mut g = h.lock().unwrap();
            g.row_array_size = value as usize;
            if g.row_array_size == 0 {
                g.row_array_size = 1;
            }
        }
    }
    SQL_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn SQLGetStmtAttr(
    statement_handle: Handle,
    attribute: SQLINTEGER,
    value: SQLPOINTER,
    _buffer_length: SQLINTEGER,
    _string_length: *mut SQLINTEGER,
) -> SqlReturn {
    let id = match handle_from_ptr(statement_handle) {
        Some(id) => id,
        None => return SQL_INVALID_HANDLE,
    };
    if attribute == SQL_ATTR_ROW_ARRAY_SIZE as i32 && !value.is_null() {
        if let Some(h) = get_handle(id) {
            let g = h.lock().unwrap();
            *(value as *mut SQLULEN) = g.row_array_size as SQLULEN;
        }
    }
    SQL_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn SQLSetConnectAttr(
    connection_handle: Handle,
    attribute: SQLINTEGER,
    _value: SQLPOINTER,
    _string_length: SQLINTEGER,
) -> SqlReturn {
    // We are read-only; accept AUTOCOMMIT_ON silently
    let _ = (connection_handle, attribute);
    SQL_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn SQLGetConnectAttr(
    _connection_handle: Handle,
    attribute: SQLINTEGER,
    value: SQLPOINTER,
    _buffer_length: SQLINTEGER,
    _string_length: *mut SQLINTEGER,
) -> SqlReturn {
    if attribute == SQL_ATTR_AUTOCOMMIT as i32 && !value.is_null() {
        unsafe { *(value as *mut SQLUINTEGER) = SQL_AUTOCOMMIT_ON as u32; }
    }
    SQL_SUCCESS
}

// ---------------------------------------------------------------------------
// SQLCancel / SQLGetInfo (stubs)
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn SQLCancel(statement_handle: Handle) -> SqlReturn {
    let _ = statement_handle;
    // MVP: no long-running ops to cancel
    SQL_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn SQLGetInfo(
    connection_handle: Handle,
    info_type: SQLUSMALLINT,
    info_value: SQLPOINTER,
    buffer_length: SQLSMALLINT,
    string_length: *mut SQLSMALLINT,
) -> SqlReturn {
    let _ = connection_handle;
    // Return a few useful values so clients don't immediately fail
    let s = match info_type {
        17 => "ML ODBC Driver",                 // SQL_DRIVER_NAME
        18 => "0.1.0",                          // SQL_DRIVER_VER
        13 => "Y",                              // SQL_SEARCH_PATTERN_ESCAPE (dummy)
        2  => "ML",                             // SQL_DATABASE_NAME
        _ => return SQL_SUCCESS,
    };
    if !info_value.is_null() && buffer_length > 0 {
        let bytes = s.as_bytes();
        let copy = (buffer_length as usize - 1).min(bytes.len());
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), info_value as *mut u8, copy);
            *((info_value as *mut u8).add(copy)) = 0;
        }
        if !string_length.is_null() {
            unsafe { *string_length = bytes.len() as SQLSMALLINT; }
        }
    }
    SQL_SUCCESS
}

// Missing import that some odbc-sys versions need
