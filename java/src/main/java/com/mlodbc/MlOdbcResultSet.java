package com.mlodbc;

import java.sql.*;
import java.math.BigDecimal;
import java.net.URL;
import java.util.Calendar;
import java.util.Map;

/**
 * Forward-only, read-only ResultSet backed by the native driver.
 */
public class MlOdbcResultSet implements ResultSet {

    private final MlOdbcStatement stmt;
    private final long nativeRs;
    private boolean closed = false;
    private boolean wasNull = false;

    private static native boolean nativeNext(long rsHandle);
    private static native String nativeGetString(long rsHandle, int col);
    private static native double nativeGetDouble(long rsHandle, int col);
    private static native long nativeGetLong(long rsHandle, int col);
    private static native int nativeGetColumnCount(long rsHandle);
    private static native String nativeGetColumnName(long rsHandle, int col);
    private static native void nativeClose(long rsHandle);

    MlOdbcResultSet(MlOdbcStatement stmt, long nativeRs) {
        this.stmt = stmt;
        this.nativeRs = nativeRs;
    }

    @Override
    public boolean next() throws SQLException {
        checkOpen();
        return nativeNext(nativeRs);
    }

    @Override
    public String getString(int columnIndex) throws SQLException {
        checkOpen();
        String v = nativeGetString(nativeRs, columnIndex);
        wasNull = (v == null);
        return v;
    }

    @Override
    public String getString(String columnLabel) throws SQLException {
        return getString(findColumn(columnLabel));
    }

    @Override
    public double getDouble(int columnIndex) throws SQLException {
        checkOpen();
        // Simplified – real impl would check null indicator
        double v = nativeGetDouble(nativeRs, columnIndex);
        return v;
    }

    @Override
    public long getLong(int columnIndex) throws SQLException {
        checkOpen();
        return nativeGetLong(nativeRs, columnIndex);
    }

    @Override
    public int getInt(int columnIndex) throws SQLException {
        return (int) getLong(columnIndex);
    }

    @Override
    public boolean wasNull() {
        return wasNull;
    }

    @Override
    public void close() throws SQLException {
        if (!closed) {
            nativeClose(nativeRs);
            closed = true;
        }
    }

    @Override
    public boolean isClosed() {
        return closed;
    }

    @Override
    public int findColumn(String columnLabel) throws SQLException {
        int count = nativeGetColumnCount(nativeRs);
        for (int i = 1; i <= count; i++) {
            if (columnLabel.equalsIgnoreCase(nativeGetColumnName(nativeRs, i))) {
                return i;
            }
        }
        throw new SQLException("Column not found: " + columnLabel);
    }

    @Override
    public ResultSetMetaData getMetaData() throws SQLException {
        return new ResultSetMetaData() {
            @Override public int getColumnCount() { return nativeGetColumnCount(nativeRs); }
            @Override public String getColumnName(int i) { return nativeGetColumnName(nativeRs, i); }
            @Override public String getColumnLabel(int i) { return getColumnName(i); }
            @Override public int getColumnType(int i) { return Types.VARCHAR; }
            @Override public String getColumnTypeName(int i) { return "VARCHAR"; }
            @Override public boolean isNullable(int i) { return ResultSetMetaData.columnNullable; }
            // … remaining methods return defaults / 0 / false
            @Override public boolean isAutoIncrement(int i) { return false; }
            @Override public boolean isCaseSensitive(int i) { return true; }
            @Override public boolean isSearchable(int i) { return true; }
            @Override public boolean isCurrency(int i) { return false; }
            @Override public boolean isSigned(int i) { return true; }
            @Override public int getColumnDisplaySize(int i) { return 64; }
            @Override public int getPrecision(int i) { return 0; }
            @Override public int getScale(int i) { return 0; }
            @Override public String getSchemaName(int i) { return ""; }
            @Override public String getTableName(int i) { return ""; }
            @Override public String getCatalogName(int i) { return ""; }
            @Override public boolean isReadOnly(int i) { return true; }
            @Override public boolean isWritable(int i) { return false; }
            @Override public boolean isDefinitelyWritable(int i) { return false; }
            @Override public String getColumnClassName(int i) { return "java.lang.String"; }
            @Override public <T> T unwrap(Class<T> iface) throws SQLException { throw new SQLException(); }
            @Override public boolean isWrapperFor(Class<?> iface) { return false; }
        };
    }

    private void checkOpen() throws SQLException {
        if (closed) throw new SQLException("ResultSet is closed");
    }

    // --- Large number of required stubs (all throw or return default) ---
    @Override public boolean getBoolean(int i) throws SQLException { return Boolean.parseBoolean(getString(i)); }
    @Override public byte getByte(int i) throws SQLException { return (byte) getInt(i); }
    @Override public short getShort(int i) throws SQLException { return (short) getInt(i); }
    @Override public float getFloat(int i) throws SQLException { return (float) getDouble(i); }
    @Override public byte[] getBytes(int i) throws SQLException { return getString(i).getBytes(); }
    @Override public Date getDate(int i) throws SQLException { return null; }
    @Override public Time getTime(int i) throws SQLException { return null; }
    @Override public Timestamp getTimestamp(int i) throws SQLException { return null; }
    @Override public java.io.InputStream getAsciiStream(int i) { return null; }
    @Override public java.io.InputStream getUnicodeStream(int i) { return null; }
    @Override public java.io.InputStream getBinaryStream(int i) { return null; }
    @Override public SQLWarning getWarnings() { return null; }
    @Override public void clearWarnings() {}
    @Override public String getCursorName() { return null; }
    @Override public Object getObject(int i) throws SQLException { return getString(i); }
    @Override public java.io.Reader getCharacterStream(int i) { return null; }
    @Override public BigDecimal getBigDecimal(int i) throws SQLException { return new BigDecimal(getString(i)); }
    @Override public boolean isBeforeFirst() { return false; }
    @Override public boolean isAfterLast() { return false; }
    @Override public boolean isFirst() { return false; }
    @Override public boolean isLast() { return false; }
    @Override public void beforeFirst() throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void afterLast() throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public boolean first() throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public boolean last() throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public int getRow() { return 0; }
    @Override public boolean absolute(int row) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public boolean relative(int rows) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public boolean previous() throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void setFetchDirection(int direction) {}
    @Override public int getFetchDirection() { return FETCH_FORWARD; }
    @Override public void setFetchSize(int rows) {}
    @Override public int getFetchSize() { return 0; }
    @Override public int getType() { return TYPE_FORWARD_ONLY; }
    @Override public int getConcurrency() { return CONCUR_READ_ONLY; }
    @Override public boolean rowUpdated() { return false; }
    @Override public boolean rowInserted() { return false; }
    @Override public boolean rowDeleted() { return false; }
    @Override public void updateNull(int i) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateBoolean(int i, boolean x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateByte(int i, byte x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateShort(int i, short x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateInt(int i, int x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateLong(int i, long x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateFloat(int i, float x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateDouble(int i, double x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateBigDecimal(int i, BigDecimal x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateString(int i, String x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateBytes(int i, byte[] x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateDate(int i, Date x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateTime(int i, Time x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateTimestamp(int i, Timestamp x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateAsciiStream(int i, java.io.InputStream x, int length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateBinaryStream(int i, java.io.InputStream x, int length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateCharacterStream(int i, java.io.Reader x, int length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateObject(int i, Object x, int scaleOrLength) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateObject(int i, Object x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void insertRow() throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateRow() throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void deleteRow() throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void refreshRow() throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void cancelRowUpdates() throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void moveToInsertRow() throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void moveToCurrentRow() throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public Statement getStatement() { return stmt; }
    @Override public Object getObject(int i, Map<String,Class<?>> map) throws SQLException { return getObject(i); }
    @Override public Ref getRef(int i) { return null; }
    @Override public Blob getBlob(int i) { return null; }
    @Override public Clob getClob(int i) { return null; }
    @Override public Array getArray(int i) { return null; }
    @Override public Date getDate(int i, Calendar cal) { return null; }
    @Override public Time getTime(int i, Calendar cal) { return null; }
    @Override public Timestamp getTimestamp(int i, Calendar cal) { return null; }
    @Override public URL getURL(int i) { return null; }
    @Override public void updateRef(int i, Ref x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateBlob(int i, Blob x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateClob(int i, Clob x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateArray(int i, Array x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public RowId getRowId(int i) { return null; }
    @Override public void updateRowId(int i, RowId x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public int getHoldability() { return CLOSE_CURSORS_AT_COMMIT; }
    @Override public void updateNString(int i, String nString) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateNClob(int i, NClob nClob) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public NClob getNClob(int i) { return null; }
    @Override public SQLXML getSQLXML(int i) { return null; }
    @Override public void updateSQLXML(int i, SQLXML xmlObject) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public String getNString(int i) throws SQLException { return getString(i); }
    @Override public java.io.Reader getNCharacterStream(int i) { return null; }
    @Override public void updateNCharacterStream(int i, java.io.Reader x, long length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateAsciiStream(int i, java.io.InputStream x, long length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateBinaryStream(int i, java.io.InputStream x, long length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateCharacterStream(int i, java.io.Reader x, long length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateBlob(int i, java.io.InputStream inputStream, long length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateClob(int i, java.io.Reader reader, long length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateNClob(int i, java.io.Reader reader, long length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateNCharacterStream(int i, java.io.Reader x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateAsciiStream(int i, java.io.InputStream x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateBinaryStream(int i, java.io.InputStream x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateCharacterStream(int i, java.io.Reader x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateBlob(int i, java.io.InputStream inputStream) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateClob(int i, java.io.Reader reader) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateNClob(int i, java.io.Reader reader) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public <T> T getObject(int i, Class<T> type) throws SQLException { return type.cast(getObject(i)); }
    // Label-based variants simply delegate
    @Override public boolean getBoolean(String c) throws SQLException { return getBoolean(findColumn(c)); }
    @Override public byte getByte(String c) throws SQLException { return getByte(findColumn(c)); }
    @Override public short getShort(String c) throws SQLException { return getShort(findColumn(c)); }
    @Override public int getInt(String c) throws SQLException { return getInt(findColumn(c)); }
    @Override public long getLong(String c) throws SQLException { return getLong(findColumn(c)); }
    @Override public float getFloat(String c) throws SQLException { return getFloat(findColumn(c)); }
    @Override public double getDouble(String c) throws SQLException { return getDouble(findColumn(c)); }
    @Override public BigDecimal getBigDecimal(String c, int scale) throws SQLException { return getBigDecimal(findColumn(c)); }
    @Override public byte[] getBytes(String c) throws SQLException { return getBytes(findColumn(c)); }
    @Override public Date getDate(String c) throws SQLException { return getDate(findColumn(c)); }
    @Override public Time getTime(String c) throws SQLException { return getTime(findColumn(c)); }
    @Override public Timestamp getTimestamp(String c) throws SQLException { return getTimestamp(findColumn(c)); }
    @Override public java.io.InputStream getAsciiStream(String c) { return null; }
    @Override public java.io.InputStream getUnicodeStream(String c) { return null; }
    @Override public java.io.InputStream getBinaryStream(String c) { return null; }
    @Override public Object getObject(String c) throws SQLException { return getObject(findColumn(c)); }
    @Override public java.io.Reader getCharacterStream(String c) { return null; }
    @Override public BigDecimal getBigDecimal(String c) throws SQLException { return getBigDecimal(findColumn(c)); }
    @Override public BigDecimal getBigDecimal(int i, int scale) throws SQLException { return getBigDecimal(i); }
    @Override public Object getObject(String c, Map<String,Class<?>> map) throws SQLException { return getObject(c); }
    @Override public Ref getRef(String c) { return null; }
    @Override public Blob getBlob(String c) { return null; }
    @Override public Clob getClob(String c) { return null; }
    @Override public Array getArray(String c) { return null; }
    @Override public Date getDate(String c, Calendar cal) { return null; }
    @Override public Time getTime(String c, Calendar cal) { return null; }
    @Override public Timestamp getTimestamp(String c, Calendar cal) { return null; }
    @Override public URL getURL(String c) { return null; }
    @Override public RowId getRowId(String c) { return null; }
    @Override public NClob getNClob(String c) { return null; }
    @Override public SQLXML getSQLXML(String c) { return null; }
    @Override public String getNString(String c) throws SQLException { return getString(c); }
    @Override public java.io.Reader getNCharacterStream(String c) { return null; }
    @Override public <T> T getObject(String c, Class<T> type) throws SQLException { return getObject(findColumn(c), type); }
    // Update-by-label stubs
    @Override public void updateNull(String c) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateBoolean(String c, boolean x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateByte(String c, byte x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateShort(String c, short x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateInt(String c, int x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateLong(String c, long x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateFloat(String c, float x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateDouble(String c, double x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateBigDecimal(String c, BigDecimal x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateString(String c, String x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateBytes(String c, byte[] x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateDate(String c, Date x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateTime(String c, Time x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateTimestamp(String c, Timestamp x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateAsciiStream(String c, java.io.InputStream x, int length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateBinaryStream(String c, java.io.InputStream x, int length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateCharacterStream(String c, java.io.Reader x, int length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateObject(String c, Object x, int scaleOrLength) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateObject(String c, Object x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateRef(String c, Ref x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateBlob(String c, Blob x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateClob(String c, Clob x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateArray(String c, Array x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateRowId(String c, RowId x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateNString(String c, String nString) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateNClob(String c, NClob nClob) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateSQLXML(String c, SQLXML xmlObject) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateNCharacterStream(String c, java.io.Reader x, long length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateAsciiStream(String c, java.io.InputStream x, long length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateBinaryStream(String c, java.io.InputStream x, long length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateCharacterStream(String c, java.io.Reader x, long length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateBlob(String c, java.io.InputStream inputStream, long length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateClob(String c, java.io.Reader reader, long length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateNClob(String c, java.io.Reader reader, long length) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateNCharacterStream(String c, java.io.Reader x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateAsciiStream(String c, java.io.InputStream x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateBinaryStream(String c, java.io.InputStream x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateCharacterStream(String c, java.io.Reader x) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateBlob(String c, java.io.InputStream inputStream) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateClob(String c, java.io.Reader reader) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public void updateNClob(String c, java.io.Reader reader) throws SQLException { throw new SQLFeatureNotSupportedException(); }
    @Override public <T> T unwrap(Class<T> iface) throws SQLException { throw new SQLException(); }
    @Override public boolean isWrapperFor(Class<?> iface) { return false; }
}
