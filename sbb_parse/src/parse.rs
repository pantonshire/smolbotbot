use std::ops::Range;

type ParseOut<'a, T> = Option<(&'a str, T)>;

fn parse_numbers(s: &str) -> ParseOut<Range<i32>> {
    let splitvec = s
        .splitn(2, ")")
        .collect::<Vec<&str>>();

    if splitvec.len() < 2 {
        return None;
    }

    let s = splitvec[0].trim();
    let rem = splitvec[1].trim_start();

    let mut ns = Vec::<i32>::new();

    let mut buf = String::new();
    let mut neg = false;
    let mut neg_enabled = true;
    let mut found_digit = false;

    fn parse_i32_buf(buf: &str, neg: bool) -> Option<i32> {
        Some(buf.parse::<i32>().ok()? * if neg { -1 } else { 1 })
    }

    for c in s.chars() {
        if c.is_ascii_digit() {
            found_digit = true;
            neg_enabled = false;
            buf.push(c);
        } else {
            if !buf.is_empty() {
                ns.push(parse_i32_buf(&buf, neg)?);
                buf.clear();
            }
            if c == '-' {
                if neg_enabled {
                    neg = true;
                }
            } else {
                neg = false;
                neg_enabled = true;
                if !found_digit {
                    return None;
                }
            }
        }
    }

    if !buf.is_empty() {
        ns.push(parse_i32_buf(&buf, neg)?);
    }

    Some((rem, numbers_range(&ns)?))
}

fn numbers_range(ns: &[i32]) -> Option<Range<i32>> {
    if ns.is_empty() {
        return None;
    }

    let first = ns[0];

    if ns.len() == 1 {
        return Some(first..first+1);
    }

    let (mut min_n, mut max_n) = (first, first);

    for &n in &ns[1..] {
        let n = if n > 0 && n < first.abs() {
            let mut major = first;
            let mut dps = 0;
            let mut x = n;
            while x > 0 {
                major /= 10;
                dps += 1;
                x /= 10;
            }
            for _ in 0..dps {
                major *= 10;
            }
            major + (n * first.signum())
        } else {
            n
        };

        if n < min_n {
            min_n = n;
        } else if n > max_n {
            max_n = n;
        }
    }

    Some(min_n..max_n+1)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_numbers() {
        use super::parse_numbers;
        assert_eq!(parse_numbers("123)"), Some(("", 123..124)));
        assert_eq!(parse_numbers("123) Teabot"), Some(("Teabot", 123..124)));
        assert_eq!(parse_numbers("  123  )  Teabot  "), Some(("Teabot  ", 123..124)));
        assert_eq!(parse_numbers("-1)"), Some(("", -1..0)));
        assert_eq!(parse_numbers("1, 2, 3)"), Some(("", 1..4)));
        assert_eq!(parse_numbers("123-124)"), Some(("", 123..125)));
        assert_eq!(parse_numbers("123 - 124)"), Some(("", 123..125)));
        assert_eq!(parse_numbers("123 & 4)"), Some(("", 123..125)));
        assert_eq!(parse_numbers("123 & 24)"), Some(("", 123..125)));
        assert_eq!(parse_numbers("124 & 3)"), Some(("", 123..125)));
        assert_eq!(parse_numbers("8, 7)"), Some(("", 7..9)));
        assert_eq!(parse_numbers("124-123)"), Some(("", 123..125)));
        assert_eq!(parse_numbers("1024 - 1048)"), Some(("", 1024..1049)));
        assert_eq!(parse_numbers("1024, 5 & 6)"), Some(("", 1024..1027)));
        assert_eq!(parse_numbers("1039, 8 & 40)"), Some(("", 1038..1041)));
    }
}
