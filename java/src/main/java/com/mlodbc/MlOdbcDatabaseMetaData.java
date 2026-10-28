package com.mlodbc;

import java.sql.*;

/** Minimal DatabaseMetaData for the ML-ODBC JDBC driver. */
public class MlOdbcDatabaseMetaData implements DatabaseMetaData {
    private final MlOdbcConnection conn;
    MlOdbcDatabaseMetaData(MlOdbcConnection conn) { this.conn = conn; }

    @Override public String getDatabaseProductName() { return "ML-ODBC"; }
    @Override public String getDatabaseProductVersion() { return "0.2.0"; }
    @Override public String getDriverName() { return "ML-ODBC JDBC Driver"; }
    @Override public String getDriverVersion() { return "0.2.0"; }
    @Override public int getDriverMajorVersion() { return 0; }
    @Override public int getDriverMinorVersion() { return 2; }
    @Override public boolean isReadOnly() { return true; }
    @Override public Connection getConnection() { return conn; }
    @Override public String getURL() { return "jdbc:mlodbc:"; }
    @Override public String getUserName() { return ""; }
    @Override public boolean supportsTransactions() { return false; }
    @Override public int getDefaultTransactionIsolation() { return Connection.TRANSACTION_NONE; }

    // Everything else returns conservative defaults / false / empty
    @Override public boolean allProceduresAreCallable() { return false; }
    @Override public boolean allTablesAreSelectable() { return true; }
    @Override public boolean nullsAreSortedHigh() { return false; }
    @Override public boolean nullsAreSortedLow() { return true; }
    @Override public boolean nullsAreSortedAtStart() { return false; }
    @Override public boolean nullsAreSortedAtEnd() { return false; }
    @Override public boolean usesLocalFiles() { return false; }
    @Override public boolean usesLocalFilePerTable() { return false; }
    @Override public boolean supportsMixedCaseIdentifiers() { return true; }
    @Override public boolean storesUpperCaseIdentifiers() { return false; }
    @Override public boolean storesLowerCaseIdentifiers() { return false; }
    @Override public boolean storesMixedCaseIdentifiers() { return true; }
    @Override public boolean supportsMixedCaseQuotedIdentifiers() { return true; }
    @Override public boolean storesUpperCaseQuotedIdentifiers() { return false; }
    @Override public boolean storesLowerCaseQuotedIdentifiers() { return false; }
    @Override public boolean storesMixedCaseQuotedIdentifiers() { return true; }
    @Override public String getIdentifierQuoteString() { return "\""; }
    @Override public String getSQLKeywords() { return ""; }
    @Override public String getNumericFunctions() { return ""; }
    @Override public String getStringFunctions() { return ""; }
    @Override public String getSystemFunctions() { return ""; }
    @Override public String getTimeDateFunctions() { return ""; }
    @Override public String getSearchStringEscape() { return "\\"; }
    @Override public String getExtraNameCharacters() { return ""; }
    @Override public boolean supportsAlterTableWithAddColumn() { return false; }
    @Override public boolean supportsAlterTableWithDropColumn() { return false; }
    @Override public boolean supportsColumnAliasing() { return true; }
    @Override public boolean nullPlusNonNullIsNull() { return true; }
    @Override public boolean supportsConvert() { return false; }
    @Override public boolean supportsConvert(int fromType, int toType) { return false; }
    @Override public boolean supportsTableCorrelationNames() { return false; }
    @Override public boolean supportsDifferentTableCorrelationNames() { return false; }
    @Override public boolean supportsExpressionsInOrderBy() { return false; }
    @Override public boolean supportsOrderByUnrelated() { return false; }
    @Override public boolean supportsGroupBy() { return false; }
    @Override public boolean supportsGroupByUnrelated() { return false; }
    @Override public boolean supportsGroupByBeyondSelect() { return false; }
    @Override public boolean supportsLikeEscapeClause() { return false; }
    @Override public boolean supportsMultipleResultSets() { return false; }
    @Override public boolean supportsMultipleTransactions() { return false; }
    @Override public boolean supportsNonNullableColumns() { return true; }
    @Override public boolean supportsMinimumSQLGrammar() { return true; }
    @Override public boolean supportsCoreSQLGrammar() { return false; }
    @Override public boolean supportsExtendedSQLGrammar() { return false; }
    @Override public boolean supportsANSI92EntryLevelSQL() { return false; }
    @Override public boolean supportsANSI92IntermediateSQL() { return false; }
    @Override public boolean supportsANSI92FullSQL() { return false; }
    @Override public boolean supportsIntegrityEnhancementFacility() { return false; }
    @Override public boolean supportsOuterJoins() { return false; }
    @Override public boolean supportsFullOuterJoins() { return false; }
    @Override public boolean supportsLimitedOuterJoins() { return false; }
    @Override public String getSchemaTerm() { return "schema"; }
    @Override public String getProcedureTerm() { return "procedure"; }
    @Override public String getCatalogTerm() { return "catalog"; }
    @Override public boolean isCatalogAtStart() { return true; }
    @Override public String getCatalogSeparator() { return "."; }
    @Override public boolean supportsSchemasInDataManipulation() { return false; }
    @Override public boolean supportsSchemasInProcedureCalls() { return false; }
    @Override public boolean supportsSchemasInTableDefinitions() { return false; }
    @Override public boolean supportsSchemasInIndexDefinitions() { return false; }
    @Override public boolean supportsSchemasInPrivilegeDefinitions() { return false; }
    @Override public boolean supportsCatalogsInDataManipulation() { return false; }
    @Override public boolean supportsCatalogsInProcedureCalls() { return false; }
    @Override public boolean supportsCatalogsInTableDefinitions() { return false; }
    @Override public boolean supportsCatalogsInIndexDefinitions() { return false; }
    @Override public boolean supportsCatalogsInPrivilegeDefinitions() { return false; }
    @Override public boolean supportsPositionedDelete() { return false; }
    @Override public boolean supportsPositionedUpdate() { return false; }
    @Override public boolean supportsSelectForUpdate() { return false; }
    @Override public boolean supportsStoredProcedures() { return false; }
    @Override public boolean supportsSubqueriesInComparisons() { return false; }
    @Override public boolean supportsSubqueriesInExists() { return false; }
    @Override public boolean supportsSubqueriesInIns() { return false; }
    @Override public boolean supportsSubqueriesInQuantifieds() { return false; }
    @Override public boolean supportsCorrelatedSubqueries() { return false; }
    @Override public boolean supportsUnion() { return false; }
    @Override public boolean supportsUnionAll() { return false; }
    @Override public boolean supportsOpenCursorsAcrossCommit() { return false; }
    @Override public boolean supportsOpenCursorsAcrossRollback() { return false; }
    @Override public boolean supportsOpenStatementsAcrossCommit() { return false; }
    @Override public boolean supportsOpenStatementsAcrossRollback() { return false; }
    @Override public int getMaxBinaryLiteralLength() { return 0; }
    @Override public int getMaxCharLiteralLength() { return 0; }
    @Override public int getMaxColumnNameLength() { return 128; }
    @Override public int getMaxColumnsInGroupBy() { return 0; }
    @Override public int getMaxColumnsInIndex() { return 0; }
    @Override public int getMaxColumnsInOrderBy() { return 0; }
    @Override public int getMaxColumnsInSelect() { return 0; }
    @Override public int getMaxColumnsInTable() { return 0; }
    @Override public int getMaxConnections() { return 0; }
    @Override public int getMaxCursorNameLength() { return 0; }
    @Override public int getMaxIndexLength() { return 0; }
    @Override public int getMaxSchemaNameLength() { return 0; }
    @Override public int getMaxProcedureNameLength() { return 0; }
    @Override public int getMaxCatalogNameLength() { return 0; }
    @Override public int getMaxRowSize() { return 0; }
    @Override public boolean doesMaxRowSizeIncludeBlobs() { return false; }
    @Override public int getMaxStatementLength() { return 0; }
    @Override public int getMaxStatements() { return 0; }
    @Override public int getMaxTableNameLength() { return 128; }
    @Override public int getMaxTablesInSelect() { return 1; }
    @Override public int getMaxUserNameLength() { return 0; }
    @Override public boolean supportsTransactionIsolationLevel(int level) { return level == Connection.TRANSACTION_NONE; }
    @Override public boolean supportsDataDefinitionAndDataManipulationTransactions() { return false; }
    @Override public boolean supportsDataManipulationTransactionsOnly() { return false; }
    @Override public boolean dataDefinitionCausesTransactionCommit() { return false; }
    @Override public boolean dataDefinitionIgnoredInTransactions() { return true; }
    @Override public ResultSet getProcedures(String c, String s, String p) { return null; }
    @Override public ResultSet getProcedureColumns(String c, String s, String p, String col) { return null; }
    @Override public ResultSet getTables(String c, String s, String t, String[] types) throws SQLException {
        // Delegate to the native SQLTables via a statement
        Statement st = conn.createStatement();
        return st.executeQuery("SELECT * FROM iris_model WHERE 1=0"); // placeholder – real impl would call SQLTables
    }
    @Override public ResultSet getSchemas() { return null; }
    @Override public ResultSet getCatalogs() { return null; }
    @Override public ResultSet getTableTypes() { return null; }
    @Override public ResultSet getColumns(String c, String s, String t, String col) { return null; }
    @Override public ResultSet getColumnPrivileges(String c, String s, String t, String col) { return null; }
    @Override public ResultSet getTablePrivileges(String c, String s, String t) { return null; }
    @Override public ResultSet getBestRowIdentifier(String c, String s, String t, int scope, boolean nullable) { return null; }
    @Override public ResultSet getVersionColumns(String c, String s, String t) { return null; }
    @Override public ResultSet getPrimaryKeys(String c, String s, String t) { return null; }
    @Override public ResultSet getImportedKeys(String c, String s, String t) { return null; }
    @Override public ResultSet getExportedKeys(String c, String s, String t) { return null; }
    @Override public ResultSet getCrossReference(String pc, String ps, String pt, String fc, String fs, String ft) { return null; }
    @Override public ResultSet getTypeInfo() { return null; }
    @Override public ResultSet getIndexInfo(String c, String s, String t, boolean unique, boolean approximate) { return null; }
    @Override public boolean supportsResultSetType(int type) { return type == ResultSet.TYPE_FORWARD_ONLY; }
    @Override public boolean supportsResultSetConcurrency(int type, int concurrency) { return concurrency == ResultSet.CONCUR_READ_ONLY; }
    @Override public boolean ownUpdatesAreVisible(int type) { return false; }
    @Override public boolean ownDeletesAreVisible(int type) { return false; }
    @Override public boolean ownInsertsAreVisible(int type) { return false; }
    @Override public boolean othersUpdatesAreVisible(int type) { return false; }
    @Override public boolean othersDeletesAreVisible(int type) { return false; }
    @Override public boolean othersInsertsAreVisible(int type) { return false; }
    @Override public boolean updatesAreDetected(int type) { return false; }
    @Override public boolean deletesAreDetected(int type) { return false; }
    @Override public boolean insertsAreDetected(int type) { return false; }
    @Override public boolean supportsBatchUpdates() { return false; }
    @Override public ResultSet getUDTs(String c, String s, String typeName, int[] types) { return null; }
    @Override public boolean supportsSavepoints() { return false; }
    @Override public boolean supportsNamedParameters() { return false; }
    @Override public boolean supportsMultipleOpenResults() { return false; }
    @Override public boolean supportsGetGeneratedKeys() { return false; }
    @Override public ResultSet getSuperTypes(String c, String s, String typeName) { return null; }
    @Override public ResultSet getSuperTables(String c, String s, String t) { return null; }
    @Override public ResultSet getAttributes(String c, String s, String typeName, String attributeName) { return null; }
    @Override public boolean supportsResultSetHoldability(int holdability) { return false; }
    @Override public int getResultSetHoldability() { return ResultSet.CLOSE_CURSORS_AT_COMMIT; }
    @Override public int getDatabaseMajorVersion() { return 0; }
    @Override public int getDatabaseMinorVersion() { return 2; }
    @Override public int getJDBCMajorVersion() { return 4; }
    @Override public int getJDBCMinorVersion() { return 2; }
    @Override public int getSQLStateType() { return sqlStateSQL; }
    @Override public boolean locatorsUpdateCopy() { return false; }
    @Override public boolean supportsStatementPooling() { return false; }
    @Override public RowIdLifetime getRowIdLifetime() { return RowIdLifetime.ROWID_UNSUPPORTED; }
    @Override public boolean supportsStoredFunctionsUsingCallSyntax() { return false; }
    @Override public boolean autoCommitFailureClosesAllResultSets() { return false; }
    @Override public ResultSet getClientInfoProperties() { return null; }
    @Override public ResultSet getFunctions(String c, String s, String f) { return null; }
    @Override public ResultSet getFunctionColumns(String c, String s, String f, String col) { return null; }
    @Override public ResultSet getPseudoColumns(String c, String s, String t, String col) { return null; }
    @Override public boolean generatedKeyAlwaysReturned() { return false; }
    @Override public <T> T unwrap(Class<T> iface) throws SQLException { throw new SQLException(); }
    @Override public boolean isWrapperFor(Class<?> iface) { return false; }
}
