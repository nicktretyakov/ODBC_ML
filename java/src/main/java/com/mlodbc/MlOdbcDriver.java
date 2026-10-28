package com.mlodbc;

import java.sql.*;
import java.util.Properties;
import java.util.logging.Logger;

/**
 * Thin JDBC driver that talks to the native ML-ODBC Rust library via JNI.
 *
 * Registration:
 *   Class.forName("com.mlodbc.MlOdbcDriver");
 *
 * URL format:
 *   jdbc:mlodbc:Backend=mock;
 *   jdbc:mlodbc:Backend=http;Url=http://localhost:8080/predict;Model=iris
 *   jdbc:mlodbc:Backend=onnx;ModelPath=/models/iris.onnx
 */
public class MlOdbcDriver implements Driver {

    static {
        try {
            // Load the native library produced by cargo build --release
            // Linux:   libml_odbc_driver.so
            // macOS:   libml_odbc_driver.dylib
            // Windows: ml_odbc_driver.dll
            System.loadLibrary("ml_odbc_driver");
            register();
        } catch (UnsatisfiedLinkError e) {
            // Native library not found – driver will still be registered
            // but connections will fail with a clear message.
            System.err.println("ML-ODBC native library not found: " + e.getMessage());
            try { register(); } catch (SQLException ignored) {}
        } catch (SQLException e) {
            throw new ExceptionInInitializerError(e);
        }
    }

    private static void register() throws SQLException {
        DriverManager.registerDriver(new MlOdbcDriver());
    }

    @Override
    public Connection connect(String url, Properties info) throws SQLException {
        if (!acceptsURL(url)) {
            return null;
        }
        String connStr = url.substring("jdbc:mlodbc:".length());
        // Merge Properties into the connection string
        if (info != null) {
            StringBuilder sb = new StringBuilder(connStr);
            for (String key : info.stringPropertyNames()) {
                if (sb.length() > 0 && !sb.toString().endsWith(";")) {
                    sb.append(';');
                }
                sb.append(key).append('=').append(info.getProperty(key));
            }
            connStr = sb.toString();
        }
        return new MlOdbcConnection(connStr);
    }

    @Override
    public boolean acceptsURL(String url) throws SQLException {
        return url != null && url.startsWith("jdbc:mlodbc:");
    }

    @Override
    public DriverPropertyInfo[] getPropertyInfo(String url, Properties info) {
        return new DriverPropertyInfo[] {
            prop("Backend", "mock | http | onnx", "mock"),
            prop("Url", "Inference endpoint (for Backend=http)", "http://127.0.0.1:8080/predict"),
            prop("Model", "Model name", "default"),
            prop("ModelPath", "Path to .onnx file (for Backend=onnx)", "")
        };
    }

    private static DriverPropertyInfo prop(String name, String desc, String def) {
        DriverPropertyInfo p = new DriverPropertyInfo(name, def);
        p.description = desc;
        return p;
    }

    @Override public int getMajorVersion() { return 0; }
    @Override public int getMinorVersion() { return 2; }
    @Override public boolean jdbcCompliant() { return false; }
    @Override public Logger getParentLogger() { return Logger.getLogger("com.mlodbc"); }
}
