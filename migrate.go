package smolbotbot

import (
  "github.com/jinzhu/gorm"
  "github.com/pantonshire/smolbotbot/model"
)

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
