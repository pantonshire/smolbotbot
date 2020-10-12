package model

func Models() []interface{} {
  return []interface{}{
    &Robot{},
    &Tag{},
    &Tweet{},
    &DirectMessage{},
    &PastDaily{},
    &ScheduledDaily{},
  }
}
