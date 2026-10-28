//! ML inference backends: Mock, Http (stub), Onnx (stub without feature).

use crate::handles::ColumnDesc;
use crate::sql_parser::ParsedQuery;
use crate::odbc_sys::{SQL_INTEGER, SQL_DOUBLE, SQL_VARCHAR, SQL_BIGINT, SQL_BIT};

pub trait Backend: Send {
    fn tables(&self) -> Vec<String>;
    fn columns(&self, table: &str) -> Result<Vec<ColumnDesc>, String>;
    fn execute(
        &self,
        query: &ParsedQuery,
    ) -> Result<(Vec<ColumnDesc>, Vec<Vec<Option<String>>>), String>;
}

// ---------------------------------------------------------------------------
// Mock
// ---------------------------------------------------------------------------

pub struct MockBackend;

impl MockBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Backend for MockBackend {
    fn tables(&self) -> Vec<String> {
        vec![
            "iris_model".into(),
            "churn_model".into(),
            "sentiment".into(),
        ]
    }

    fn columns(&self, table: &str) -> Result<Vec<ColumnDesc>, String> {
        match table.to_lowercase().as_str() {
            "iris_model" => Ok(vec![
                col("sepal_length", SQL_DOUBLE, true, 15, 6, 20),
                col("sepal_width", SQL_DOUBLE, true, 15, 6, 20),
                col("petal_length", SQL_DOUBLE, true, 15, 6, 20),
                col("petal_width", SQL_DOUBLE, true, 15, 6, 20),
                col("predicted_class", SQL_VARCHAR, false, 0, 0, 32),
                col("confidence", SQL_DOUBLE, false, 15, 6, 20),
            ]),
            "churn_model" => Ok(vec![
                col("customer_id", SQL_BIGINT, false, 19, 0, 20),
                col("tenure", SQL_INTEGER, true, 10, 0, 11),
                col("monthly_charges", SQL_DOUBLE, true, 15, 2, 20),
                col("churn_probability", SQL_DOUBLE, false, 15, 6, 20),
                col("will_churn", SQL_BIT, false, 1, 0, 1),
            ]),
            "sentiment" => Ok(vec![
                col("text", SQL_VARCHAR, false, 0, 0, 1024),
                col("label", SQL_VARCHAR, false, 0, 0, 16),
                col("score", SQL_DOUBLE, false, 15, 6, 20),
            ]),
            _ => Err(format!("unknown model/table: {}", table)),
        }
    }

    fn execute(
        &self,
        query: &ParsedQuery,
    ) -> Result<(Vec<ColumnDesc>, Vec<Vec<Option<String>>>), String> {
        let cols = self.columns(&query.table)?;
        let mut rows = match query.table.to_lowercase().as_str() {
            "iris_model" => vec![
                row(&["5.1", "3.5", "1.4", "0.2", "setosa", "0.98"]),
                row(&["6.2", "2.9", "4.3", "1.3", "versicolor", "0.87"]),
                row(&["7.3", "2.9", "6.3", "1.8", "virginica", "0.95"]),
                row(&["5.0", "3.4", "1.5", "0.2", "setosa", "0.97"]),
                row(&["6.7", "3.1", "4.7", "1.5", "versicolor", "0.91"]),
            ],
            "churn_model" => vec![
                row(&["1001", "24", "70.5", "0.12", "0"]),
                row(&["1002", "3", "95.0", "0.78", "1"]),
                row(&["1003", "48", "45.2", "0.05", "0"]),
                row(&["1004", "12", "110.0", "0.65", "1"]),
            ],
            "sentiment" => vec![
                row(&["I love this product", "positive", "0.94"]),
                row(&["Terrible experience", "negative", "0.89"]),
                row(&["It is okay", "neutral", "0.61"]),
            ],
            _ => return Err(format!("unknown model: {}", query.table)),
        };

        apply_where_and_limit(&cols, &mut rows, query);
        project_columns(cols, rows, query)
    }
}

// ---------------------------------------------------------------------------
// HTTP stub (no reqwest in minimal build)
// ---------------------------------------------------------------------------

pub struct HttpBackend {
    url: String,
    model: String,
}

impl HttpBackend {
    pub fn new(url: String, model: String) -> Self {
        Self { url, model }
    }
}

impl Backend for HttpBackend {
    fn tables(&self) -> Vec<String> {
        vec![self.model.clone()]
    }

    fn columns(&self, _table: &str) -> Result<Vec<ColumnDesc>, String> {
        Ok(vec![
            col("id", SQL_BIGINT, false, 19, 0, 20),
            col("prediction", SQL_DOUBLE, false, 15, 6, 20),
            col("label", SQL_VARCHAR, true, 0, 0, 64),
        ])
    }

    fn execute(
        &self,
        _query: &ParsedQuery,
    ) -> Result<(Vec<ColumnDesc>, Vec<Vec<Option<String>>>), String> {
        Err(format!(
            "HTTP backend is a stub in this minimal build (url={}, model={}). \
             Add reqwest dependency to enable real HTTP inference.",
            self.url, self.model
        ))
    }
}

// ---------------------------------------------------------------------------
// ONNX stub
// ---------------------------------------------------------------------------

pub struct OnnxBackend {
    model_path: String,
}

impl OnnxBackend {
    pub fn new(model_path: String) -> Result<Self, String> {
        Ok(Self { model_path })
    }
}

impl Backend for OnnxBackend {
    fn tables(&self) -> Vec<String> {
        let name = std::path::Path::new(&self.model_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("onnx_model")
            .to_string();
        vec![name]
    }

    fn columns(&self, _table: &str) -> Result<Vec<ColumnDesc>, String> {
        Ok(vec![
            col("feature_0", SQL_DOUBLE, true, 15, 6, 20),
            col("feature_1", SQL_DOUBLE, true, 15, 6, 20),
            col("feature_2", SQL_DOUBLE, true, 15, 6, 20),
            col("feature_3", SQL_DOUBLE, true, 15, 6, 20),
            col("predicted_label", SQL_VARCHAR, false, 0, 0, 64),
            col("score", SQL_DOUBLE, false, 15, 6, 20),
        ])
    }

    fn execute(
        &self,
        _query: &ParsedQuery,
    ) -> Result<(Vec<ColumnDesc>, Vec<Vec<Option<String>>>), String> {
        Err(format!(
            "ONNX backend stub (path={}). Enable feature onnx + ort crate for real inference.",
            self.model_path
        ))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn col(
    name: &str,
    sql_type: i16,
    nullable: bool,
    precision: i16,
    scale: i16,
    display: i16,
) -> ColumnDesc {
    ColumnDesc {
        name: name.into(),
        sql_type,
        nullable,
        precision,
        scale,
        display_size: display,
    }
}

fn row(vals: &[&str]) -> Vec<Option<String>> {
    vals.iter().map(|v| Some((*v).to_string())).collect()
}

fn apply_where_and_limit(
    cols: &[ColumnDesc],
    rows: &mut Vec<Vec<Option<String>>>,
    query: &ParsedQuery,
) {
    if let Some(ref w) = query.where_clause {
        if let Some(col_idx) = cols
            .iter()
            .position(|c| c.name.eq_ignore_ascii_case(&w.column))
        {
            rows.retain(|r| {
                let cell = r
                    .get(col_idx)
                    .and_then(|v| v.as_ref())
                    .map(|s| s.as_str())
                    .unwrap_or("");
                match w.op.as_str() {
                    "=" => cell == w.value,
                    "!=" | "<>" => cell != w.value,
                    "<" => cell
                        .parse::<f64>()
                        .ok()
                        .and_then(|a| w.value.parse::<f64>().ok().map(|b| a < b))
                        .unwrap_or(false),
                    ">" => cell
                        .parse::<f64>()
                        .ok()
                        .and_then(|a| w.value.parse::<f64>().ok().map(|b| a > b))
                        .unwrap_or(false),
                    "<=" => cell
                        .parse::<f64>()
                        .ok()
                        .and_then(|a| w.value.parse::<f64>().ok().map(|b| a <= b))
                        .unwrap_or(false),
                    ">=" => cell
                        .parse::<f64>()
                        .ok()
                        .and_then(|a| w.value.parse::<f64>().ok().map(|b| a >= b))
                        .unwrap_or(false),
                    _ => false,
                }
            });
        }
    }
    if let Some(lim) = query.limit {
        rows.truncate(lim);
    }
}

fn project_columns(
    cols: Vec<ColumnDesc>,
    rows: Vec<Vec<Option<String>>>,
    query: &ParsedQuery,
) -> Result<(Vec<ColumnDesc>, Vec<Vec<Option<String>>>), String> {
    if query.columns.len() == 1 && query.columns[0] == "*" {
        return Ok((cols, rows));
    }
    let indices: Vec<usize> = query
        .columns
        .iter()
        .filter_map(|name| cols.iter().position(|c| c.name.eq_ignore_ascii_case(name)))
        .collect();
    if indices.len() != query.columns.len() {
        return Err("one or more selected columns not found".into());
    }
    let new_cols: Vec<ColumnDesc> = indices.iter().map(|&i| cols[i].clone()).collect();
    let new_rows: Vec<Vec<Option<String>>> = rows
        .into_iter()
        .map(|r| indices.iter().map(|&i| r[i].clone()).collect())
        .collect();
    Ok((new_cols, new_rows))
}
