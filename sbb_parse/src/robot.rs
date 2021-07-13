use std::borrow::Cow;

use lazy_static::lazy_static;
use regex::Regex;
use unidecode::unidecode;

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Robot<'a> {
    pub number: i32,
    pub name: RobotName<'a>,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct RobotName<'a> {
    pub prefix: Cow<'a, str>,
    pub suffix: Cow<'a, str>,
    pub plural: Option<Cow<'a, str>>,
}

impl RobotName<'_> {
    pub fn identifier(&self) -> String {
        lazy_static! {
            static ref NON_WORD_RE: Regex = Regex::new(r"\W+").unwrap();
        }

        let ascii = unidecode(&self.prefix);

        let mut ident = NON_WORD_RE
            .replace_all(&ascii, "")
            .to_lowercase();

        ident.retain(|c| !c.is_whitespace());

        ident
    }
}
