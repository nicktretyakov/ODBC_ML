//! JNI exports for the Java JDBC driver (com.mlodbc.*).
//!
//! Enabled with `--features jni`.
//! The Java side calls these via System.loadLibrary("ml_odbc_driver").

#![cfg(feature = "jni")]

use jni::objects::{JClass, JString, JObject};
use jni::sys::{jlong, jboolean, jint, JNI_TRUE, JNI_FALSE};
use jni::JNIEnv;
use log::{debug, error, info};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use once_cell::sync::Lazy;

use crate::backend::Backend;
use crate::handles::{create_backend, ColumnDesc};
use crate::sql_parser::parse_sql;

// ---------------------------------------------------------------------------
// Simple registry for JNI-side connections and statements
// (independent of the ODBC handle registry so both APIs can coexist)
// ---------------------------------------------------------------------------

struct JniConnection {
    backend: Box<dyn Backend + Send>,
    conn_str: String,
}

struct JniStatement {
    conn_id: usize,
    // result of the last execute
    cols: Vec<ColumnDesc>,
    rows: Vec<Vec<Option<String>>>,
    current_row: isize,
}

static JNI_CONNS: Lazy<Mutex<HashMap<usize, Arc<Mutex<JniConnection>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static JNI_STMTS: Lazy<Mutex<HashMap<usize, Arc<Mutex<JniStatement>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static JNI_NEXT: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(1));

fn next_id() -> usize {
    let mut g = JNI_NEXT.lock();
    let id = *g;
    *g += 1;
    id
}

fn throw_sql_exception(env: &mut JNIEnv, msg: &str) {
    let _ = env.throw_new("java/sql/SQLException", msg);
}

// ---------------------------------------------------------------------------
// MlOdbcConnection natives
// ---------------------------------------------------------------------------

/// Java: private static native long nativeConnect(String connStr)
#[no_mangle]
pub extern "system" fn Java_com_mlodbc_MlOdbcConnection_nativeConnect<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    conn_str: JString<'local>,
) -> jlong {
    let conn_str: String = match env.get_string(&conn_str) {
        Ok(s) => s.into(),
        Err(e) => {
            throw_sql_exception(&mut env, &format!("invalid connection string: {}", e));
            return 0;
        }
    };

    info!("JNI nativeConnect: {}", conn_str);

    let backend = match create_backend(&conn_str) {
        Ok(b) => b,
        Err(e) => {
            throw_sql_exception(&mut env, &e);
            return 0;
        }
    };

    let id = next_id();
    JNI_CONNS.lock().insert(
        id,
        Arc::new(Mutex::new(JniConnection {
            backend,
            conn_str,
        })),
    );
    id as jlong
}

/// Java: private static native void nativeDisconnect(long handle)
#[no_mangle]
pub extern "system" fn Java_com_mlodbc_MlOdbcConnection_nativeDisconnect(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    let id = handle as usize;
    debug!("JNI nativeDisconnect: {}", id);
    JNI_CONNS.lock().remove(&id);
}

/// Java: private static native long nativeCreateStatement(long connHandle)
#[no_mangle]
pub extern "system" fn Java_com_mlodbc_MlOdbcConnection_nativeCreateStatement<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    conn_handle: jlong,
) -> jlong {
    let conn_id = conn_handle as usize;
    if !JNI_CONNS.lock().contains_key(&conn_id) {
        throw_sql_exception(&mut env, "invalid connection handle");
        return 0;
    }

    let id = next_id();
    JNI_STMTS.lock().insert(
        id,
        Arc::new(Mutex::new(JniStatement {
            conn_id,
            cols: Vec::new(),
            rows: Vec::new(),
            current_row: -1,
        })),
    );
    id as jlong
}

// ---------------------------------------------------------------------------
// MlOdbcStatement natives
// ---------------------------------------------------------------------------

/// Java: private static native long nativeExecute(long stmtHandle, String sql)
/// Returns the same stmt handle on success (ResultSet is backed by the statement).
#[no_mangle]
pub extern "system" fn Java_com_mlodbc_MlOdbcStatement_nativeExecute<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    stmt_handle: jlong,
    sql: JString<'local>,
) -> jlong {
    let stmt_id = stmt_handle as usize;
    let sql: String = match env.get_string(&sql) {
        Ok(s) => s.into(),
        Err(e) => {
            throw_sql_exception(&mut env, &format!("invalid SQL: {}", e));
            return 0;
        }
    };

    debug!("JNI nativeExecute: {}", sql);

    let parsed = match parse_sql(&sql) {
        Ok(p) => p,
        Err(e) => {
            throw_sql_exception(&mut env, &e.to_string());
            return 0;
        }
    };

    // Resolve connection → backend
    let conn_id = {
        let stmts = JNI_STMTS.lock();
        match stmts.get(&stmt_id) {
            Some(s) => s.lock().conn_id,
            None => {
                throw_sql_exception(&mut env, "invalid statement handle");
                return 0;
            }
        }
    };

    let result = {
        let conns = JNI_CONNS.lock();
        let conn = match conns.get(&conn_id) {
            Some(c) => c.clone(),
            None => {
                throw_sql_exception(&mut env, "connection closed");
                return 0;
            }
        };
        let g = conn.lock();
        // Re-create backend from stored connection string is safer for Send
        // but we already have the backend; just call it.
        g.backend.execute(&parsed)
    };

    match result {
        Ok((cols, rows)) => {
            let stmts = JNI_STMTS.lock();
            if let Some(s) = stmts.get(&stmt_id) {
                let mut g = s.lock();
                g.cols = cols;
                g.rows = rows;
                g.current_row = -1;
            }
            stmt_id as jlong // ResultSet uses the same handle
        }
        Err(e) => {
            throw_sql_exception(&mut env, &e);
            0
        }
    }
}

/// Java: private static native void nativeFree(long stmtHandle)
#[no_mangle]
pub extern "system" fn Java_com_mlodbc_MlOdbcStatement_nativeFree(
    _env: JNIEnv,
    _class: JClass,
    stmt_handle: jlong,
) {
    JNI_STMTS.lock().remove(&(stmt_handle as usize));
}

// ---------------------------------------------------------------------------
// MlOdbcResultSet natives
// ---------------------------------------------------------------------------

/// Java: private static native boolean nativeNext(long rsHandle)
#[no_mangle]
pub extern "system" fn Java_com_mlodbc_MlOdbcResultSet_nativeNext(
    _env: JNIEnv,
    _class: JClass,
    rs_handle: jlong,
) -> jboolean {
    let id = rs_handle as usize;
    let stmts = JNI_STMTS.lock();
    let stmt = match stmts.get(&id) {
        Some(s) => s,
        None => return JNI_FALSE,
    };
    let mut g = stmt.lock();
    g.current_row += 1;
    if g.current_row >= 0 && (g.current_row as usize) < g.rows.len() {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

/// Java: private static native String nativeGetString(long rsHandle, int col)
#[no_mangle]
pub extern "system" fn Java_com_mlodbc_MlOdbcResultSet_nativeGetString<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    rs_handle: jlong,
    col: jint,
) -> jni::sys::jstring {
    let id = rs_handle as usize;
    let stmts = JNI_STMTS.lock();
    let stmt = match stmts.get(&id) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    let g = stmt.lock();
    let row_idx = g.current_row as usize;
    let col_idx = (col as usize).saturating_sub(1);
    if row_idx >= g.rows.len() || col_idx >= g.rows[row_idx].len() {
        return std::ptr::null_mut();
    }
    match &g.rows[row_idx][col_idx] {
        Some(s) => env
            .new_string(s)
            .map(|js| js.into_raw())
            .unwrap_or(std::ptr::null_mut()),
        None => std::ptr::null_mut(),
    }
}

/// Java: private static native double nativeGetDouble(long rsHandle, int col)
#[no_mangle]
pub extern "system" fn Java_com_mlodbc_MlOdbcResultSet_nativeGetDouble(
    _env: JNIEnv,
    _class: JClass,
    rs_handle: jlong,
    col: jint,
) -> f64 {
    let id = rs_handle as usize;
    let stmts = JNI_STMTS.lock();
    let stmt = match stmts.get(&id) {
        Some(s) => s,
        None => return 0.0,
    };
    let g = stmt.lock();
    let row_idx = g.current_row as usize;
    let col_idx = (col as usize).saturating_sub(1);
    if row_idx >= g.rows.len() || col_idx >= g.rows[row_idx].len() {
        return 0.0;
    }
    g.rows[row_idx][col_idx]
        .as_ref()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0)
}

/// Java: private static native long nativeGetLong(long rsHandle, int col)
#[no_mangle]
pub extern "system" fn Java_com_mlodbc_MlOdbcResultSet_nativeGetLong(
    _env: JNIEnv,
    _class: JClass,
    rs_handle: jlong,
    col: jint,
) -> jlong {
    let id = rs_handle as usize;
    let stmts = JNI_STMTS.lock();
    let stmt = match stmts.get(&id) {
        Some(s) => s,
        None => return 0,
    };
    let g = stmt.lock();
    let row_idx = g.current_row as usize;
    let col_idx = (col as usize).saturating_sub(1);
    if row_idx >= g.rows.len() || col_idx >= g.rows[row_idx].len() {
        return 0;
    }
    g.rows[row_idx][col_idx]
        .as_ref()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0) as jlong
}

/// Java: private static native int nativeGetColumnCount(long rsHandle)
#[no_mangle]
pub extern "system" fn Java_com_mlodbc_MlOdbcResultSet_nativeGetColumnCount(
    _env: JNIEnv,
    _class: JClass,
    rs_handle: jlong,
) -> jint {
    let id = rs_handle as usize;
    let stmts = JNI_STMTS.lock();
    let stmt = match stmts.get(&id) {
        Some(s) => s,
        None => return 0,
    };
    stmt.lock().cols.len() as jint
}

/// Java: private static native String nativeGetColumnName(long rsHandle, int col)
#[no_mangle]
pub extern "system" fn Java_com_mlodbc_MlOdbcResultSet_nativeGetColumnName<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    rs_handle: jlong,
    col: jint,
) -> jni::sys::jstring {
    let id = rs_handle as usize;
    let stmts = JNI_STMTS.lock();
    let stmt = match stmts.get(&id) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    let g = stmt.lock();
    let idx = (col as usize).saturating_sub(1);
    if idx >= g.cols.len() {
        return std::ptr::null_mut();
    }
    env.new_string(&g.cols[idx].name)
        .map(|js| js.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// Java: private static native void nativeClose(long rsHandle)
#[no_mangle]
pub extern "system" fn Java_com_mlodbc_MlOdbcResultSet_nativeClose(
    _env: JNIEnv,
    _class: JClass,
    _rs_handle: jlong,
) {
    // ResultSet shares the statement handle; nothing extra to free here.
}
