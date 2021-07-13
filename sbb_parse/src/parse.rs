use std::ops::Range;
use std::borrow::Cow;
use std::fmt;

use lazy_static::lazy_static;
use regex::Regex;

use crate::robot::{Robot, RobotName};

struct ParseOut<'a, T> {
    remainder: &'a str,
    output: T,
}

type OptParseOut<'a, T> = Option<ParseOut<'a, T>>;

impl<'a, T> fmt::Debug for ParseOut<'a, T> where T: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParseOut")
            .field("output", &self.output)
            .field("remainder", &self.remainder)
            .finish()
    }
}

impl<'a, T> PartialEq for ParseOut<'a, T> where T: Eq {
    fn eq(&self, other: &Self) -> bool {
        self.output == other.output && self.remainder == other.remainder
    }
}

impl<'a, T> Eq for ParseOut<'a, T> where T: Eq {}

impl<'a, T> ParseOut<'a, T> {
    const fn new(remainder: &'a str, output: T) -> Self {
        ParseOut {
            remainder,
            output,
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ParsedGroup<'a> {
    pub robots: Vec<Robot<'a>>,
    pub body: &'a str,
    pub cw: Option<&'a str>,
}

pub fn parse_group(text: &str) -> Option<ParsedGroup> {
    const MAX_GROUP_SIZE: usize = 5;

    lazy_static! {
        // Meaning                             | Regex fragment
        // =====================================================
        // Word character                      | \w
        // Zero or more of any character       |   .*
        // End of string                       |     $
        static ref BODY_RE: Regex = Regex::new(r"\w.*$").unwrap();
    }

    let s = text.trim();

    let ParseOut { remainder: s, output: cw }
        = parse_cw(s);

    let ParseOut { remainder: s, output: n_range }
        = parse_numbers(s)?;

    let ParseOut { remainder: s, output: (names, partial_names) }
        = parse_names(s, n_range.len().min(MAX_GROUP_SIZE))?;

    let body = BODY_RE
        .find(s)
        .map(|m| m.as_str())
        .unwrap_or("");

    let robots = names
        .into_iter()
        .enumerate()
        .map(|(i, name)| Robot{
            number: n_range.start + (i as i32),
            name: if partial_names {
                RobotName{ plural: None, ..name }
            } else {
                name
            },
        })
        .collect::<Vec<Robot>>();

    Some(ParsedGroup {
        robots,
        body,
        cw,
    })
}

fn parse_cw(s: &str) -> ParseOut<Option<&str>> {
    lazy_static! {
        static ref CW_RE: Regex = Regex::new(r"[\[\(][Cc][Nn]:\s*(\S[^\]\)]+)[\]\)]").unwrap();
    }

    let captures = match CW_RE.captures(s) {
        Some(cs) => cs,
        None     => return ParseOut::new(s, None),
    };

    let match_end = captures.get(0).unwrap().end();
    let warning_type = captures.get(1).unwrap().as_str().trim();

    ParseOut::new(s[match_end..].trim_start(), Some(warning_type))
}

fn parse_numbers(s: &str) -> OptParseOut<Range<i32>> {
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

    Some(ParseOut::new(rem, numbers_range(&ns)?))
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

fn parse_names(s: &str, target_n: usize) -> OptParseOut<(Vec<RobotName>, bool)> {
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
            prefix: Cow::Borrowed(caps.get(1).unwrap().as_str()),
            suffix: Cow::Borrowed(caps.get(2).unwrap().as_str()),
            plural: caps.get(3).map(|m| Cow::Borrowed(m.as_str())),
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

        let first_suffix = names[0].suffix.clone();
        let first_plural = names[0].plural.clone();

        let partial_names = s
            .split_whitespace()
            .filter(|&w| w.to_lowercase() != "and")
            .map(|w| PARTIAL_BOT_RE.captures(w))
            .filter(|m| m.is_some())
            .map(|m| m.unwrap())
            .filter(|m| m[1].chars().any(|c| !c.is_ascii_digit()))
            .map(|m| RobotName{
                prefix: Cow::Borrowed(m.get(1).unwrap().as_str()),
                suffix: first_suffix.clone(),
                plural: first_plural.clone(),
            });

        for (i, name) in partial_names.take(diff).enumerate() {
            names.insert(i, name);
        }
    }

    Some(ParseOut::new(&s[matches_end..], (names, use_partial_names)))
}

#[cfg(test)]
mod tests {
    use super::{ParseOut, ParsedGroup};
    use crate::robot::RobotName;

    #[test]
    fn test_parse_numbers() {
        use super::parse_numbers;

        assert_eq!(parse_numbers("123)"), Some(ParseOut::new("", 123..124)));
        assert_eq!(parse_numbers("123) Teabot"), Some(ParseOut::new("Teabot", 123..124)));
        assert_eq!(parse_numbers("  123  )  Teabot  "), Some(ParseOut::new("Teabot  ", 123..124)));
        assert_eq!(parse_numbers("-1)"), Some(ParseOut::new("", -1..0)));
        assert_eq!(parse_numbers("1, 2, 3)"), Some(ParseOut::new("", 1..4)));
        assert_eq!(parse_numbers("123-124)"), Some(ParseOut::new("", 123..125)));
        assert_eq!(parse_numbers("123 - 124)"), Some(ParseOut::new("", 123..125)));
        assert_eq!(parse_numbers("123 & 4)"), Some(ParseOut::new("", 123..125)));
        assert_eq!(parse_numbers("123 & 24)"), Some(ParseOut::new("", 123..125)));
        assert_eq!(parse_numbers("124 & 3)"), Some(ParseOut::new("", 123..125)));
        assert_eq!(parse_numbers("8, 7)"), Some(ParseOut::new("", 7..9)));
        assert_eq!(parse_numbers("124-123)"), Some(ParseOut::new("", 123..125)));
        assert_eq!(parse_numbers("1024 - 1048)"), Some(ParseOut::new("", 1024..1049)));
        assert_eq!(parse_numbers("1024, 5 & 6)"), Some(ParseOut::new("", 1024..1027)));
        assert_eq!(parse_numbers("1039, 8 & 40)"), Some(ParseOut::new("", 1038..1041)));
        assert_eq!(parse_numbers("123"), None);
        assert_eq!(parse_numbers("Foo baa"), None);
        assert_eq!(parse_numbers("2147483646)"), Some(ParseOut::new("", 2147483646..2147483647)));
        assert_eq!(parse_numbers("2147483647)"), None);
        assert_eq!(parse_numbers("2147483648)"), None);
        assert_eq!(parse_numbers("Hello)"), None);
        assert_eq!(parse_numbers("@foo 123)"), None);
        assert_eq!(parse_numbers("@foo123)"), None);
    }

    #[test]
    fn test_parse_names() {
        use super::parse_names;

        assert_eq!(
            parse_names("Teabot. Brings you tea", 1),
            Some(ParseOut::new(". Brings you tea", (vec![RobotName{ prefix: "Tea".into(), suffix: "bot".into(), plural: None }], false)))
        );
        
        assert_eq!(
            parse_names("Mischiefbots. Oh no!!", 1),
            Some(ParseOut::new(". Oh no!!", (vec![RobotName{ prefix: "Mischief".into(), suffix: "bot".into(), plural: Some("s".into()) }], false)))
        );
        
        assert_eq!(
            parse_names("R.O.B.O.T.S.", 1),
            Some(ParseOut::new(".", (vec![RobotName{ prefix: "R.O.".into(), suffix: "B.O.T".into(), plural: Some(".S".into()) }], false)))
        );
        
        assert_eq!(
            parse_names("Saltbot and pepperbot.", 1),
            Some(ParseOut::new(" and pepperbot.", (vec![RobotName{ prefix: "Salt".into(), suffix: "bot".into(), plural: None }], false)))
        );
        
        assert_eq!(
            parse_names("Saltbot and pepperbot.", 2),
            Some(ParseOut::new(".", (vec![RobotName{ prefix: "Salt".into(), suffix: "bot".into(), plural: None }, RobotName{ prefix: "pepper".into(), suffix: "bot".into(), plural: None }], false)))
        );
        
        assert_eq!(
            parse_names("Saltbot and pepperbot.", 3),
            Some(ParseOut::new(".", (vec![RobotName{ prefix: "Salt".into(), suffix: "bot".into(), plural: None }, RobotName{ prefix: "pepper".into(), suffix: "bot".into(), plural: None }], false)))
        );
        
        assert_eq!(
            parse_names("Salt- and pepperbots.", 2),
            Some(ParseOut::new(".", (vec![RobotName{ prefix: "Salt".into(), suffix: "bot".into(), plural: Some("s".into()) }, RobotName{ prefix: "pepper".into(), suffix: "bot".into(), plural: Some("s".into()) }], true)))
        );
    }

    #[test]
    fn test_parse_group() {
        use super::{parse_group, Robot};

        assert_eq!(
            parse_group("1207) Transrightsbot. Is just here to let all its trans pals know that they are valid and they are loved! \u{1f3f3}\u{fe0f}\u{200d}\u{26a7}\u{fe0f}\u{2764}\u{fe0f}\u{1f916}"),
            Some(ParsedGroup { robots: vec![Robot { number: 1207, name: RobotName { prefix: "Transrights".into(), suffix: "bot".into(), plural: None } }], body: "Is just here to let all its trans pals know that they are valid and they are loved! \u{1f3f3}\u{fe0f}\u{200d}\u{26a7}\u{fe0f}\u{2764}\u{fe0f}\u{1f916}", cw: None })
        );
        
        assert_eq!(
            parse_group("558/9) Salt- and Pepperbots. Bring you salt and pepper."),
            Some(ParsedGroup { robots: vec![Robot { number: 558, name: RobotName { prefix: "Salt".into(), suffix: "bot".into(), plural: None } }, Robot { number: 559, name: RobotName { prefix: "Pepper".into(), suffix: "bot".into(), plural: None } }], body: "Bring you salt and pepper.", cw: None })
        );
        
        assert_eq!(
            parse_group("690 - 692) Marybot, Josephbot and Donkeybot. For complicated tax reasons, Marybot and Josephbot are forced to temporarily relocate to Bethlehem, just as Marybot recieves a mysterious package from Gabrielbot on behalf of Godbot Labs."),
            Some(ParsedGroup { robots: vec![Robot { number: 690, name: RobotName { prefix: "Mary".into(), suffix: "bot".into(), plural: None } }, Robot { number: 691, name: RobotName { prefix: "Joseph".into(), suffix: "bot".into(), plural: None } }, Robot { number: 692, name: RobotName { prefix: "Donkey".into(), suffix: "bot".into(), plural: None } }], body: "For complicated tax reasons, Marybot and Josephbot are forced to temporarily relocate to Bethlehem, just as Marybot recieves a mysterious package from Gabrielbot on behalf of Godbot Labs.", cw: None })
        );
        
        assert_eq!(
            parse_group("[CN: sexual assault] 651) Believeherbot. Reminds you to believe the testimony of women survivors of sexual assault; reminds you to look at the gendered power structures in place before you dismiss them as unreliable; reminds you that this is the fucking turning point."),
            Some(ParsedGroup { robots: vec![Robot { number: 651, name: RobotName { prefix: "Believeher".into(), suffix: "bot".into(), plural: None } }], body: "Reminds you to believe the testimony of women survivors of sexual assault; reminds you to look at the gendered power structures in place before you dismiss them as unreliable; reminds you that this is the fucking turning point.", cw: Some("sexual assault") })
        );
    }
}
