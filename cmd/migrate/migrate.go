package main

import (
  "github.com/jessevdk/go-flags"
  "github.com/pantonshire/smolbotbot"
  "github.com/pantonshire/smolbotbot/database"
  "os"
)

func main() {
  var opts struct {
    ConfigPath string `short:"c" long:"config" description:"Path to the configuration file"`
  }
  if _, err := flags.Parse(&opts); err != nil {
    if flagErr, ok := err.(*flags.Error); ok {
      if flagErr.Type == flags.ErrHelp {
        os.Exit(0)
      } else {
        os.Exit(1)
      }
    } else {
      panic(err)
    }
  }
  config, err := smolbotbot.LoadConfig(opts.ConfigPath)
  if err != nil {
    panic(err)
  }
  db, err := database.Connect(config.DB)
  if err != nil {
    panic(err)
  }
  defer func() {
    if err := db.Close(); err != nil {
      panic(err)
    }
  }()
  if err := smolbotbot.Migrate(db); err != nil {
    panic(err)
  }
}
