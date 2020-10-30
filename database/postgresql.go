// +build postgres

package database

import (
  "fmt"
  _ "github.com/jinzhu/gorm/dialects/postgres"
)

const Dialect = "postgres"

func (config Config) connectionStr() string {
  var host string
  if config.Host != "" {
    host = config.Host
  } else {
    host = localhost
  }

  var sslMode string
  if config.SSL {
    sslMode = "enable"
  } else {
    sslMode = "disable"
  }

  return fmt.Sprintf("user=%s password=%s host=%s port=%d dbname=%s sslmode=%s TimeZone=%s",
    config.User, config.Password, host, config.Port, config.Database, sslMode, config.Timezone)
}
