//! Integration tests calling ODBC C entry points directly.

use ml_odbc_driver::odbc_sys::*;
use ml_odbc_driver::*;
use std::ptr;
use std::ffi::CString;

fn cstr(s: &str) -> CString {
    CString::new(s).unwrap()
}

#[test]
fn alloc_env_dbc_stmt_and_free() {
    unsafe {
        let mut env: Handle = ptr::null_mut();
        assert_eq!(SQLAllocHandle(SQL_HANDLE_ENV, ptr::null_mut(), &mut env), SQL_SUCCESS);
        assert!(!env.is_null());

        let mut dbc: Handle = ptr::null_mut();
        assert_eq!(SQLAllocHandle(SQL_HANDLE_DBC, env, &mut dbc), SQL_SUCCESS);

        let mut stmt: Handle = ptr::null_mut();
        assert_eq!(SQLAllocHandle(SQL_HANDLE_STMT, dbc, &mut stmt), SQL_SUCCESS);

        assert_eq!(SQLFreeHandle(SQL_HANDLE_STMT, stmt), SQL_SUCCESS);
        assert_eq!(SQLFreeHandle(SQL_HANDLE_DBC, dbc), SQL_SUCCESS);
        assert_eq!(SQLFreeHandle(SQL_HANDLE_ENV, env), SQL_SUCCESS);
    }
}

#[test]
fn connect_exec_fetch_mock() {
    unsafe {
        let mut env: Handle = ptr::null_mut();
        assert_eq!(SQLAllocHandle(SQL_HANDLE_ENV, ptr::null_mut(), &mut env), SQL_SUCCESS);

        let mut dbc: Handle = ptr::null_mut();
        assert_eq!(SQLAllocHandle(SQL_HANDLE_DBC, env, &mut dbc), SQL_SUCCESS);

        let conn_str = cstr("Driver={ML ODBC Driver};Backend=mock;");
        let mut out_len: SQLSMALLINT = 0;
        let rc = SQLDriverConnect(
            dbc,
            ptr::null_mut(),
            conn_str.as_ptr() as *const SQLCHAR,
            SQL_NTS,
            ptr::null_mut(),
            0,
            &mut out_len,
            0,
        );
        assert_eq!(rc, SQL_SUCCESS, "SQLDriverConnect failed");

        let mut stmt: Handle = ptr::null_mut();
        assert_eq!(SQLAllocHandle(SQL_HANDLE_STMT, dbc, &mut stmt), SQL_SUCCESS);

        let sql = cstr("SELECT * FROM iris_model LIMIT 3");
        let rc = SQLExecDirect(stmt, sql.as_ptr() as *const SQLCHAR, SQL_NTS as SQLINTEGER);
        assert_eq!(rc, SQL_SUCCESS, "SQLExecDirect failed");

        let mut col_count: SQLSMALLINT = 0;
        assert_eq!(SQLNumResultCols(stmt, &mut col_count), SQL_SUCCESS);
        assert!(col_count >= 4, "expected at least 4 columns, got {}", col_count);

        let mut rows = 0;
        loop {
            let rc = SQLFetch(stmt);
            if rc == SQL_NO_DATA {
                break;
            }
            assert_eq!(rc, SQL_SUCCESS);
            rows += 1;

            let mut buf = [0u8; 64];
            let mut ind: SQLLEN = 0;
            let _ = SQLGetData(
                stmt,
                1,
                1,
                buf.as_mut_ptr() as SQLPOINTER,
                buf.len() as SQLLEN,
                &mut ind,
            );
        }
        assert_eq!(rows, 3, "LIMIT 3 should return 3 rows");

        assert_eq!(SQLFreeHandle(SQL_HANDLE_STMT, stmt), SQL_SUCCESS);
        assert_eq!(SQLDisconnect(dbc), SQL_SUCCESS);
        assert_eq!(SQLFreeHandle(SQL_HANDLE_DBC, dbc), SQL_SUCCESS);
        assert_eq!(SQLFreeHandle(SQL_HANDLE_ENV, env), SQL_SUCCESS);
    }
}

#[test]
fn sql_tables_returns_models() {
    unsafe {
        let mut env: Handle = ptr::null_mut();
        SQLAllocHandle(SQL_HANDLE_ENV, ptr::null_mut(), &mut env);
        let mut dbc: Handle = ptr::null_mut();
        SQLAllocHandle(SQL_HANDLE_DBC, env, &mut dbc);

        let conn_str = cstr("Backend=mock;");
        let mut out_len: SQLSMALLINT = 0;
        assert_eq!(
            SQLDriverConnect(
                dbc,
                ptr::null_mut(),
                conn_str.as_ptr() as *const SQLCHAR,
                SQL_NTS,
                ptr::null_mut(),
                0,
                &mut out_len,
                0,
            ),
            SQL_SUCCESS
        );

        let mut stmt: Handle = ptr::null_mut();
        SQLAllocHandle(SQL_HANDLE_STMT, dbc, &mut stmt);

        assert_eq!(
            SQLTables(
                stmt,
                ptr::null(),
                0,
                ptr::null(),
                0,
                ptr::null(),
                0,
                ptr::null(),
                0,
            ),
            SQL_SUCCESS
        );

        let mut tables = Vec::new();
        loop {
            if SQLFetch(stmt) != SQL_SUCCESS {
                break;
            }
            let mut buf = [0u8; 128];
            let mut ind: SQLLEN = 0;
            SQLGetData(
                stmt,
                3,
                1,
                buf.as_mut_ptr() as SQLPOINTER,
                buf.len() as SQLLEN,
                &mut ind,
            );
            if ind > 0 {
                let end = (ind as usize).min(buf.len());
                let name = String::from_utf8_lossy(&buf[..end])
                    .trim_end_matches('\0')
                    .to_string();
                tables.push(name);
            }
        }

        assert!(
            tables.iter().any(|t| t.contains("iris_model")),
            "tables={:?}",
            tables
        );
        assert!(
            tables.iter().any(|t| t.contains("churn_model")),
            "tables={:?}",
            tables
        );

        SQLFreeHandle(SQL_HANDLE_STMT, stmt);
        SQLDisconnect(dbc);
        SQLFreeHandle(SQL_HANDLE_DBC, dbc);
        SQLFreeHandle(SQL_HANDLE_ENV, env);
    }
}

#[test]
fn where_filter_works() {
    unsafe {
        let mut env: Handle = ptr::null_mut();
        SQLAllocHandle(SQL_HANDLE_ENV, ptr::null_mut(), &mut env);
        let mut dbc: Handle = ptr::null_mut();
        SQLAllocHandle(SQL_HANDLE_DBC, env, &mut dbc);

        let conn_str = cstr("Backend=mock;");
        let mut out_len: SQLSMALLINT = 0;
        SQLDriverConnect(
            dbc,
            ptr::null_mut(),
            conn_str.as_ptr() as *const SQLCHAR,
            SQL_NTS,
            ptr::null_mut(),
            0,
            &mut out_len,
            0,
        );

        let mut stmt: Handle = ptr::null_mut();
        SQLAllocHandle(SQL_HANDLE_STMT, dbc, &mut stmt);

        let sql = cstr(
            "SELECT predicted_class, confidence FROM iris_model WHERE confidence > 0.9",
        );
        assert_eq!(
            SQLExecDirect(stmt, sql.as_ptr() as *const SQLCHAR, SQL_NTS as SQLINTEGER),
            SQL_SUCCESS
        );

        let mut rows = 0;
        while SQLFetch(stmt) == SQL_SUCCESS {
            rows += 1;
        }
        assert!(rows >= 2, "expected filtered rows, got {}", rows);

        SQLFreeHandle(SQL_HANDLE_STMT, stmt);
        SQLDisconnect(dbc);
        SQLFreeHandle(SQL_HANDLE_DBC, dbc);
        SQLFreeHandle(SQL_HANDLE_ENV, env);
    }
}
