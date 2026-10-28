# ML-ODBC Driver v0.3

ODBC-драйвер + JDBC (JNI), который представляет **результаты inference ML-моделей** как обычные реляционные таблицы.

```sql
SELECT * FROM iris_model WHERE confidence > 0.9 LIMIT 10;
SELECT predicted_class, confidence FROM iris_model;
```

## Возможности v0.3

| Компонент | Статус |
|-----------|--------|
| ODBC 3.x entry points (MVP set) | ✅ |
| Thread-safe handles (`parking_lot`) | ✅ |
| SQL-парсер на `sqlparser` | ✅ |
| Mock / HTTP backends | ✅ |
| **ONNX backend с `ort::Session`** (feature `onnx`) | ✅ |
| **JNI bridge для JDBC** (feature `jni`) | ✅ |
| **Интеграционные тесты** (C API + pyodbc script) | ✅ |

## Быстрый старт

```bash
# Сборка (базовая)
cargo build --release

# С JNI (для Java JDBC)
cargo build --release --features jni

# С ONNX (нужен ort + libonnxruntime)
# 1. Раскомментируйте ort в Cargo.toml
# 2. cargo build --release --features onnx
```

Установка (Linux):

```bash
sudo cp target/release/libml_odbc_driver.so /usr/local/lib/
sudo ldconfig
# добавить запись из config/odbcinst.ini в /etc/odbcinst.ini
```

## Connection string

```
Driver={ML ODBC Driver};Backend=mock;
Driver={ML ODBC Driver};Backend=http;Url=http://127.0.0.1:8080/predict;Model=iris;
Driver={ML ODBC Driver};Backend=onnx;ModelPath=/opt/models/iris.onnx;
```

Опционально рядом с `.onnx` положите файл `iris.onnx.labels` (по одной метке класса на строку).

## Тесты

### Встроенные (без Driver Manager)

```bash
cargo test --test integration_mock
```

Тесты напрямую вызывают `SQLAllocHandle` / `SQLDriverConnect` / `SQLExecDirect` / `SQLFetch` и проверяют:

- выделение/освобождение хэндлов
- SELECT + LIMIT на mock iris_model
- SQLTables возвращает iris/churn/sentiment
- WHERE confidence > 0.9 фильтрует строки

### pyodbc (после установки драйвера)

```bash
pip install pyodbc
python scripts/test_pyodbc.py
```

### odbc-api (Rust-клиент)

См. заготовку в `scripts/test_odbc_api.rs` — подключите `odbc-api` в отдельном клиентском крейте.

## JDBC (Java)

```bash
# Собрать native с JNI
cargo build --release --features jni

# Java-классы уже в java/src/main/java/com/mlodbc/
# Добавьте libml_odbc_driver в java.library.path

Class.forName("com.mlodbc.MlOdbcDriver");
Connection c = DriverManager.getConnection("jdbc:mlodbc:Backend=mock;");
ResultSet rs = c.createStatement().executeQuery("SELECT * FROM iris_model LIMIT 5");
while (rs.next()) {
    System.out.println(rs.getString("predicted_class"));
}
```

JNI-экспорты реализованы в `src/jni_bridge.rs`:

- `Java_com_mlodbc_MlOdbcConnection_nativeConnect`
- `nativeDisconnect` / `nativeCreateStatement`
- `Java_com_mlodbc_MlOdbcStatement_nativeExecute` / `nativeFree`
- `Java_com_mlodbc_MlOdbcResultSet_nativeNext` / `nativeGetString` / `nativeGetDouble` / …

## ONNX backend (feature = "onnx")

При включённой фиче:

1. Загружается `ort::Session` из `ModelPath`.
2. Интроспекция входов/выходов → schema `feature_0 … feature_N, predicted_label, score`.
3. `execute` строит batch, вызывает `session.run`, argmax по logits → label + score.
4. Метки классов читаются из `<model>.labels` (опционально).

Без фичи драйвер всё равно собирается; при `Backend=onnx` возвращается понятная ошибка с инструкцией.

## Структура

```
ml-odbc-driver/
├── src/
│   ├── lib.rs
│   ├── handles.rs
│   ├── diag.rs
│   ├── sql_parser.rs      # sqlparser
│   ├── backend.rs         # Mock / Http / Onnx(Session)
│   ├── driver.rs          # ODBC C API
│   └── jni_bridge.rs      # JNI (feature jni)
├── java/…/com/mlodbc/     # JDBC driver
├── tests/integration_mock.rs
├── scripts/test_pyodbc.py
└── config/odbcinst.ini
```

## Лицензия

MIT OR Apache-2.0
