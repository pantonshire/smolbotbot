package database

import "github.com/jinzhu/gorm"

const localhost = "localhost"

func Connect(config Config) (*gorm.DB, error) {
  db, err := gorm.Open(Dialect, config.connectionStr())
  if err != nil {
    return nil, err
  }
  db.BlockGlobalUpdate(true)
  if config.Debug {
    db.LogMode(true)
  }
  return db, nil
}
