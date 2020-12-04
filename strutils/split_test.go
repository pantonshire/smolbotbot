package strutils

import (
  "fmt"
  "testing"
  "unicode"
)

func TestSplitOnSeps(t *testing.T) {
  var tests = []struct {
    input              string
    inputBufLen        uint
    inputSeps          []string
    expectS1, expectS2 string
  }{
    {"Hello, world", 2, []string{", "}, "Hello", ", world"},
    {"Hello, world", 2, []string{"l", ", "}, "He", "llo, world"},
    {"Hello, world", 1, []string{", "}, "Hello, world", ""},
  }

  for _, tt := range tests {
    name := fmt.Sprintf("\"%s\"", tt.input)
    t.Run(name, func(t *testing.T) {
      s1, s2 := SplitOnSeps(tt.input, tt.inputBufLen, tt.inputSeps...)
      if s1 != tt.expectS1 {
        t.Errorf("got s1=%s, expected %s", s1, tt.expectS1)
      }
      if s2 != tt.expectS2 {
        t.Errorf("got s2=%s, expected %s", s2, tt.expectS2)
      }
    })
  }
}

func TestSplitOnPred(t *testing.T) {
  p0 := func(buf []rune) (bool, int) {
    if len(buf) == 2 && buf[0] == ',' && unicode.IsSpace(buf[1]) {
      return true, 1
    }
    return false, 0
  }

  var tests = []struct {
    input              string
    inputBufLen        uint
    inputPred          func([]rune) (bool, int)
    expectS1, expectS2 string
  }{
    {"Hello, world", 2, p0, "Hello,", " world"},
  }

  for _, tt := range tests {
    name := fmt.Sprintf("\"%s\"", tt.input)
    t.Run(name, func(t *testing.T) {
      s1, s2 := SplitOnPred(tt.input, tt.inputBufLen, tt.inputPred)
      if s1 != tt.expectS1 {
        t.Errorf("got s1=%s, expected %s", s1, tt.expectS1)
      }
      if s2 != tt.expectS2 {
        t.Errorf("got s2=%s, expected %s", s2, tt.expectS2)
      }
    })
  }
}
