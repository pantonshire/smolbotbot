use crate::protocol;

#[derive(Clone, Copy, Debug)]
pub enum Language {
    English,
}

impl Language {
    pub(crate) fn serialize(self) -> protocol::Language {
        match self {
            Language::English => protocol::Language::English,
        }
    }

    pub(crate) fn serialize_i32(self) -> i32 {
        self.serialize() as i32
    }
}
