package api

import (
	"database/sql"
	"encoding/json"
	"log"
	"net/http"
	"strconv"

	"github.com/go-chi/chi"
	// Blank import used because the MySQL must be loaded but does not need to be directly accessed.
	_ "github.com/go-sql-driver/mysql"
)

// API stores a pointer to the DB and pointers to queries needed for the api.
type API struct {
	Database     *sql.DB
	RobotQueries map[string]*sql.Stmt
}

// NewAPI creates a new database and robot queries, and packages them into an API struct.
func NewAPI(username string, password string, host string, dbname string) *API {
	db, err := sql.Open("mysql", username+":"+password+"@("+host+")/"+dbname)

	if err != nil {
		log.Panic(err)
	}

	err = db.Ping()

	if err != nil {
		log.Panic(err)
	}

	return &API{Database: db, RobotQueries: makeRobotQueries(db)}
}

// Close closes the database and queries associated with the API.
func (api API) Close() {
	defer api.Database.Close()
	defer closeRobotQueries(api.RobotQueries)
}

func robotsResponse(writer http.ResponseWriter, request *http.Request, robots []Robot) {
	writer.Header().Set("Content-Type", "application/json")
	json.NewEncoder(writer).Encode(robots)
}

func (api API) latest(writer http.ResponseWriter, request *http.Request, n int) {
	robotsResponse(writer, request, runSelectRobots(api.Database, api.RobotQueries["latest"], n))
}

// LatestRobot makes a json response of the latest small robot in the database.
func (api API) LatestRobot(writer http.ResponseWriter, request *http.Request) {
	api.latest(writer, request, 1)
}

// LatestRobots makes a json response of the latest n small robots in the database.
func (api API) LatestRobots(writer http.ResponseWriter, request *http.Request) {
	n, err := strconv.Atoi(chi.URLParam(request, "n"))

	if err != nil {
		http.NotFound(writer, request)
	} else {
		api.latest(writer, request, n)
	}
}

// ByTag makes a json response of the small robots with the tag.
func (api API) ByTag(writer http.ResponseWriter, request *http.Request) {
	tag := chi.URLParam(request, "tag")
	centre, left, right := "% "+tag+" %", "% "+tag, tag+" %"
	result := runSelectRobots(api.Database, api.RobotQueries["byTag"], tag, centre, left, right)
	robotsResponse(writer, request, result)
}
