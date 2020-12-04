package smolbotbot

import (
  "fmt"
  "testing"
)

func TestExtractNumbers(t *testing.T) {
  var tests = []struct {
    input     string
    expectMin int
    expectMax int
    expectOk  bool
  }{
    {input: "123", expectMin: 123, expectMax: 123, expectOk: true},
    {input: "123 124", expectMin: 123, expectMax: 124, expectOk: true},
    {input: "-123", expectMin: -123, expectMax: -123, expectOk: true},
    {input: "-123 -122", expectMin: -123, expectMax: -122, expectOk: true},
    {input: "123-124", expectMin: 123, expectMax: 124, expectOk: true},
    {input: "123-125", expectMin: 123, expectMax: 125, expectOk: true},
    {input: "123 - 125", expectMin: 123, expectMax: 125, expectOk: true},
    {input: "123/124", expectMin: 123, expectMax: 124, expectOk: true},
    {input: "123/4", expectMin: 123, expectMax: 124, expectOk: true},
    {input: "hello 123", expectMin: 0, expectMax: 0, expectOk: false},
    {input: "", expectMin: 0, expectMax: 0, expectOk: false},
  }

  for _, tt := range tests {
    name := fmt.Sprintf("\"%s\"", tt.input)
    t.Run(name, func(t *testing.T) {
      resultMin, resultMax, ok := extractNumbers(tt.input)
      if ok != tt.expectOk {
        t.Errorf("got ok=%t, expected %t", ok, tt.expectOk)
      }
      if resultMin != tt.expectMin {
        t.Errorf("got min=%d, expected %d", resultMin, tt.expectMin)
      }
      if resultMax != tt.expectMax {
        t.Errorf("got max=%d, expected %d", resultMin, tt.expectMax)
      }
    })
  }
}

func TestParseBotName(t *testing.T) {
  var tests = []struct {
    input      string
    expectName botName
    expectOk   bool
  }{
    {"", botName{}, false},
    {"Hello", botName{}, false},
    {"Teabot", botName{prefix: "Tea", suffix: "bot"}, true},
    {"Teabots", botName{prefix: "Tea", suffix: "bots"}, true},
    {"R.O.B.O.T.S.", botName{prefix: "R.O.", suffix: "B.O.T.S"}, true},
  }

  for _, tt := range tests {
    name := fmt.Sprintf("\"%s\"", tt.input)
    t.Run(name, func(t *testing.T) {
      n, ok := parseBotName(tt.input)
      if ok != tt.expectOk {
        t.Errorf("got ok=%t, expected %t", ok, tt.expectOk)
      }
      if tt.expectOk {
        if n != tt.expectName {
          t.Errorf("got name=%s, expected %s", n, tt.expectName)
        }
      }
    })
  }
}

func TestExtractBotNames(t *testing.T) {
  var tests = []struct {
    input       string
    inputN      int
    expectNames []botName
  }{
    {"", 1, []botName{}},
    {"Teabot", 1, []botName{{"Tea", "bot"}}},
    {"Teabot", 2, []botName{{"Tea", "bot"}}},
    {"Marybot, Josephbot and Donkeybot", 3, []botName{
      {"Mary", "bot"},
      {"Joseph", "bot"},
      {"Donkey", "bot"}}},
    {"Salt- and Pepperbots", 2, []botName{
      {"Salt", "bot"},
      {"Pepper", "bots"}}},
    {"Salt- and Pepperbots", 3, []botName{
      {"Salt", "bot"},
      {"Pepper", "bots"}}},
  }

  for _, tt := range tests {
    name := fmt.Sprintf("\"%s\"", tt.input)
    t.Run(name, func(t *testing.T) {
      ns := extractBotNames(tt.input, tt.inputN)
      if len(ns) != len(tt.expectNames) {
        t.Errorf("got names=%s, expected %s", ns, tt.expectNames)
      }
      for i := 0; i < len(ns); i++ {
        if ns[i] != tt.expectNames[i] {
          t.Errorf("got names[%d]=%s, expected %s", i, ns[i], tt.expectNames[i])
        }
      }
    })
  }
}
