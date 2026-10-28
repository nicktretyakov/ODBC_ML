package com.mlodbc;

import java.sql.*;
import java.util.Map;
import java.util.Properties;
import java.util.concurrent.Executor;

/**
 * JDBC Connection backed by the native ML-ODBC driver (via JNI).
 *
 * The native side re-uses the same handle management and backends
 * that the pure ODBC path uses.
 */
public class MlOdbcConnection implements Connection {

    // Native handle (opaque pointer / id returned by the Rust side)
    private long nativeHandle;
    private boolean closed = false;
    private final String connectionString;

    // Native methods – implemented in the Rust cdylib via JNI
    private static native long nativeConnect(String connStr) throws SQLException;
    private static native void nativeDisconnect(long handle);
    private static native long nativeCreateStatement(long connHandle) throws SQLException;

    MlOdbcConnection(String connStr) throws SQLException {
        this.connectionString = connStr;
        this.nativeHandle = nativeConnect(connStr);
        if (this.nativeHandle == 0) {
            throw new SQLException("Failed to open native ML-ODBC connection");
        }
    }

    long getNativeHandle() {
        return nativeHandle;
    }

    @Override
    public Statement createStatement() throws SQLException {
        checkOpen();
        long stmt = nativeCreateStatement(nativeHandle);
        return new MlOdbcStatement(this, stmt);
    }

    @Override
    public PreparedStatement prepareStatement(String sql) throws SQLException {
        checkOpen();
        MlOdbcStatement stmt = (MlOdbcStatement) createStatement();
        stmt.setPreparedSql(sql);
        return stmt;
    }

    @Override
    public void close() throws SQLException {
        if (!closed) {
            nativeDisconnect(nativeHandle);
            nativeHandle = 0;
            closed = true;
        }
    }

    @Override
    public boolean isClosed() {
        return closed;
    }

    private void checkOpen() throws SQLException {
        if (closed) throw new SQLException("Connection is closed");
    }

    // --- Minimal stubs required by the JDBC interface -------------------

    @Override public String getCatalog() { return null; }
    @Override public void setCatalog(String catalog) {}
    @Override public String getSchema() { return null; }
    @Override public void setSchema(String schema) {}
    @Override public int getTransactionIsolation() { return TRANSACTION_NONE; }
    @Override public void setTransactionIsolation(int level) {}
    @Override public boolean getAutoCommit() { return true; }
    @Override public void setAutoCommit(boolean autoCommit) {}
    @Override public void commit() {}
    @Override public void rollback() {}
    @Override public boolean isReadOnly() { return true; }
    @Override public void setReadOnly(boolean readOnly) {}
    @Override public DatabaseMetaData getMetaData() throws SQLException {
        return new MlOdbcDatabaseMetaData(this);
    }
    @Override public SQLWarning getWarnings() { return null; }
    @Override public void clearWarnings() {}
    @Override public String nativeSQL(String sql) { return sql; }
    @Override public boolean isValid(int timeout) { return !closed; }

    // Everything else throws "not supported" for the skeleton
    @Override public CallableStatement prepareCall(String sql) throws SQLException {
        throw new SQLFeatureNotSupportedException();
    }
    @Override public Statement createStatement(int a, int b) throws SQLException {
        return createStatement();
    }
    @Override public PreparedStatement prepareStatement(String s, int a, int b) throws SQLException {
        return prepareStatement(s);
    }
    @Override public CallableStatement prepareCall(String s, int a, int b) throws SQLException {
        throw new SQLFeatureNotSupportedException();
    }
    @Override public Map<String,Class<?>> getTypeMap() { return null; }
    @Override public void setTypeMap(Map<String,Class<?>> map) {}
    @Override public void setHoldability(int holdability) {}
    @Override public int getHoldability() { return ResultSet.CLOSE_CURSORS_AT_COMMIT; }
    @Override public Savepoint setSavepoint() throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public Savepoint setSavepoint(String name) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void rollback(Savepoint s) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void releaseSavepoint(Savepoint s) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public Statement createStatement(int a, int b, int c) throws SQLException { return createStatement(); }
    @Override public PreparedStatement prepareStatement(String s, int a, int b, int c) throws SQLException { return prepareStatement(s); }
    @Override public CallableStatement prepareCall(String s, int a, int b, int c) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public PreparedStatement prepareStatement(String s, int autoKeys) throws SQLException { return prepareStatement(s); }
    @Override public PreparedStatement prepareStatement(String s, int[] cols) throws SQLException { return prepareStatement(s); }
    @Override public PreparedStatement prepareStatement(String s, String[] cols) throws SQLException { return prepareStatement(s); }
    @Override public Clob createClob() throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public Blob createBlob() throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public NClob createNClob() throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public SQLXML createSQLXML() throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void setClientInfo(String name, String value) {}
    @Override public void setClientInfo(Properties properties) {}
    @Override public String getClientInfo(String name) { return null; }
    @Override public Properties getClientInfo() { return new Properties(); }
    @Override public Array createArrayOf(String typeName, Object[] elements) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public Struct createStruct(String typeName, Object[] attributes) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void abort(Executor executor) throws SQLException { close(); }
    @Override public void setNetworkTimeout(Executor executor, int milliseconds) {}
    @Override public int getNetworkTimeout() { return 0; }
    @Override public <T> T unwrap(Class<T> iface) throws SQLException { throw new SQLException("not a wrapper"); }
    @Override public boolean isWrapperFor(Class<?> iface) { return false; }
}
