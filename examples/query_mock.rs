//! Example: query the mock ML models via the ODBC driver itself
//! (useful once the shared library is registered with the Driver Manager).
//!
//!   cargo run --example query_mock
//!
//! Requires the driver to be installed (see README).

fn main() {
    println!("ML-ODBC Driver – example client");
    println!("================================");
    println!();
    println!("This binary is a placeholder.");
    println!("After installing the driver, use any ODBC client, for example:");
    println!();
    println!("  isql -v \"Driver={{ML ODBC Driver}};Backend=mock;\"");
    println!();
    println!("  or from Python:");
    println!("  import pyodbc");
    println!("  conn = pyodbc.connect('Driver={{ML ODBC Driver}};Backend=mock;')");
    println!("  for row in conn.cursor().execute('SELECT * FROM iris_model'):");
    println!("      print(row)");
    println!();
    println!("  or from Rust with odbc-api:");
    println!("  let env = Environment::new()?;");
    println!("  let conn = env.connect_with_connection_string(");
    println!("      \"Driver={{ML ODBC Driver}};Backend=mock;\",");
    println!("      ConnectionOptions::default())?;");
}
