package api

import (
	"database/sql"
)

func makeRobotQueries(db *sql.DB) map[string]*sql.Stmt {
	const table = "robots"

	robotQueries := map[string]*sql.Stmt{
		"byName":   prepareSelectAll(db, table, equalsCondition("name")),
		"byPrefix": prepareSelectAll(db, table, equalsCondition("prefix")),
		"byTag":    prepareSelectAll(db, table, "WHERE tags LIKE ? OR tags LIKE ? OR tags LIKE ? OR tags LIKE ?"),
		"latest":   prepareSelectAll(db, table, "ORDER BY timestamp DESC LIMIT ?"),
	}

	addNumericQueries(db, table, robotQueries, "ID", "id")
	addNumericQueries(db, table, robotQueries, "Number", "number")
	addNumericQueries(db, table, robotQueries, "Timestamp", "timestamp")

	return robotQueries
}

func closeRobotQueries(queries map[string]*sql.Stmt) {
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
