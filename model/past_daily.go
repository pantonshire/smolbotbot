package model

import (
  "github.com/jinzhu/gorm"
  "time"
)

type PastDaily struct {
  gorm.Model
  RobotID uint      `gorm:"not null"`
  Date    time.Time `gorm:"not null"`
}

func (PastDaily) ForeignKeys() []ForeignKey {
  return []ForeignKey{
    {column: "robot_id", delete: Cascade, update: Cascade},
  }
}
