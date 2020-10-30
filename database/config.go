package database

type Config struct {
  Host     string `json:"host"`
  Port     uint   `json:"port"`
  Database string `json:"database"`
  User     string `json:"user"`
  Password string `json:"password"`
  Charset  string `json:"charset"`
  Timezone string `json:"timezone"`
  SSL      bool   `json:"ssl"`
  Debug    bool   `json:"debug"`
}
