package model

import (
  "github.com/jinzhu/gorm"
  "time"
)

type Tweet struct {
  gorm.Model
  TweetID    uint64    `gorm:"type:bigint;not null;unique_index"`
  Time       time.Time `gorm:"not null"`
  UserID     uint64    `gorm:"type:bigint;not null"`
  UserHandle string    `gorm:"not null"`
}
