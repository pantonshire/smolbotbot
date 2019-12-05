package smolapi

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

// NewRouter creates a new chi router for the API that can be mounted into the main router.
func (api API) NewRouter() http.Handler {
	router := chi.NewRouter()

	router.Get("/latest/{n}", func(writer http.ResponseWriter, request *http.Request) {
		if n, err := integerURLParam(writer, request, "n"); err == nil {
			api.latest(writer, request, n)
		}
	})

	router.Get("/latest", func(writer http.ResponseWriter, request *http.Request) {
		api.latest(writer, request, 1)
	})

	router.Get("/name/{name}", func(writer http.ResponseWriter, request *http.Request) {
		api.simpleQuery(writer, request, "byname", "name")
	})

	router.Get("/prefix/{prefix}", func(writer http.ResponseWriter, request *http.Request) {
		api.simpleQuery(writer, request, "byprefix", "prefix")
	})

	router.Get("/tag/{tag}", func(writer http.ResponseWriter, request *http.Request) {
		tag := chi.URLParam(request, "tag")
		centre, left, right := "% "+tag+" %", tag+" %", "% "+tag
		result := runSelectRobots(api.Database, api.RobotQueries["bytag"], tag, centre, left, right)
		robotsResponse(writer, request, result)
	})

	for _, valueName := range []string{"id", "number", "timestamp"} {
		router.Get("/"+valueName+"/from/{from}/to/{to}", func(writer http.ResponseWriter, request *http.Request) {
			api.numericValueRange(writer, request, valueName)
		})

		router.Get("/"+valueName+"/from/{from}", func(writer http.ResponseWriter, request *http.Request) {
			api.numericValue(writer, request, "from", valueName)
		})

		router.Get("/"+valueName+"/to/{to}", func(writer http.ResponseWriter, request *http.Request) {
			api.numericValue(writer, request, "to", valueName)
		})

		router.Get("/"+valueName+"/{by}", func(writer http.ResponseWriter, request *http.Request) {
			api.numericValue(writer, request, "by", valueName)
		})
	}

	return router
}

func robotsResponse(writer http.ResponseWriter, request *http.Request, robots []Robot) {
	writer.Header().Set("Content-Type", "application/json")
	json.NewEncoder(writer).Encode(robots)
}

func integerURLParam(writer http.ResponseWriter, request *http.Request, paramName string) (int, error) {
	n, err := strconv.Atoi(chi.URLParam(request, paramName))

	if err != nil {
		http.NotFound(writer, request)
	}

	return n, err
}

func (api API) simpleQuery(writer http.ResponseWriter, request *http.Request, query string, paramName string) {
	param := chi.URLParam(request, paramName)
	queryResult := runSelectRobots(api.Database, api.RobotQueries[query], param)
	robotsResponse(writer, request, queryResult)
}

func (api API) latest(writer http.ResponseWriter, request *http.Request, numRobots int) {
	robotsResponse(writer, request, runSelectRobots(api.Database, api.RobotQueries["latest"], numRobots))
}

func (api API) numericValueRange(writer http.ResponseWriter, request *http.Request, valueName string) {
	if from, err := integerURLParam(writer, request, "from"); err == nil {
		if to, err := integerURLParam(writer, request, "to"); err == nil {
			result := runSelectRobots(api.Database, api.RobotQueries["range"+valueName], from, to)
			robotsResponse(writer, request, result)
		}
	}
}

func (api API) numericValue(writer http.ResponseWriter, request *http.Request, searchType string, valueName string) {
	if value, err := integerURLParam(writer, request, searchType); err == nil {
		result := runSelectRobots(api.Database, api.RobotQueries[searchType+valueName], value)
		robotsResponse(writer, request, result)
	}
}
