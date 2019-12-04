package api

import (
	"database/sql"
)

func makeAPIQueries(db *sql.DB) map[string]*sql.Stmt {
	const table = "robots"

	apiQueries := map[string]*sql.Stmt{
		"byName":   prepareSelectAll(db, table, equalsCondition("name")),
		"byPrefix": prepareSelectAll(db, table, equalsCondition("prefix")),
		"byTag":    prepareSelectAll(db, table, "WHERE tags LIKE ? OR tags LIKE ? OR tags LIKE ? OR tags LIKE ?"),
		"latest":   prepareSelectAll(db, table, "ORDER BY timestamp DESC LIMIT ?"),
	}

	addNumericQueries(db, table, apiQueries, "ID", "id")
	addNumericQueries(db, table, apiQueries, "Number", "number")
	addNumericQueries(db, table, apiQueries, "Timestamp", "timestamp")

	return apiQueries
}

func closeAPIQueries(queries map[string]*sql.Stmt) {
	for _, stmt := range queries {
		stmt.Close()
	}
}

func equalsCondition(attribute string) string {
	return "WHERE " + attribute + " = ?"
}

func fromCondition(attribute string) string {
	return "WHERE " + attribute + " >= ?"
}

func toCondition(attribute string) string {
	return "WHERE " + attribute + " <= ?"
}

func rangeCondition(attribute string) string {
	return "WHERE " + attribute + " >= ? AND " + attribute + " <= ?"
}

func addNumericQueries(db *sql.DB, table string, queries map[string]*sql.Stmt, keyName string, columnName string) {
	queries["by"+keyName] = prepareSelectAll(db, table, equalsCondition(columnName))
	queries["from"+keyName] = prepareSelectAll(db, table, fromCondition(columnName))
	queries["to"+keyName] = prepareSelectAll(db, table, toCondition(columnName))
	queries["range"+keyName] = prepareSelectAll(db, table, rangeCondition(columnName))
}
