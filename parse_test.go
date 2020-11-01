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
