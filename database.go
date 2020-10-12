package smolbotbot

import (
  "fmt"
  "github.com/jinzhu/gorm"
  _ "github.com/jinzhu/gorm/dialects/mysql"
  "smolbotbot/model"
)

func ConnectMysql(config DatabaseConfig) (*gorm.DB, error) {
  db, err := gorm.Open("mysql", config.MysqlURI())
  if err != nil {
    return nil, err
  }
  db.BlockGlobalUpdate(true)
  if config.Debug {
    db.LogMode(true)
  }
  return db, nil
}

func Migrate(db *gorm.DB) error {
  models := model.Models()
  return db.Transaction(func(tx *gorm.DB) error {
    db.AutoMigrate(models...)
    for _, m := range models {
      if relational, ok := m.(model.Relational); ok {
        for _, key := range relational.ForeignKeys() {
          if err := key.Apply(tx.Model(m)); err != nil {
           return err
          }
        }
      }
    }
    return nil
  })
}

func (config DatabaseConfig) MysqlURI() string {
  return fmt.Sprintf("%s:%s@(%s)/%s?charset=%s&parseTime=True&loc=%s",
    config.User, config.Password, config.Host, config.Database, config.GetCharset(), config.GetLoc())
}

func (config DatabaseConfig) GetCharset() string {
  const defaultCharset = "utf8mb4"
  if config.Charset != "" {
    return config.Charset
  }
  return defaultCharset
}

func (config DatabaseConfig) GetLoc() string {
  const defaultLoc = "Local"
  if config.Loc != "" {
    return config.Loc
  }
  return defaultLoc
}
