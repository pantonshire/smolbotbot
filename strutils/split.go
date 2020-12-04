package strutils

func SplitOnPred(s string, bufLen uint, pred func([]rune) (bool, int)) (string, string) {
  if bufLen == 0 {
    return s, ""
  }
  var n uint
  rbuf := make([]rune, bufLen)
  ibuf := make([]int, bufLen)
  for i, r := range s {
    if n < bufLen {
      rbuf[n] = r
      ibuf[n] = i
      n++
    } else {
      var j uint
      for j = 1; j < bufLen; j++ {
        rbuf[j-1] = rbuf[j]
        ibuf[j-1] = ibuf[j]
      }
      rbuf[j-1] = r
      ibuf[j-1] = i
    }
    if ok, k := pred(rbuf[:n]); ok {
      splitAt := ibuf[k]
      return s[:splitAt], s[splitAt:]
    }
  }
  return s, ""
}

func SplitOnSeps(s string, bufLen uint, seps ...string) (string, string) {
  return SplitOnPred(s, bufLen, func(buf []rune) (bool, int) {
    for _, sep := range seps {
      match := true
      var j int
      for _, sr := range sep {
        if j == len(buf) {
          match = false
          break
        }
        if buf[j] != sr {
          match = false
          break
        }
        j++
      }
      if match {
        return true, 0
      }
    }
    return false, 0
  })
}
