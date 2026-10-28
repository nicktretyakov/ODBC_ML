package com.mlodbc;

import java.sql.*;

public class MlOdbcStatement implements PreparedStatement {

    private final MlOdbcConnection conn;
    private final long nativeStmt;
    private String preparedSql;
    private MlOdbcResultSet currentRs;
    private boolean closed = false;

    private static native long nativeExecute(long stmtHandle, String sql) throws SQLException;
    private static native void nativeFree(long stmtHandle);

    MlOdbcStatement(MlOdbcConnection conn, long nativeStmt) {
        this.conn = conn;
        this.nativeStmt = nativeStmt;
    }

    void setPreparedSql(String sql) {
        this.preparedSql = sql;
    }

    @Override
    public ResultSet executeQuery(String sql) throws SQLException {
        checkOpen();
        long rsHandle = nativeExecute(nativeStmt, sql);
        currentRs = new MlOdbcResultSet(this, rsHandle);
        return currentRs;
    }

    @Override
    public ResultSet executeQuery() throws SQLException {
        if (preparedSql == null) throw new SQLException("No SQL prepared");
        return executeQuery(preparedSql);
    }

    @Override
    public boolean execute(String sql) throws SQLException {
        executeQuery(sql);
        return true;
    }

    @Override
    public boolean execute() throws SQLException {
        return execute(preparedSql);
    }

    @Override
    public ResultSet getResultSet() {
        return currentRs;
    }

    @Override
    public void close() throws SQLException {
        if (!closed) {
            nativeFree(nativeStmt);
            closed = true;
        }
    }

    @Override
    public boolean isClosed() {
        return closed;
    }

    private void checkOpen() throws SQLException {
        if (closed) throw new SQLException("Statement is closed");
    }

    // --- Stubs -----------------------------------------------------------

    @Override public int executeUpdate(String sql) throws SQLException { throw new SQLFeatureNotSupportedException("read-only"); }
    @Override public int executeUpdate() throws SQLException { throw new SQLFeatureNotSupportedException("read-only"); }
    @Override public void setNull(int i, int sqlType) {}
    @Override public void setBoolean(int i, boolean x) {}
    @Override public void setByte(int i, byte x) {}
    @Override public void setShort(int i, short x) {}
    @Override public void setInt(int i, int x) {}
    @Override public void setLong(int i, long x) {}
    @Override public void setFloat(int i, float x) {}
    @Override public void setDouble(int i, double x) {}
    @Override public void setBigDecimal(int i, java.math.BigDecimal x) {}
    @Override public void setString(int i, String x) {}
    @Override public void setBytes(int i, byte[] x) {}
    @Override public void setDate(int i, Date x) {}
    @Override public void setTime(int i, Time x) {}
    @Override public void setTimestamp(int i, Timestamp x) {}
    @Override public void setAsciiStream(int i, java.io.InputStream x, int length) {}
    @Override public void setUnicodeStream(int i, java.io.InputStream x, int length) {}
    @Override public void setBinaryStream(int i, java.io.InputStream x, int length) {}
    @Override public void clearParameters() {}
    @Override public void setObject(int i, Object x, int targetSqlType) {}
    @Override public void setObject(int i, Object x) {}
    @Override public void addBatch() {}
    @Override public void setCharacterStream(int i, java.io.Reader reader, int length) {}
    @Override public void setRef(int i, Ref x) {}
    @Override public void setBlob(int i, Blob x) {}
    @Override public void setClob(int i, Clob x) {}
    @Override public void setArray(int i, Array x) {}
    @Override public ResultSetMetaData getMetaData() { return null; }
    @Override public void setDate(int i, Date x, java.util.Calendar cal) {}
    @Override public void setTime(int i, Time x, java.util.Calendar cal) {}
    @Override public void setTimestamp(int i, Timestamp x, java.util.Calendar cal) {}
    @Override public void setNull(int i, int sqlType, String typeName) {}
    @Override public void setURL(int i, java.net.URL x) {}
    @Override public ParameterMetaData getParameterMetaData() { return null; }
    @Override public void setRowId(int i, RowId x) {}
    @Override public void setNString(int i, String value) {}
    @Override public void setNCharacterStream(int i, java.io.Reader value, long length) {}
    @Override public void setNClob(int i, NClob value) {}
    @Override public void setClob(int i, java.io.Reader reader, long length) {}
    @Override public void setBlob(int i, java.io.InputStream inputStream, long length) {}
    @Override public void setNClob(int i, java.io.Reader reader, long length) {}
    @Override public void setSQLXML(int i, SQLXML xmlObject) {}
    @Override public void setObject(int i, Object x, int targetSqlType, int scaleOrLength) {}
    @Override public void setAsciiStream(int i, java.io.InputStream x, long length) {}
    @Override public void setBinaryStream(int i, java.io.InputStream x, long length) {}
    @Override public void setCharacterStream(int i, java.io.Reader reader, long length) {}
    @Override public void setAsciiStream(int i, java.io.InputStream x) {}
    @Override public void setBinaryStream(int i, java.io.InputStream x) {}
    @Override public void setCharacterStream(int i, java.io.Reader reader) {}
    @Override public void setNCharacterStream(int i, java.io.Reader value) {}
    @Override public void setClob(int i, java.io.Reader reader) {}
    @Override public void setBlob(int i, java.io.InputStream inputStream) {}
    @Override public void setNClob(int i, java.io.Reader reader) {}

    // Statement methods
    @Override public int getMaxFieldSize() { return 0; }
    @Override public void setMaxFieldSize(int max) {}
    @Override public int getMaxRows() { return 0; }
    @Override public void setMaxRows(int max) {}
    @Override public void setEscapeProcessing(boolean enable) {}
    @Override public int getQueryTimeout() { return 0; }
    @Override public void setQueryTimeout(int seconds) {}
    @Override public void cancel() {}
    @Override public SQLWarning getWarnings() { return null; }
    @Override public void clearWarnings() {}
    @Override public void setCursorName(String name) {}
    @Override public int getUpdateCount() { return -1; }
    @Override public boolean getMoreResults() { return false; }
    @Override public void setFetchDirection(int direction) {}
    @Override public int getFetchDirection() { return ResultSet.FETCH_FORWARD; }
    @Override public void setFetchSize(int rows) {}
    @Override public int getFetchSize() { return 0; }
    @Override public int getResultSetConcurrency() { return ResultSet.CONCUR_READ_ONLY; }
    @Override public int getResultSetType() { return ResultSet.TYPE_FORWARD_ONLY; }
    @Override public void addBatch(String sql) {}
    @Override public void clearBatch() {}
    @Override public int[] executeBatch() { return new int[0]; }
    @Override public Connection getConnection() { return conn; }
    @Override public boolean getMoreResults(int current) { return false; }
    @Override public ResultSet getGeneratedKeys() { return null; }
    @Override public int executeUpdate(String sql, int autoGeneratedKeys) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public int executeUpdate(String sql, int[] columnIndexes) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public int executeUpdate(String sql, String[] columnNames) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public boolean execute(String sql, int autoGeneratedKeys) throws SQLException { return execute(sql); }
    @Override public boolean execute(String sql, int[] columnIndexes) throws SQLException { return execute(sql); }
    @Override public boolean execute(String sql, String[] columnNames) throws SQLException { return execute(sql); }
    @Override public int getResultSetHoldability() { return ResultSet.CLOSE_CURSORS_AT_COMMIT; }
    @Override public boolean isPoolable() { return false; }
    @Override public void setPoolable(boolean poolable) {}
    @Override public void closeOnCompletion() {}
    @Override public boolean isCloseOnCompletion() { return false; }
    @Override public <T> T unwrap(Class<T> iface) throws SQLException { throw new SQLException(); }
    @Override public boolean isWrapperFor(Class<?> iface) { return false; }
}
