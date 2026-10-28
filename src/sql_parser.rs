//! Minimal hand-written SQL subset parser (no external sqlparser dependency).
//! Compatible with older toolchains.

#[derive(Debug, Clone)]
pub struct ParsedQuery {
    pub columns: Vec<String>,
    pub table: String,
    pub where_clause: Option<WhereClause>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct WhereClause {
    pub column: String,
    pub op: String,
    pub value: String,
}

#[derive(Debug)]
pub enum ParseError {
    Unsupported(String),
    Syntax(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Unsupported(s) => write!(f, "Unsupported SQL feature: {}", s),
            ParseError::Syntax(s) => write!(f, "SQL syntax error: {}", s),
        }
    }
}

impl std::error::Error for ParseError {}

pub fn parse_sql(sql: &str) -> Result<ParsedQuery, ParseError> {
    let sql = sql.trim();
    let upper = sql.to_uppercase();

    if !upper.starts_with("SELECT") {
        return Err(ParseError::Unsupported(
            "only SELECT statements are supported".into(),
        ));
    }

    let from_pos = upper
        .find(" FROM ")
        .ok_or_else(|| ParseError::Syntax("missing FROM".into()))?;

    let select_part = sql[6..from_pos].trim();
    let rest = &sql[from_pos + 6..];

    let columns: Vec<String> = if select_part == "*" || select_part.is_empty() {
        vec!["*".into()]
    } else {
        select_part
            .split(',')
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty())
            .collect()
    };

    let mut table = rest.trim().to_string();
    let mut where_clause = None;
    let mut limit = None;

    let upper_rest = table.to_uppercase();

    if let Some(limit_pos) = upper_rest.rfind(" LIMIT ") {
        let limit_str = table[limit_pos + 7..].trim();
        limit = Some(
            limit_str
                .parse::<usize>()
                .map_err(|_| ParseError::Syntax(format!("invalid LIMIT: {}", limit_str)))?,
        );
        table = table[..limit_pos].trim().to_string();
    }

    let upper_table = table.to_uppercase();
    if let Some(where_pos) = upper_table.find(" WHERE ") {
        let where_str = table[where_pos + 7..].trim().to_string();
        table = table[..where_pos].trim().to_string();
        where_clause = Some(parse_where(&where_str)?);
    }

    let u = table.to_uppercase();
    if u.contains(" JOIN ") || u.contains(" GROUP BY ") || u.contains(" ORDER BY ") {
        return Err(ParseError::Unsupported(
            "JOIN / GROUP BY / ORDER BY are not supported in MVP".into(),
        ));
    }

    if table.is_empty() {
        return Err(ParseError::Syntax("empty table name".into()));
    }

    Ok(ParsedQuery {
        columns,
        table,
        where_clause,
        limit,
    })
}

fn parse_where(s: &str) -> Result<WhereClause, ParseError> {
    let ops = ["<=", ">=", "!=", "<>", "=", "<", ">"];
    for op in ops {
        if let Some(pos) = s.find(op) {
            let col = s[..pos].trim().to_string();
            let val = s[pos + op.len()..]
                .trim()
                .trim_matches('\'')
                .trim_matches('"')
                .to_string();
            if col.is_empty() || val.is_empty() {
                return Err(ParseError::Syntax("invalid WHERE clause".into()));
            }
            return Ok(WhereClause {
                column: col,
                op: op.to_string(),
                value: val,
            });
        }
    }
    Err(ParseError::Syntax(format!("unsupported WHERE: {}", s)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_select() {
        let q = parse_sql("SELECT * FROM iris_model LIMIT 10").unwrap();
        assert_eq!(q.table, "iris_model");
        assert_eq!(q.limit, Some(10));
        assert!(q.where_clause.is_none());
    }

    #[test]
    fn with_where() {
        let q = parse_sql("SELECT score FROM model WHERE feature1 > 0.5 LIMIT 5").unwrap();
        assert_eq!(q.table, "model");
        assert_eq!(q.limit, Some(5));
        let w = q.where_clause.unwrap();
        assert_eq!(w.column, "feature1");
        assert_eq!(w.op, ">");
        assert_eq!(w.value, "0.5");
    }

    #[test]
    fn rejects_join() {
        assert!(parse_sql("SELECT * FROM a JOIN b ON a.id = b.id").is_err());
    }
}
