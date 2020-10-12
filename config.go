package smolbotbot

import (
  "encoding/json"
  "io/ioutil"
  "path/filepath"
)

type Options struct {
  Verbosity int
}

type Config struct {
  DB DatabaseConfig `json:"db"`
}

type DatabaseConfig struct {
  Host     string `json:"host"`
  Database string `json:"database"`
  User     string `json:"user"`
  Password string `json:"password"`
  Charset  string `json:"charset"`
  Loc      string `json:"loc"`
  Debug    bool   `json:"debug"`
}

func LoadConfig(configPath string) (Config, error) {
  configPath = filepath.Clean(configPath)
  configData, err := ioutil.ReadFile(configPath)
  if err != nil {
    return Config{}, err
  }
  var config Config
  if err := json.Unmarshal(configData, &config); err != nil {
    return Config{}, err
  }
  return config, nil
}
