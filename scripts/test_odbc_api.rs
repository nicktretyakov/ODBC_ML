//! Stand-alone example that uses the `odbc-api` crate as a client
//! against the ML-ODBC driver (must be installed system-wide).
//!
//!   cargo run --example test_odbc_api --manifest-path scripts/Cargo_odbc_api.toml
//!
//! Or copy this file into a separate binary crate that depends on odbc-api.

/*
use odbc_api::{Environment, ConnectionOptions, Cursor};

fn main() -> anyhow::Result<()> {
    let env = Environment::new()?;
    let conn_str = "Driver={ML ODBC Driver};Backend=mock;";
    let conn = env.connect_with_connection_string(conn_str, ConnectionOptions::default())?;

    if let Some(mut cursor) = conn.execute("SELECT predicted_class, confidence FROM iris_model WHERE confidence > 0.9", ())? {
        let mut row = cursor.next_row()?;
        while let Some(mut r) = row {
            let mut label = String::new();
            let mut score = 0.0f64;
            r.get_text(1, &mut label)?;
            // score via buffer …
            println!("{} {}", label, score);
            row = cursor.next_row()?;
        }
    }
    Ok(())
}
*/
fn main() {
    println!("See the commented code in scripts/test_odbc_api.rs");
    println!("Add odbc-api as a dependency of a separate client crate and run against the installed driver.");
}
