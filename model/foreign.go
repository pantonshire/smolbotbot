package model

import (
  "fmt"
  "github.com/jinzhu/gorm"
  "strings"
)

type ForeignKeyConstraint string

const (
  Restrict ForeignKeyConstraint = "restrict"
  Cascade  ForeignKeyConstraint = "cascade"
  SetNull  ForeignKeyConstraint = "set null"
)

type ForeignKey struct {
  column  string
  foreign string
  delete  ForeignKeyConstraint
  update  ForeignKeyConstraint
}

func (fk ForeignKey) Apply(db *gorm.DB) error {
  var foreign string
  if fk.foreign != "" {
    foreign = fk.foreign
  } else {
    foreign = fk.defaultForeign()
  }
  return db.AddForeignKey(fk.column, foreign, string(fk.delete), string(fk.update)).Error
}

func (fk ForeignKey) DummyApply() {
  var foreign string
  if fk.foreign != "" {
    foreign = fk.foreign
  } else {
    foreign = fk.defaultForeign()
  }
  fmt.Println(fmt.Sprintf("Apply foreign key: %s REFERENCES %s ON DELETE %s ON UPDATE %s", fk.column, foreign, fk.delete, fk.update))
}

func (fk ForeignKey) defaultForeign() string {
  splitIndex := strings.LastIndex(fk.column, "_")
  if splitIndex >= 0 {
    prefix, suffix := fk.column[:splitIndex], fk.column[splitIndex+1:]
    if suffix == "" {
      suffix = "id"
    }
    return fmt.Sprintf("%ss(%s)", prefix, suffix)
  }
  return fk.column + "(id)"
}

type Relational interface {
  ForeignKeys() []ForeignKey
}
