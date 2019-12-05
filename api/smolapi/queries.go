package smolapi

import (
	"database/sql"
	"log"
)

func prepareSelectAll(db *sql.DB, table string, modifiers string) *sql.Stmt {
	queryStr := "SELECT * FROM " + table + " " + modifiers

	stmt, err := db.Prepare(queryStr)

	if err != nil {
		log.Panic(err)
	}

	return stmt
}

func runSelectRobots(db *sql.DB, stmt *sql.Stmt, args ...interface{}) []Robot {
	rows, err := stmt.Query(args...)

	if err != nil {
		log.Panic(err)
	}

	defer rows.Close()

	robots := make([]Robot, 0)

	for rows.Next() {
		var robot Robot
		err := rows.Scan(&robot.ID, &robot.Number, &robot.Name.Full, &robot.Name.Prefix, &robot.Tweet.ID,
			&robot.Tweet.Timestamp, &robot.Description, &robot.Image.URL, &robot.Image.Alt, &robot.Tags)

		if err != nil {
			log.Panic(err)
		}

		robots = append(robots, robot)
	}

	return robots
}
