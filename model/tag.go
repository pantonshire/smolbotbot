package model

import "github.com/jinzhu/gorm"

type Tag struct {
  gorm.Model
  RobotID uint   `gorm:"not null"`
  Text    string `gorm:"not null"`
}

func (Tag) ForeignKeys() []ForeignKey {
  return []ForeignKey{
    {column: "robot_id", delete: Cascade, update: Cascade},
  }
}
