// Builder Pattern: SQL Query Builder with Fluent Chaining
//
// Demonstrates a consuming builder (takes `self`) that produces
// a validated SQL query with parameterized values. Shows how
// builders can accumulate complex state through method chaining.
//
// Run: rustc query.rs && ./query

use std::fmt;

// --- Target struct ---

#[derive(Debug, Clone)]
struct Query {
    sql: String,
    params: Vec<QueryParam>,
}

#[derive(Debug, Clone)]
enum QueryParam {
    Text(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

impl fmt::Display for QueryParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryParam::Text(s) => write!(f, "'{}'", s),
            QueryParam::Int(n) => write!(f, "{}", n),
            QueryParam::Float(n) => write!(f, "{}", n),
            QueryParam::Bool(b) => write!(f, "{}", b),
            QueryParam::Null => write!(f, "NULL"),
        }
    }
}

// --- Builder ---

#[derive(Debug, Clone)]
enum OrderDirection {
    Asc,
    Desc,
}

struct SelectBuilder {
    table: String,
    columns: Vec<String>,
    conditions: Vec<(String, QueryParam)>,
    order_by: Vec<(String, OrderDirection)>,
    limit: Option<u64>,
    offset: Option<u64>,
    joins: Vec<String>,
    group_by: Vec<String>,
}

impl SelectBuilder {
    fn from(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            columns: Vec::new(),
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            joins: Vec::new(),
            group_by: Vec::new(),
        }
    }

    // Column selection -- accumulating (each call adds columns)
    fn select(mut self, columns: &[&str]) -> Self {
        self.columns.extend(columns.iter().map(|c| c.to_string()));
        self
    }

    fn select_all(mut self) -> Self {
        self.columns.clear();
        self.columns.push("*".to_string());
        self
    }

    // WHERE conditions -- accumulating
    fn where_eq(mut self, column: &str, value: QueryParam) -> Self {
        self.conditions.push((format!("{} = ${}", column, self.conditions.len() + 1), value));
        self
    }

    fn where_gt(mut self, column: &str, value: QueryParam) -> Self {
        self.conditions.push((format!("{} > ${}", column, self.conditions.len() + 1), value));
        self
    }

    fn where_lt(mut self, column: &str, value: QueryParam) -> Self {
        self.conditions.push((format!("{} < ${}", column, self.conditions.len() + 1), value));
        self
    }

    fn where_like(mut self, column: &str, pattern: &str) -> Self {
        self.conditions.push((
            format!("{} LIKE ${}", column, self.conditions.len() + 1),
            QueryParam::Text(pattern.to_string()),
        ));
        self
    }

    fn where_null(mut self, column: &str) -> Self {
        self.conditions.push((format!("{} IS NULL", column), QueryParam::Null));
        self
    }

    // JOIN
    fn inner_join(mut self, table: &str, on: &str) -> Self {
        self.joins.push(format!("INNER JOIN {} ON {}", table, on));
        self
    }

    fn left_join(mut self, table: &str, on: &str) -> Self {
        self.joins.push(format!("LEFT JOIN {} ON {}", table, on));
        self
    }

    // ORDER BY -- accumulating
    fn order_by(mut self, column: &str, direction: OrderDirection) -> Self {
        self.order_by.push((column.to_string(), direction));
        self
    }

    fn order_asc(self, column: &str) -> Self {
        self.order_by(column, OrderDirection::Asc)
    }

    fn order_desc(self, column: &str) -> Self {
        self.order_by(column, OrderDirection::Desc)
    }

    // GROUP BY
    fn group_by(mut self, columns: &[&str]) -> Self {
        self.group_by.extend(columns.iter().map(|c| c.to_string()));
        self
    }

    // LIMIT / OFFSET
    fn limit(mut self, n: u64) -> Self {
        self.limit = Some(n);
        self
    }

    fn offset(mut self, n: u64) -> Self {
        self.offset = Some(n);
        self
    }

    // Pagination helper
    fn page(self, page: u64, page_size: u64) -> Self {
        let offset = page.saturating_sub(1) * page_size;
        self.limit(page_size).offset(offset)
    }

    // Build: produces the validated query
    fn build(self) -> Result<Query, String> {
        if self.columns.is_empty() {
            return Err("no columns selected -- use .select() or .select_all()".to_string());
        }

        let mut sql = String::new();
        let mut params = Vec::new();

        // SELECT
        sql.push_str("SELECT ");
        sql.push_str(&self.columns.join(", "));

        // FROM
        sql.push_str(" FROM ");
        sql.push_str(&self.table);

        // JOINs
        for join in &self.joins {
            sql.push(' ');
            sql.push_str(join);
        }

        // WHERE
        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            let mut clause_parts = Vec::new();
            for (clause, param) in &self.conditions {
                clause_parts.push(clause.clone());
                if !matches!(param, QueryParam::Null) {
                    params.push(param.clone());
                }
            }
            sql.push_str(&clause_parts.join(" AND "));
        }

        // GROUP BY
        if !self.group_by.is_empty() {
            sql.push_str(" GROUP BY ");
            sql.push_str(&self.group_by.join(", "));
        }

        // ORDER BY
        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let parts: Vec<String> = self.order_by.iter().map(|(col, dir)| {
                let dir_str = match dir {
                    OrderDirection::Asc => "ASC",
                    OrderDirection::Desc => "DESC",
                };
                format!("{} {}", col, dir_str)
            }).collect();
            sql.push_str(&parts.join(", "));
        }

        // LIMIT
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        // OFFSET
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        Ok(Query { sql, params })
    }
}

// --- Demo ---

fn main() {
    // Simple query
    let query = SelectBuilder::from("users")
        .select(&["id", "name", "email"])
        .where_eq("active", QueryParam::Bool(true))
        .order_asc("name")
        .limit(50)
        .build()
        .expect("valid query");

    println!("Simple query:");
    println!("  SQL:    {}", query.sql);
    println!("  Params: {:?}\n", query.params);

    // Complex query with join and pagination
    let query = SelectBuilder::from("orders")
        .select(&["orders.id", "users.name", "orders.total", "orders.created_at"])
        .inner_join("users", "users.id = orders.user_id")
        .where_gt("orders.total", QueryParam::Float(100.0))
        .where_eq("orders.status", QueryParam::Text("completed".to_string()))
        .order_desc("orders.created_at")
        .page(3, 25)
        .build()
        .expect("valid query");

    println!("Complex query:");
    println!("  SQL:    {}", query.sql);
    println!("  Params: {:?}\n", query.params);

    // Aggregation query
    let query = SelectBuilder::from("events")
        .select(&["event_type", "COUNT(*) as count"])
        .where_gt("created_at", QueryParam::Text("2025-01-01".to_string()))
        .group_by(&["event_type"])
        .order_desc("count")
        .build()
        .expect("valid query");

    println!("Aggregation query:");
    println!("  SQL:    {}", query.sql);
    println!("  Params: {:?}\n", query.params);

    // Error case: no columns selected
    let result = SelectBuilder::from("users").build();
    println!("Error case: {:?}", result.err());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_select() {
        let q = SelectBuilder::from("users")
            .select(&["id", "name"])
            .build()
            .unwrap();

        assert_eq!(q.sql, "SELECT id, name FROM users");
        assert!(q.params.is_empty());
    }

    #[test]
    fn test_select_all() {
        let q = SelectBuilder::from("users")
            .select_all()
            .build()
            .unwrap();

        assert_eq!(q.sql, "SELECT * FROM users");
    }

    #[test]
    fn test_where_clause() {
        let q = SelectBuilder::from("users")
            .select(&["id"])
            .where_eq("active", QueryParam::Bool(true))
            .build()
            .unwrap();

        assert!(q.sql.contains("WHERE active = $1"));
        assert_eq!(q.params.len(), 1);
    }

    #[test]
    fn test_multiple_conditions() {
        let q = SelectBuilder::from("users")
            .select(&["id"])
            .where_eq("active", QueryParam::Bool(true))
            .where_gt("age", QueryParam::Int(18))
            .build()
            .unwrap();

        assert!(q.sql.contains("WHERE active = $1 AND age > $2"));
        assert_eq!(q.params.len(), 2);
    }

    #[test]
    fn test_join() {
        let q = SelectBuilder::from("orders")
            .select(&["orders.id", "users.name"])
            .inner_join("users", "users.id = orders.user_id")
            .build()
            .unwrap();

        assert!(q.sql.contains("INNER JOIN users ON users.id = orders.user_id"));
    }

    #[test]
    fn test_order_by() {
        let q = SelectBuilder::from("users")
            .select(&["id"])
            .order_asc("name")
            .order_desc("created_at")
            .build()
            .unwrap();

        assert!(q.sql.contains("ORDER BY name ASC, created_at DESC"));
    }

    #[test]
    fn test_pagination() {
        let q = SelectBuilder::from("users")
            .select(&["id"])
            .page(3, 25)
            .build()
            .unwrap();

        assert!(q.sql.contains("LIMIT 25"));
        assert!(q.sql.contains("OFFSET 50"));
    }

    #[test]
    fn test_error_no_columns() {
        let result = SelectBuilder::from("users").build();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no columns selected"));
    }

    #[test]
    fn test_group_by() {
        let q = SelectBuilder::from("events")
            .select(&["type", "COUNT(*)"])
            .group_by(&["type"])
            .build()
            .unwrap();

        assert!(q.sql.contains("GROUP BY type"));
    }

    #[test]
    fn test_where_null() {
        let q = SelectBuilder::from("users")
            .select(&["id"])
            .where_null("deleted_at")
            .build()
            .unwrap();

        assert!(q.sql.contains("deleted_at IS NULL"));
    }
}
