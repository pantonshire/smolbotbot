package smolapi

import (
	"database/sql"
)

func makeRobotQueries(db *sql.DB) map[string]*sql.Stmt {
	const table = "robots"

	robotQueries := map[string]*sql.Stmt{
		"byname":   prepareSelectAll(db, table, equalsCondition("name")),
		"byprefix": prepareSelectAll(db, table, equalsCondition("prefix")),
		"bytag":    prepareSelectAll(db, table, "WHERE tags LIKE ? OR tags LIKE ? OR tags LIKE ? OR tags LIKE ?"),
		"latest":   prepareSelectAll(db, table, "ORDER BY timestamp DESC LIMIT ?"),
	}

	addNumericQueries(db, table, robotQueries, "id")
	addNumericQueries(db, table, robotQueries, "number")
	addNumericQueries(db, table, robotQueries, "timestamp")

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

func addNumericQueries(db *sql.DB, table string, queries map[string]*sql.Stmt, columnName string) {
	queries["by"+columnName] = prepareSelectAll(db, table, equalsCondition(columnName))
	queries["from"+columnName] = prepareSelectAll(db, table, fromCondition(columnName))
	queries["to"+columnName] = prepareSelectAll(db, table, toCondition(columnName))
	queries["range"+columnName] = prepareSelectAll(db, table, rangeCondition(columnName))
}
