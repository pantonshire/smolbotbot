package smolbotbot

import (
  "github.com/pantonshire/goldcrest/twitter1"
  "time"
)

func newTwitterClient(config GoldcrestConfig) twitter1.Client {
  if config.IsRemote {
    return twitter1.Remote(nil, config.Auth, time.Second*time.Duration(config.Local.TimeoutSeconds))
  } else {
    return twitter1.Local(config.Auth, config.Local.Twitter, time.Second*time.Duration(config.Local.TimeoutSeconds))
  }
}
