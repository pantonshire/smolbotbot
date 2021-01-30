#[derive(Clone, Debug)]
pub struct Sentence {
    pub tokens: Vec<Token>,
    pub entities: Vec<Entity>,
}

impl Sentence {
    pub fn join(sentences: Vec<Sentence>) -> Sentence {
        let (tc, ec) = sentences
            .iter()
            .fold((0, 0), |(tc, ec), s| (tc + s.tokens.len(), ec + s.entities.len()));
        let mut tokens = Vec::with_capacity(tc);
        let mut entities = Vec::with_capacity(ec);
        for sentence in sentences {
            tokens.extend(sentence.tokens.into_iter());
            entities.extend(sentence.entities.into_iter());
        }
        Sentence{
            tokens,
            entities,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub full: Text,
    pub stem: Text,
    pub tag: Tag,
    pub label: String,
}

impl Token {
    pub fn is_stopword(&self) -> bool {
        self.full.cleaned.is_none()
    }

    pub fn is_at_mention(&self) -> bool {
        self.full.raw.starts_with("@")
    }

    pub fn is_hashtag(&self) -> bool {
        self.full.raw.starts_with("#")
    }

    pub fn is_apostrophe(&self) -> bool {
        self.full.raw.starts_with("'")
    }
}

#[derive(Clone, Debug)]
pub struct Text {
    pub raw: String,
    pub cleaned: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Entity {
    pub text: String,
    pub label: String,
}

#[derive(Clone, Debug)]
pub enum Tag {
    LParen,
    RParen,
    Comma,
    Colon,
    Period,
    ClosingQuote,
    OpeningQuote,
    NumberSign,
    Currency,
    /// Conjunction, coordinating
    CC,
    /// Cardinal number
    CD,
    /// Determiner
    DT,
    /// Existential there
    EX,
    /// Foreign word
    FW,
    /// Conjunction, subordinating or preposition
    IN,
    /// Adjective
    JJ,
    /// Adjective, comparative
    JJR,
    /// Adjective, superlative
    JJS,
    /// List marker
    LS,
    /// Modal
    MD,
    /// Noun, singular or mass
    NN,
    /// noun, proper singular
    NNP,
    /// Noun, proper plural
    NNPS,
    /// Noun, plural
    NNS,
    /// Predeterminer
    PDT,
    /// Possessive ending
    POS,
    /// Pronoun, personal
    PRP,
    /// Pronoun, possessive
    PRPS,
    /// Adverb
    RB,
    /// Adverb, comparative
    RBR,
    /// Adverb, superlative
    RBS,
    /// Adverb, particle
    RP,
    /// Symbol
    SYM,
    /// Infinitival to
    TO,
    /// Interjection
    UH,
    /// Verb, base form
    VB,
    /// Verb, past tense
    VBD,
    /// Verb, gerund or present participle
    VBG,
    /// Verb, past participle
    VBN,
    /// Verb, non-3rd person singular present
    VBP,
    /// Verb, 3rd person singular present
    VBZ,
    /// Wh-determiner
    WDT,
    /// Wh-pronoun, personal
    WP,
    /// Wh-pronoun, possessive
    WPS,
    /// Wh-adverb
    WRB,
    Other(String),
}

impl Tag {
    pub fn is_noun(&self) -> bool {
        match self {
            Tag::NN   => true,
            Tag::NNP  => true,
            Tag::NNPS => true,
            Tag::NNS  => true,
            _         => false,
        }
    }

    pub fn is_verb(&self) -> bool {
        match self {
            Tag::VB  => true,
            Tag::VBD => true,
            Tag::VBG => true,
            Tag::VBN => true,
            Tag::VBP => true,
            Tag::VBZ => true,
            _        => false,
        }
    }

    pub fn is_adjective(&self) -> bool {
        match self {
            Tag::JJ  => true,
            Tag::JJR => true,
            Tag::JJS => true,
            _        => false,
        }
    }
}
