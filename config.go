package smolbotbot

import (
  "encoding/json"
  "github.com/pantonshire/goldcrest/twitter1"
  "github.com/pantonshire/smolbotbot/database"
  "io/ioutil"
  "path/filepath"
)

type Options struct {
  Verbosity int
}

type Config struct {
  DB        database.Config `json:"db"`
  Goldcrest GoldcrestConfig `json:"goldcrest"`
}

type GoldcrestConfig struct {
  IsRemote bool                  `json:"is_remote"`
  Auth     twitter1.AuthPair     `json:"auth"`
  Local    GoldcrestLocalConfig  `json:"local"`
  Remote   GoldcrestRemoteConfig `json:"remote"`
}

type GoldcrestLocalConfig struct {
  Twitter        twitter1.TwitterConfig `json:"twitter"`
  TimeoutSeconds uint                   `json:"timeout_seconds"`
}

type GoldcrestRemoteConfig struct {
  Host           string `json:"host"`
  Port           uint   `json:"port"`
  TimeoutSeconds uint   `json:"timeout_seconds"`
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
