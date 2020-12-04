package smolbotbot

import (
  "sort"
  "strconv"
  "strings"
  "unicode"
)

type botName struct {
  prefix, suffix string
}

func (name botName) join() string {
  return name.prefix + name.suffix
}

func (name botName) singular() botName {
  return botName{prefix: name.prefix, suffix: makeBotSuffixSingular(name.suffix)}
}

//func parseRobotTweet(tweet twitter1.Tweet) []model.Robot {
//  const groupedRobotLimit = 5
//
//  parts := strings.SplitN(tweet.Text, ")", 2)
//  if len(parts) != 2 {
//    return nil
//  }
//
//  minNumber, maxNumber, ok := extractNumbers(parts[0])
//  if !ok {
//    return nil
//  }
//  if maxNumber > minNumber+groupedRobotLimit {
//    maxNumber = minNumber + groupedRobotLimit
//  }
//
//  body := strings.TrimSpace(parts[1])
//
//  intro, description := strutils.SplitOnPred(body, 3, func(buf []rune) (bool, int) {
//    if len(buf) >= 2 {
//      r0, r1 := buf[0], buf[1]
//      if (r0 == '.' || r0 == ':' || r0 == ';' || r0 == '!' || r0 == '?') && unicode.IsSpace(r1) {
//        return true, 0
//      }
//
//      if len(buf) >= 3 {
//        r2 := buf[2]
//        if unicode.IsSpace(r0) && (r1 == '-' || r1 == '\u2013' || r1 == '\u2014') && unicode.IsSpace(r2) {
//          return true, 0
//        }
//      }
//    }
//    return false, 0
//  })
//
//  intro, description = strings.TrimSpace(intro), strings.TrimSpace(description)
//
//  names := extractBotNames(intro, maxNumber-minNumber+1)
//}
//
//func generateTags(robot model.Robot) []model.Tag {
//
//}

// Attempts to extract a range of robot numbers from a tweet robot number prefix.
// Returns the minimum number, the maximum number and a boolean to indicate whether or not any numbers
// were found.
//
// For example:
//  extractNumbers("123")         -> 123, 123, true
//  extractNumbers("123,124,125") -> 123, 125, true
//  extractNumbers("123 - 125")   -> 123, 125, true
//  extractNumbers("Teabot")      -> 0, 0, false
func extractNumbers(text string) (min int, max int, ok bool) {
  fields := strings.Fields(strings.TrimSpace(text))
  if len(fields) == 0 {
    return 0, 0, false
  }
  var listedNs []int
  for i, field := range fields {
    var start int
    var num, neg bool
    var foundNumber bool
    for j, r := range field {
      if '0' <= r && r <= '9' {
        if !num {
          foundNumber = true
          num = true
          if !neg {
            start = j
          }
        }
      } else if r == '-' && !num && !neg {
        neg = true
        start = j
      } else {
        if num {
          num = false
          n, err := strconv.Atoi(field[start:j])
          if err != nil {
            panic(err)
          }
          listedNs = append(listedNs, n)
        }
        neg = false
      }
    }
    if num {
      n, err := strconv.Atoi(field[start:])
      if err != nil {
        panic(err)
      }
      listedNs = append(listedNs, n)
    }
    if i == 0 && !foundNumber {
      return 0, 0, false
    }
  }
  if len(listedNs) == 0 {
    return 0, 0, false
  } else if len(listedNs) == 1 {
    return listedNs[0], listedNs[0], true
  }
  for i := 1; i < len(listedNs); i++ {
    if listedNs[i] < listedNs[0] && listedNs[i] > 0 {
      base := listedNs[0]
      var dps int
      for x := listedNs[i]; x > 0; x /= 10 {
        dps++
        base /= 10
      }
      for i := 0; i < dps; i++ {
        base *= 10
      }
      listedNs[i] = base + listedNs[i]
    }
  }
  min, max = listedNs[0], listedNs[0]
  for i := 1; i < len(listedNs); i++ {
    if listedNs[i] < min {
      min = listedNs[i]
    } else if listedNs[i] > max {
      max = listedNs[i]
    }
  }
  return min, max, true
}

// Attempts to extract the first n robot names from the given body text.
func extractBotNames(text string, n int) []botName {
  if n < 1 {
    return nil
  }

  var names []botName
  var nonNames []string
  var nameIndices, nonNameIndices []int

  for i, tok := range strings.Fields(text) {
    if name, ok := parseBotName(tok); ok {
      names = append(names, name)
      if n--; n == 0 {
        return names
      }
      nameIndices = append(nameIndices, i)
    } else {
      nonNames = append(nonNames, tok)
      nonNameIndices = append(nonNameIndices, i)
    }
  }

  nameSort := func(i, j int) bool {
    return nameIndices[i] < nameIndices[j]
  }

  for i, tok := range nonNames {
    tok = strings.TrimFunc(tok, func(r rune) bool {
      return !(unicode.IsLetter(r) || unicode.IsNumber(r))
    })
    if strings.ToLower(tok) != "and" {
      names = append(names, botName{prefix: tok, suffix: "bot"})
      nameIndices = append(nameIndices, nonNameIndices[i])
      if n--; n == 0 {
        sort.Slice(names, nameSort)
        return names
      }
    }
  }

  sort.Slice(names, nameSort)
  return names
}

func dropSubstringAt(s, sep string) string {
  split := strings.SplitN(s, sep, 2)
  if len(split) == 0 {
    return ""
  }
  return split[0]
}

var botPluralSuffix = [4]rune{'b', 'o', 't', 's'}

func parseBotName(token string) (name botName, ok bool) {
  const (
    minMatchLength = 3
    maxMatchLength = 4
  )
  var builder strings.Builder
  var l, start, cutAt int
  for i, r := range token {
    if unicode.IsLetter(r) {
      if l < maxMatchLength && unicode.ToLower(r) == botPluralSuffix[l] {
        if l == 0 {
          start = i
        }
        builder.WriteRune(r)
        cutAt = 0
        l++
      } else if l > 0 {
        builder.Reset()
        l = 0
      }
    } else if l > 0 {
      builder.WriteRune(r)
      if cutAt == 0 {
        cutAt = i
      }
    }
  }
  if l < minMatchLength {
    return botName{}, false
  }
  if cutAt > 0 {
    return botName{prefix: token[:start], suffix: builder.String()[:cutAt-start]}, true
  }
  return botName{prefix: token[:start], suffix: builder.String()}, true
}

func makeBotSuffixSingular(suffix string) string {
  var cutAt int
  for i, r := range suffix {
    lr := unicode.ToLower(r)
    if lr == botPluralSuffix[len(botPluralSuffix)-1] {
      if cutAt == 0 {
        return suffix[:i]
      } else {
        return suffix[:cutAt]
      }
    }
    var isSuffixCharacter bool
    for i := 0; i < len(botPluralSuffix)-1; i++ {
      if lr == botPluralSuffix[i] {
        isSuffixCharacter = true
        break
      }
    }
    if isSuffixCharacter {
      cutAt = 0
    } else if cutAt == 0 {
      cutAt = i
    }
  }
  return suffix
}
