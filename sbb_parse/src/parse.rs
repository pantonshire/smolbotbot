use std::ops::Range;
use lazy_static::lazy_static;
use regex::Regex;
use chrono::NaiveDateTime;

use sbb_data::new::*;
use std::borrow::Borrow;

type ParseOut<'a, T> = Option<(&'a str, T)>;

#[derive(PartialEq, Eq, Debug)]
struct RobotName<'a> {
    prefix: &'a str,
    suffix: &'a str,
    plural: Option<&'a str>,
}

pub fn parse_group(text: &str, tweet_id: i64, tweet_time: NaiveDateTime, image_url: Option<&str>, alt: Option<&str>) -> Option<(Vec<NewRobot>, &str)> {
    const MAX_GROUP_SIZE: usize = 5;

    let (s, n_range) = parse_numbers(text)?;
    let (s, (names, partial_names)) = parse_names(s, n_range.len().min(MAX_GROUP_SIZE))?;

    None
}

fn parse_numbers(s: &str) -> ParseOut<Range<i32>> {
    let split_vec = s
        .splitn(2, ")")
        .collect::<Vec<&str>>();

    if split_vec.len() < 2 {
        return None;
    }

    let s = split_vec[0].trim();
    let rem = split_vec[1].trim_start();

    let mut ns = Vec::<i32>::new();

    let mut buf = String::new();
    let mut neg = false;
    let mut neg_enabled = true;
    let mut found_digit = false;

    fn parse_i32_buf(buf: &str, neg: bool) -> Option<i32> {
        let n = buf.parse::<i32>().ok()? * if neg { -1 } else { 1 };
        if n == i32::MAX {
            return None;
        }
        Some(n)
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

fn parse_names(s: &str, target_n: usize) -> ParseOut<(Vec<RobotName>, bool)> {
    lazy_static! {
        // Meaning                            | Regex fragment
        // =======================================================================================
        // First matching group               | (   )
        // One or more non-whitespace         |  \S+
        // Second matching group              |      (                            )
        // Uppercase or lowercase B           |       [Bb]
        // 0 or more non-word, non-whitespace |           [^\w\s]*
        // Uppercase or lowercase O           |                   [Oo]
        // 0 or more non-word, non-whitespace |                       [^\w\s]*
        // Uppercase or lowercase T           |                               [Tt]
        // Third matching group, optional     |                                    (            )?
        // 0 or more non-word, non-whitespace |                                     [^\w\s]*
        // Uppercase or lowercase S           |                                             [Ss]
        static ref BOT_RE: Regex = Regex::new(r"(\S+)([Bb][^\w\s]*[Oo][^\w\s]*[Tt])([^\w\s]*[Ss])?").unwrap();

        // Meaning                                    | Regex fragment
        // =======================================================================================
        // Beginning of the string                    | ^
        // First matching group                       |  (      )
        // 2 or more word characters                  |   \w{2,}
        // Second matching group, optional            |          ( )?
        // Hyphen character literal                   |           -
        static ref PARTIAL_BOT_RE: Regex = Regex::new(r"^(\w{2,})(-)?").unwrap();
    }

    let mut names = Vec::<RobotName>::new();
    let mut first_match = true;
    let mut matches_start = 0;
    let mut matches_end = 0;

    for caps in BOT_RE.captures_iter(s) {
        if names.len() == target_n {
            break;
        }

        names.push(RobotName{
            prefix: caps.get(1).unwrap().as_str(),
            suffix: caps.get(2).unwrap().as_str(),
            plural: caps.get(3).map(|m| m.as_str()),
        });

        let full_match = caps.get(0).unwrap();
        if first_match {
            first_match = false;
            matches_start = full_match.start();
        }
        matches_end = full_match.end();
    }

    if names.is_empty() {
        return None;
    }

    let use_partial_names = names.len() < target_n && matches_start > 0;

    if use_partial_names {
        let diff = target_n - names.len();
        let s = &s[..matches_start];

        let first_suffix = names[0].suffix;
        let first_plural = names[0].plural;

        let partial_names = s
            .split_whitespace()
            .filter(|&w| w.to_lowercase() != "and")
            .map(|w| PARTIAL_BOT_RE.captures(w))
            .filter(|m| m.is_some())
            .map(|m| m.unwrap())
            .filter(|m| m[1].chars().any(|c| !c.is_ascii_digit()))
            .map(|m| RobotName{
                prefix: m.get(1).unwrap().as_str(),
                suffix: first_suffix,
                plural: first_plural,
            });

        for (i, name) in partial_names.take(diff).enumerate() {
            names.insert(i, name);
        }
    }

    Some((&s[matches_end..], (names, use_partial_names)))
}

#[cfg(test)]
mod tests {
    use crate::parse::RobotName;

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
        assert_eq!(parse_numbers("123"), None);
        assert_eq!(parse_numbers("Foo baa"), None);
        assert_eq!(parse_numbers("2147483646)"), Some(("", 2147483646..2147483647)));
        assert_eq!(parse_numbers("2147483647)"), None);
        assert_eq!(parse_numbers("2147483648)"), None);
    }

    #[test]
    fn test_parse_names() {
        use super::parse_names;
        assert_eq!(parse_names("Teabot. Brings you tea", 1), Some((". Brings you tea", (vec![RobotName{ prefix: "Tea", suffix: "bot", plural: None }], false))));
        assert_eq!(parse_names("Mischiefbots. Oh no!!", 1), Some((". Oh no!!", (vec![RobotName{ prefix: "Mischief", suffix: "bot", plural: Some("s") }], false))));
        assert_eq!(parse_names("R.O.B.O.T.S.", 1), Some((".", (vec![RobotName{ prefix: "R.O.", suffix: "B.O.T", plural: Some(".S") }], false))));
        assert_eq!(parse_names("Saltbot and pepperbot.", 1), Some((" and pepperbot.", (vec![RobotName{ prefix: "Salt", suffix: "bot", plural: None }], false))));
        assert_eq!(parse_names("Saltbot and pepperbot.", 2), Some((".", (vec![RobotName{ prefix: "Salt", suffix: "bot", plural: None }, RobotName{ prefix: "pepper", suffix: "bot", plural: None }], false))));
        assert_eq!(parse_names("Saltbot and pepperbot.", 3), Some((".", (vec![RobotName{ prefix: "Salt", suffix: "bot", plural: None }, RobotName{ prefix: "pepper", suffix: "bot", plural: None }], false))));
        assert_eq!(parse_names("Salt- and pepperbots.", 2), Some((".", (vec![RobotName{ prefix: "Salt", suffix: "bot", plural: Some("s") }, RobotName{ prefix: "pepper", suffix: "bot", plural: Some("s") }], true))));
    }
}
