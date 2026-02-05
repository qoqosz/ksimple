pub(crate) const VERB_TOKENS: &str = " +-!#,@=~&|*";
pub(crate) const ADVERB_TOKENS: &str = " /\\";

/// A token in the k/simple programming language.
#[derive(Clone, Debug)]
pub(crate) enum Token {
    /// A number.
    Number(i64),
    /// A global variable, a-z.
    Global(u8),
    /// A symbol, verb or adverb.
    Symbol(u8),
    /// A colon.
    Colon,
}

impl Token {
    /// Returns true if the token can start a negative number.
    pub(crate) fn can_start_negative(&self) -> bool {
        match self {
            Token::Colon => true,
            Token::Symbol(symbol) => {
                VERB_TOKENS.as_bytes().contains(symbol) || ADVERB_TOKENS.as_bytes().contains(symbol)
            }
            _ => false,
        }
    }
}

pub(crate) fn verb_index(token: u8) -> usize {
    VERB_TOKENS
        .as_bytes()
        .iter()
        .position(|&byte| byte == token)
        .unwrap_or(0)
}

pub(crate) fn adverb_index(token: u8) -> usize {
    ADVERB_TOKENS
        .as_bytes()
        .iter()
        .position(|&byte| byte == token)
        .unwrap_or(0)
}

pub(crate) fn tokenize_line(line: &str) -> Result<Vec<Token>, ()> {
    let bytes = line.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        let byte = bytes[index];

        // Skip whitespace.
        if byte.is_ascii_whitespace() {
            index += 1;
            continue;
        }

        // Assignment operator.
        if byte == b':' {
            tokens.push(Token::Colon);
            index += 1;
            continue;
        }

        // Check if the token can start a negative number.
        let can_start_negative = tokens.last().map_or(true, |t| t.can_start_negative());

        // Read number.
        if (byte == b'-'
            && can_start_negative
            && index + 1 < bytes.len()
            && bytes[index + 1].is_ascii_digit())
            || byte.is_ascii_digit()
        {
            let mut sign: i64 = 1;
            if byte == b'-' {
                sign = -1;
                index += 1;
            }

            let mut value: i64 = 0;
            let mut saw_digit = false;

            while index < bytes.len() && bytes[index].is_ascii_digit() {
                saw_digit = true;
                let digit = (bytes[index] - b'0') as i64;
                value = value
                    .checked_mul(10)
                    .and_then(|value| value.checked_add(digit))
                    .ok_or(())?;
                index += 1;
            }

            if !saw_digit {
                return Err(());
            }

            tokens.push(Token::Number(value * sign));
            continue;
        }

        // Read global variable.
        if byte.is_ascii_lowercase() {
            tokens.push(Token::Global(byte));
            index += 1;
            continue;
        }

        // Read symbol.
        tokens.push(Token::Symbol(byte));
        index += 1;
    }

    Ok(tokens)
}
