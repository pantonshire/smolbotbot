// +build mysql

package database

import (
  "fmt"
  _ "github.com/jinzhu/gorm/dialects/mysql"
)

const Dialect = "mysql"

func (config Config) connectionStr() string {
  var host string
  if config.Host != "" {
    host = config.Host
  } else {
    host = localhost
  }

  var socket string
  if config.Port != 0 {
    socket = fmt.Sprintf("%s:%d", host, config.Port)
  } else {
    socket = host
  }

  return fmt.Sprintf("%s:%s@(%s)/%s?charset=%s&parseTime=True&loc=%s",
    config.User, config.Password, socket, config.Database, config.Charset, config.Timezone)
}
