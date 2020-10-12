package model

import (
  "github.com/jinzhu/gorm"
  "time"
)

type Robot struct {
  gorm.Model
  Number      int       `gorm:"not null"`
  Name        string    `gorm:"not null"`
  Prefix      string    `gorm:"not null"`
  TweetID     uint64    `gorm:"type:bigint;not null"`
  Time        time.Time `gorm:"not null"`
  Description string    `gorm:"type:text;not null"`
  Alt         string    `gorm:"type:text"`
  ImageURL    string
}
