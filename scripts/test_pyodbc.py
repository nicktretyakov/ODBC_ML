#!/usr/bin/env python3
"""
Integration test for ML-ODBC Driver via pyodbc.

Prerequisites:
  1. Build & install the driver (see README)
  2. pip install pyodbc
  3. Driver registered in odbcinst.ini

Run:
  python scripts/test_pyodbc.py
"""

import sys

try:
    import pyodbc
except ImportError:
    print("pyodbc not installed – skipping")
    sys.exit(0)

CONN_STR = "Driver={ML ODBC Driver};Backend=mock;"

def main():
    print("Connecting:", CONN_STR)
    conn = pyodbc.connect(CONN_STR, autocommit=True)
    cur = conn.cursor()

    print("\n--- SQLTables (via SELECT from known models) ---")
    for table in ("iris_model", "churn_model", "sentiment"):
        cur.execute(f"SELECT * FROM {table} LIMIT 1")
        cols = [d[0] for d in cur.description]
        print(f"  {table}: columns = {cols}")

    print("\n--- iris_model WHERE confidence > 0.9 ---")
    cur.execute(
        "SELECT predicted_class, confidence FROM iris_model WHERE confidence > 0.9"
    )
    rows = cur.fetchall()
    for r in rows:
        print(" ", r)
    assert len(rows) >= 2, "expected filtered rows"

    print("\n--- churn_model will_churn = 1 ---")
    cur.execute("SELECT customer_id, churn_probability FROM churn_model WHERE will_churn = 1")
    for r in cur:
        print(" ", r)

    print("\nAll pyodbc checks passed.")
    conn.close()

if __name__ == "__main__":
    main()
