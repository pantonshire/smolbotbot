use crate::data::*;
use crate::protocol;
use crate::error::{DeserializationError, DeserializationResult, Exists};

pub(crate) trait Deserialize<T> {
    fn deserialize(self) -> DeserializationResult<T>;
}

impl<S, T> Deserialize<Option<T>> for Option<S> where S: Deserialize<T> {
    fn deserialize(self) -> DeserializationResult<Option<T>> {
        self
            .map(S::deserialize)
            .map_or(Ok(None), |x| x.map(Some))
    }
}

impl<S, T> Deserialize<Vec<T>> for Vec<S> where S: Deserialize<T> {
    fn deserialize(self) -> DeserializationResult<Vec<T>> {
        self
            .into_iter()
            .map(S::deserialize)
            .collect()
    }
}

impl Deserialize<Sentence> for protocol::Sentence {
    fn deserialize(self) -> DeserializationResult<Sentence> {
        Ok(Sentence{
            tokens: self.tokens.deserialize()?,
            entities: self.entities.deserialize()?,
        })
    }
}

impl Deserialize<Token> for protocol::Token {
    fn deserialize(self) -> DeserializationResult<Token> {
        Ok(Token{
            full: self.full.exists()?.deserialize()?,
            stem: self.stem.exists()?.deserialize()?,
            tag: self.pos_tag.exists()?.deserialize()?,
            label: self.label,
        })
    }
}

impl Deserialize<Text> for protocol::Text {
    fn deserialize(self) -> DeserializationResult<Text> {
        Ok(Text{
            raw: self.raw,
            cleaned: if self.cleaned.is_empty() {
                None
            } else {
                Some(self.cleaned)
            },
        })
    }
}

impl Deserialize<Entity> for protocol::Entity {
    fn deserialize(self) -> DeserializationResult<Entity> {
        Ok(Entity{
            text: self.text,
            label: self.label,
        })
    }
}

impl Deserialize<Tag> for protocol::token::PosTag {
    fn deserialize(self) -> DeserializationResult<Tag> {
        use protocol::token::PosTag;
        match self {
            PosTag::Tag(i) => match protocol::Tag::from_i32(i) {
                Some(tag) => Ok(match tag {
                    protocol::Tag::LParen       => Tag::LParen,
                    protocol::Tag::RParen       => Tag::RParen,
                    protocol::Tag::Comma        => Tag::Comma,
                    protocol::Tag::Colon        => Tag::Colon,
                    protocol::Tag::Period       => Tag::Period,
                    protocol::Tag::ClosingQuote => Tag::ClosingQuote,
                    protocol::Tag::OpeningQuote => Tag::OpeningQuote,
                    protocol::Tag::NumberSign   => Tag::NumberSign,
                    protocol::Tag::Currency     => Tag::Currency,
                    protocol::Tag::Cc           => Tag::CC,
                    protocol::Tag::Cd           => Tag::CD,
                    protocol::Tag::Dt           => Tag::DT,
                    protocol::Tag::Ex           => Tag::EX,
                    protocol::Tag::Fw           => Tag::FW,
                    protocol::Tag::In           => Tag::IN,
                    protocol::Tag::Jj           => Tag::JJ,
                    protocol::Tag::Jjr          => Tag::JJR,
                    protocol::Tag::Jjs          => Tag::JJS,
                    protocol::Tag::Ls           => Tag::LS,
                    protocol::Tag::Md           => Tag::MD,
                    protocol::Tag::Nn           => Tag::NN,
                    protocol::Tag::Nnp          => Tag::NNP,
                    protocol::Tag::Nnps         => Tag::NNPS,
                    protocol::Tag::Nns          => Tag::NNS,
                    protocol::Tag::Pdt          => Tag::PDT,
                    protocol::Tag::Pos          => Tag::POS,
                    protocol::Tag::Prp          => Tag::PRP,
                    protocol::Tag::Prps         => Tag::PRPS,
                    protocol::Tag::Rb           => Tag::RB,
                    protocol::Tag::Rbr          => Tag::RBR,
                    protocol::Tag::Rbs          => Tag::RBS,
                    protocol::Tag::Rp           => Tag::RP,
                    protocol::Tag::Sym          => Tag::SYM,
                    protocol::Tag::To           => Tag::TO,
                    protocol::Tag::Uh           => Tag::UH,
                    protocol::Tag::Vb           => Tag::VB,
                    protocol::Tag::Vbd          => Tag::VBD,
                    protocol::Tag::Vbg          => Tag::VBG,
                    protocol::Tag::Vbn          => Tag::VBN,
                    protocol::Tag::Vbp          => Tag::VBP,
                    protocol::Tag::Vbz          => Tag::VBZ,
                    protocol::Tag::Wdt          => Tag::WDT,
                    protocol::Tag::Wp           => Tag::WP,
                    protocol::Tag::Wps          => Tag::WPS,
                    protocol::Tag::Wrb          => Tag::WRB,
                }),
                None => Err(DeserializationError::FieldOutOfRange),
            }
            PosTag::Other(s) => Ok(Tag::Other(s)),
        }
    }
}
