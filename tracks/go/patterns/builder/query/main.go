// Builder Pattern: SQL Query Builder with Fluent Chaining
//
// Demonstrates a fluent builder that constructs validated SQL queries.
// Each method returns *QueryBuilder for chaining. The final Build()
// method validates the query and returns the SQL string + parameters.
//
// Run: go run ./query/

package main

import (
	"fmt"
	"strings"
)

// --- Types ---

// SortDirection represents ascending or descending order.
type SortDirection int

const (
	Asc  SortDirection = iota
	Desc
)

func (d SortDirection) String() string {
	if d == Desc {
		return "DESC"
	}
	return "ASC"
}

// whereClause holds a single WHERE condition with its bind parameters.
type whereClause struct {
	condition string
	args      []any
}

// orderByClause holds a column name and sort direction.
type orderByClause struct {
	column    string
	direction SortDirection
}

// joinClause holds a JOIN specification.
type joinClause struct {
	joinType string // "INNER", "LEFT", "RIGHT"
	table    string
	on       string
}

// --- QueryBuilder ---

// QueryBuilder constructs SQL SELECT queries using fluent method chaining.
// It accumulates clauses and produces a validated query string at Build() time.
type QueryBuilder struct {
	selects  []string
	table    string
	joins    []joinClause
	wheres   []whereClause
	groupBys []string
	havings  []whereClause
	orderBys []orderByClause
	limit    int
	offset   int
	distinct bool
	err      error // Captures first error; subsequent methods short-circuit
}

// NewQuery creates a new empty QueryBuilder.
func NewQuery() *QueryBuilder {
	return &QueryBuilder{
		limit:  -1, // -1 means no limit
		offset: -1,
	}
}

// Select specifies the columns to retrieve.
func (qb *QueryBuilder) Select(columns ...string) *QueryBuilder {
	if qb.err != nil {
		return qb
	}
	if len(columns) == 0 {
		qb.err = fmt.Errorf("SELECT requires at least one column")
		return qb
	}
	for _, col := range columns {
		if col == "" {
			qb.err = fmt.Errorf("SELECT column name cannot be empty")
			return qb
		}
	}
	qb.selects = append(qb.selects, columns...)
	return qb
}

// Distinct adds DISTINCT to the SELECT clause.
func (qb *QueryBuilder) Distinct() *QueryBuilder {
	if qb.err != nil {
		return qb
	}
	qb.distinct = true
	return qb
}

// From specifies the table to query.
func (qb *QueryBuilder) From(table string) *QueryBuilder {
	if qb.err != nil {
		return qb
	}
	if table == "" {
		qb.err = fmt.Errorf("FROM table name cannot be empty")
		return qb
	}
	qb.table = table
	return qb
}

// Join adds an INNER JOIN clause.
func (qb *QueryBuilder) Join(table, on string) *QueryBuilder {
	return qb.addJoin("INNER", table, on)
}

// LeftJoin adds a LEFT JOIN clause.
func (qb *QueryBuilder) LeftJoin(table, on string) *QueryBuilder {
	return qb.addJoin("LEFT", table, on)
}

func (qb *QueryBuilder) addJoin(joinType, table, on string) *QueryBuilder {
	if qb.err != nil {
		return qb
	}
	if table == "" || on == "" {
		qb.err = fmt.Errorf("JOIN requires both table and ON condition")
		return qb
	}
	qb.joins = append(qb.joins, joinClause{
		joinType: joinType,
		table:    table,
		on:       on,
	})
	return qb
}

// Where adds a WHERE condition. Multiple calls are combined with AND.
// Use ? as a placeholder for parameters.
func (qb *QueryBuilder) Where(condition string, args ...any) *QueryBuilder {
	if qb.err != nil {
		return qb
	}
	if condition == "" {
		qb.err = fmt.Errorf("WHERE condition cannot be empty")
		return qb
	}
	qb.wheres = append(qb.wheres, whereClause{
		condition: condition,
		args:      args,
	})
	return qb
}

// GroupBy specifies columns to group by.
func (qb *QueryBuilder) GroupBy(columns ...string) *QueryBuilder {
	if qb.err != nil {
		return qb
	}
	qb.groupBys = append(qb.groupBys, columns...)
	return qb
}

// Having adds a HAVING condition (used with GROUP BY).
func (qb *QueryBuilder) Having(condition string, args ...any) *QueryBuilder {
	if qb.err != nil {
		return qb
	}
	qb.havings = append(qb.havings, whereClause{
		condition: condition,
		args:      args,
	})
	return qb
}

// OrderBy adds a column to the ORDER BY clause.
func (qb *QueryBuilder) OrderBy(column string, dir SortDirection) *QueryBuilder {
	if qb.err != nil {
		return qb
	}
	if column == "" {
		qb.err = fmt.Errorf("ORDER BY column name cannot be empty")
		return qb
	}
	qb.orderBys = append(qb.orderBys, orderByClause{
		column:    column,
		direction: dir,
	})
	return qb
}

// Limit sets the maximum number of rows to return.
func (qb *QueryBuilder) Limit(n int) *QueryBuilder {
	if qb.err != nil {
		return qb
	}
	if n < 0 {
		qb.err = fmt.Errorf("LIMIT must be non-negative, got %d", n)
		return qb
	}
	qb.limit = n
	return qb
}

// Offset sets the number of rows to skip.
func (qb *QueryBuilder) Offset(n int) *QueryBuilder {
	if qb.err != nil {
		return qb
	}
	if n < 0 {
		qb.err = fmt.Errorf("OFFSET must be non-negative, got %d", n)
		return qb
	}
	qb.offset = n
	return qb
}

// Build validates the accumulated clauses and produces the SQL string
// and bind parameters. Returns an error if the query is invalid.
func (qb *QueryBuilder) Build() (string, []any, error) {
	// Check for errors accumulated during chaining
	if qb.err != nil {
		return "", nil, fmt.Errorf("query build error: %w", qb.err)
	}

	// Validate required clauses
	if len(qb.selects) == 0 {
		return "", nil, fmt.Errorf("query requires SELECT columns")
	}
	if qb.table == "" {
		return "", nil, fmt.Errorf("query requires FROM table")
	}
	if len(qb.havings) > 0 && len(qb.groupBys) == 0 {
		return "", nil, fmt.Errorf("HAVING requires GROUP BY")
	}

	var sql strings.Builder
	var allArgs []any

	// SELECT
	sql.WriteString("SELECT ")
	if qb.distinct {
		sql.WriteString("DISTINCT ")
	}
	sql.WriteString(strings.Join(qb.selects, ", "))

	// FROM
	sql.WriteString(" FROM ")
	sql.WriteString(qb.table)

	// JOINs
	for _, j := range qb.joins {
		fmt.Fprintf(&sql, " %s JOIN %s ON %s", j.joinType, j.table, j.on)
	}

	// WHERE
	if len(qb.wheres) > 0 {
		sql.WriteString(" WHERE ")
		conditions := make([]string, len(qb.wheres))
		for i, w := range qb.wheres {
			conditions[i] = w.condition
			allArgs = append(allArgs, w.args...)
		}
		sql.WriteString(strings.Join(conditions, " AND "))
	}

	// GROUP BY
	if len(qb.groupBys) > 0 {
		sql.WriteString(" GROUP BY ")
		sql.WriteString(strings.Join(qb.groupBys, ", "))
	}

	// HAVING
	if len(qb.havings) > 0 {
		sql.WriteString(" HAVING ")
		conditions := make([]string, len(qb.havings))
		for i, h := range qb.havings {
			conditions[i] = h.condition
			allArgs = append(allArgs, h.args...)
		}
		sql.WriteString(strings.Join(conditions, " AND "))
	}

	// ORDER BY
	if len(qb.orderBys) > 0 {
		sql.WriteString(" ORDER BY ")
		orders := make([]string, len(qb.orderBys))
		for i, o := range qb.orderBys {
			orders[i] = fmt.Sprintf("%s %s", o.column, o.direction)
		}
		sql.WriteString(strings.Join(orders, ", "))
	}

	// LIMIT
	if qb.limit >= 0 {
		fmt.Fprintf(&sql, " LIMIT %d", qb.limit)
	}

	// OFFSET
	if qb.offset >= 0 {
		fmt.Fprintf(&sql, " OFFSET %d", qb.offset)
	}

	return sql.String(), allArgs, nil
}

// String returns a debug-friendly representation of the query.
func (qb *QueryBuilder) String() string {
	sql, args, err := qb.Build()
	if err != nil {
		return fmt.Sprintf("INVALID QUERY: %v", err)
	}
	if len(args) > 0 {
		return fmt.Sprintf("%s  [args: %v]", sql, args)
	}
	return sql
}

// --- Main ---

func main() {
	fmt.Println("=== SQL Query Builder with Fluent Chaining ===")
	fmt.Println(strings.Repeat("-", 55))

	// Example 1: Simple select
	fmt.Println("\n1. Simple SELECT:")
	sql, args, err := NewQuery().
		Select("name", "email").
		From("users").
		Where("active = ?", true).
		OrderBy("name", Asc).
		Limit(10).
		Build()

	if err != nil {
		fmt.Printf("   Error: %v\n", err)
	} else {
		fmt.Printf("   SQL:  %s\n", sql)
		fmt.Printf("   Args: %v\n", args)
	}

	// Example 2: Complex query with joins
	fmt.Println("\n2. JOIN query:")
	sql, args, err = NewQuery().
		Select("u.name", "u.email", "o.total", "o.created_at").
		From("users u").
		Join("orders o", "o.user_id = u.id").
		LeftJoin("addresses a", "a.user_id = u.id").
		Where("u.active = ?", true).
		Where("o.total > ?", 100.00).
		OrderBy("o.created_at", Desc).
		Limit(25).
		Offset(50).
		Build()

	if err != nil {
		fmt.Printf("   Error: %v\n", err)
	} else {
		fmt.Printf("   SQL:  %s\n", sql)
		fmt.Printf("   Args: %v\n", args)
	}

	// Example 3: Aggregation with GROUP BY and HAVING
	fmt.Println("\n3. Aggregation query:")
	sql, args, err = NewQuery().
		Select("department", "COUNT(*) as headcount", "AVG(salary) as avg_salary").
		From("employees").
		Where("hire_date > ?", "2024-01-01").
		GroupBy("department").
		Having("COUNT(*) >= ?", 5).
		OrderBy("avg_salary", Desc).
		Build()

	if err != nil {
		fmt.Printf("   Error: %v\n", err)
	} else {
		fmt.Printf("   SQL:  %s\n", sql)
		fmt.Printf("   Args: %v\n", args)
	}

	// Example 4: DISTINCT
	fmt.Println("\n4. DISTINCT query:")
	sql, _, err = NewQuery().
		Distinct().
		Select("department").
		From("employees").
		OrderBy("department", Asc).
		Build()

	if err != nil {
		fmt.Printf("   Error: %v\n", err)
	} else {
		fmt.Printf("   SQL:  %s\n", sql)
	}

	// Example 5: Validation errors
	fmt.Println("\n5. Validation errors:")

	// Missing FROM
	_, _, err = NewQuery().
		Select("name").
		Build()
	fmt.Printf("   No FROM:        %v\n", err)

	// Missing SELECT
	_, _, err = NewQuery().
		From("users").
		Build()
	fmt.Printf("   No SELECT:      %v\n", err)

	// Negative LIMIT
	_, _, err = NewQuery().
		Select("name").
		From("users").
		Limit(-5).
		Build()
	fmt.Printf("   Negative LIMIT: %v\n", err)

	// HAVING without GROUP BY
	_, _, err = NewQuery().
		Select("name").
		From("users").
		Having("COUNT(*) > ?", 1).
		Build()
	fmt.Printf("   HAVING no GROUP BY: %v\n", err)

	fmt.Println(strings.Repeat("-", 55))
	fmt.Println("\nKey takeaways:")
	fmt.Println("- Fluent chaining reads like a SQL query")
	fmt.Println("- First error short-circuits all subsequent calls")
	fmt.Println("- Build() validates the complete query")
	fmt.Println("- Pointer receivers ensure chaining works correctly")
	fmt.Println("- Parameterized queries prevent SQL injection")
}
