package model

import (
  "github.com/jinzhu/gorm"
  "time"
)

type ScheduledDaily struct {
  gorm.Model
  RobotID  uint      `gorm:"not null"`
  Date     time.Time `gorm:"not null"`
  Priority int       `gorm:"not null"`
}

func (ScheduledDaily) ForeignKeys() []ForeignKey {
  return []ForeignKey{
    {column: "robot_id", delete: Cascade, update: Cascade},
  }
}
