//! Thread-safe handle management.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock, OnceLock};

use crate::odbc_sys::{Handle, SqlReturn, SQL_INVALID_HANDLE, SQL_SUCCESS};
use crate::backend::{Backend, MockBackend, HttpBackend, OnnxBackend};
use crate::diag::Diagnostic;

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

fn registry() -> &'static RwLock<HashMap<usize, Arc<Mutex<HandleData>>>> {
    static REG: OnceLock<RwLock<HashMap<usize, Arc<Mutex<HandleData>>>>> = OnceLock::new();
    REG.get_or_init(|| RwLock::new(HashMap::new()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleKind {
    Env,
    Dbc,
    Stmt,
}

#[derive(Debug, Clone)]
pub struct ColumnDesc {
    pub name: String,
    pub sql_type: i16,
    pub nullable: bool,
    pub precision: i16,
    pub scale: i16,
    pub display_size: i16,
}

#[derive(Debug, Clone)]
pub struct BoundColumn {
    pub target_type: i16,
    pub target_value: *mut std::ffi::c_void,
    pub buffer_length: isize,
    pub strlen_or_ind: *mut isize,
}

unsafe impl Send for BoundColumn {}
unsafe impl Sync for BoundColumn {}

pub struct HandleData {
    pub kind: HandleKind,
    pub parent: Option<usize>,
    pub diagnostics: Vec<Diagnostic>,
    pub connected: bool,
    pub connection_string: String,
    pub backend: Option<Box<dyn Backend + Send>>,
    pub sql: Option<String>,
    pub result_cols: Vec<ColumnDesc>,
    pub result_rows: Vec<Vec<Option<String>>>,
    pub current_row: isize,
    pub row_array_size: usize,
    pub bound_cols: HashMap<u16, BoundColumn>,
    pub prepared: bool,
    pub executed: bool,
}

impl Default for HandleData {
    fn default() -> Self {
        Self {
            kind: HandleKind::Env,
            parent: None,
            diagnostics: Vec::new(),
            connected: false,
            connection_string: String::new(),
            backend: None,
            sql: None,
            result_cols: Vec::new(),
            result_rows: Vec::new(),
            current_row: -1,
            row_array_size: 1,
            bound_cols: HashMap::new(),
            prepared: false,
            executed: false,
        }
    }
}

pub fn alloc_handle(kind: HandleKind, parent: Option<usize>) -> Result<usize, SqlReturn> {
    if kind != HandleKind::Env {
        let parent_id = parent.ok_or(SQL_INVALID_HANDLE)?;
        let reg = registry().read().unwrap();
        let parent_h = reg.get(&parent_id).ok_or(SQL_INVALID_HANDLE)?;
        let parent_data = parent_h.lock().unwrap();
        match (kind, parent_data.kind) {
            (HandleKind::Dbc, HandleKind::Env) => {}
            (HandleKind::Stmt, HandleKind::Dbc) => {}
            _ => return Err(SQL_INVALID_HANDLE),
        }
    }

    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let data = HandleData {
        kind,
        parent,
        ..Default::default()
    };
    registry().write().unwrap().insert(id, Arc::new(Mutex::new(data)));
    Ok(id)
}

pub fn free_handle(id: usize) -> SqlReturn {
    let children: Vec<usize> = {
        let reg = registry().read().unwrap();
        reg.iter()
            .filter_map(|(cid, h)| {
                let g = h.lock().unwrap();
                if g.parent == Some(id) {
                    Some(*cid)
                } else {
                    None
                }
            })
            .collect()
    };
    for child in children {
        let _ = free_handle(child);
    }
    if registry().write().unwrap().remove(&id).is_some() {
        SQL_SUCCESS
    } else {
        SQL_INVALID_HANDLE
    }
}

pub fn get_handle(id: usize) -> Option<Arc<Mutex<HandleData>>> {
    registry().read().unwrap().get(&id).cloned()
}

pub fn handle_from_ptr(handle: Handle) -> Option<usize> {
    if handle.is_null() {
        None
    } else {
        Some(handle as usize)
    }
}

pub fn ptr_from_id(id: usize) -> Handle {
    id as Handle
}

pub fn create_backend(conn_str: &str) -> Result<Box<dyn Backend + Send>, String> {
    let backend = extract_param(conn_str, "Backend")
        .unwrap_or_else(|| "mock".into())
        .to_lowercase();
    match backend.as_str() {
        "http" => {
            let url = extract_param(conn_str, "Url")
                .or_else(|| extract_param(conn_str, "URL"))
                .unwrap_or_else(|| "http://127.0.0.1:8080/predict".into());
            let model = extract_param(conn_str, "Model").unwrap_or_else(|| "default".into());
            Ok(Box::new(HttpBackend::new(url, model)))
        }
        "onnx" => {
            let path = extract_param(conn_str, "ModelPath")
                .or_else(|| extract_param(conn_str, "Model"))
                .ok_or_else(|| "Backend=onnx requires ModelPath=…".to_string())?;
            Ok(Box::new(OnnxBackend::new(path)?))
        }
        _ => Ok(Box::new(MockBackend::new())),
    }
}

fn extract_param(conn_str: &str, key: &str) -> Option<String> {
    for part in conn_str.split(';') {
        let part = part.trim();
        if let Some((k, v)) = part.split_once('=') {
            if k.trim().eq_ignore_ascii_case(key) {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}
