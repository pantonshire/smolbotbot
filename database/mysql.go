// +build mysql !postgre

package database

import (
  "fmt"
  _ "github.com/jinzhu/gorm/dialects/mysql"
)

const Dialect = "mysql"

const defaultTimezone = "Local"

func (config Config) connectionStr() string {
  host := config.Host
  if host == "" {
    host = localhost
  }

  var socket string
  if config.Port != 0 {
    socket = fmt.Sprintf("%s:%d", host, config.Port)
  } else {
    socket = host
  }

  timezone := config.Timezone
  if timezone == "" {
    timezone = defaultTimezone
  }

  return fmt.Sprintf("%s:%s@(%s)/%s?charset=%s&parseTime=True&loc=%s",
    config.User, config.Password, socket, config.Database, config.Charset, timezone)
}
