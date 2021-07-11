use crate::tag::{NbtCompound, NbtList, NbtTag};
use std::{
    borrow::Cow,
    char,
    convert::AsRef,
    error::Error,
    fmt::{self, Debug, Display, Formatter},
    iter::Peekable,
    mem,
    str::{self, CharIndices},
};

/// Parses the given string into an NBT tag compound.
///
/// # Examples
///
/// ```
/// # use quartz_nbt::*;
/// use quartz_nbt::snbt;
///
/// let mut compound = NbtCompound::new();
/// compound.insert("short", -10i16);
/// compound.insert("string", "fizzbuzz");
/// compound.insert("array", vec![1i64, 1, 2, 3, 5]);
///
/// const SNBT: &str = "{short: -10s, string: fizzbuzz, array: [L; 1, 1, 2, 3, 5]}";
///
/// assert_eq!(compound, snbt::parse(SNBT).unwrap());
/// ```
///
/// The parser will immediately quit when it encounters a syntax error. Displaying these errors
/// will provide useful information about where the error occurred, what went wrong, and what
/// was expected.
///
/// ```
/// use quartz_nbt::snbt;
///
/// const ERRONEOUS_SNBT: &str = "{garbage:; -'bleh ]";
/// let result = snbt::parse(ERRONEOUS_SNBT);
/// assert!(result.is_err());
/// assert_eq!(
///     result.unwrap_err().to_string(),
///     "Unexpected token at column 9 near '{garbage:;', expected value"
/// );
/// ```
pub fn parse<T: AsRef<str> + ?Sized>(string_nbt: &T) -> Result<NbtCompound, SnbtError> {
    let mut tokens = Lexer::new(string_nbt.as_ref());
    let open_curly = tokens.assert_next(Token::OpenCurly)?;
    parse_compound_tag(&mut tokens, &open_curly)
}

// Parses the next value in the token stream
fn parse_next_value<'a>(
    tokens: &mut Lexer<'a>,
    delimiter: Option<fn(char) -> bool>,
) -> Result<NbtTag, SnbtError> {
    let token = tokens.next(delimiter).transpose()?;
    parse_value(tokens, token)
}

// Parses a token into a value
fn parse_value<'a>(tokens: &mut Lexer<'a>, token: Option<TokenData>) -> Result<NbtTag, SnbtError> {
    match token {
        // Open curly brace indicates a compound tag is present
        Some(
            td
            @ TokenData {
                token: Token::OpenCurly,
                ..
            },
        ) => parse_compound_tag(tokens, &td).map(Into::into),

        // Open square brace indicates that some kind of list is present
        Some(
            td
            @
            TokenData {
                token: Token::OpenSquare,
                ..
            },
        ) => parse_list(tokens, &td),

        // Could be a value token or delimiter token
        Some(td @ _) => match td.into_tag() {
            Ok(tag) => Ok(tag),
            Err(td) => Err(SnbtError::unexpected_token(tokens.raw, Some(&td), "value")),
        },

        // We expected a value but ran out of data
        None => Err(SnbtError::unexpected_eos("value")),
    }
}

// Parses a list, which can be either a generic tag list or vector of primitives
fn parse_list<'a>(tokens: &mut Lexer<'a>, open_square: &TokenData) -> Result<NbtTag, SnbtError> {
    const DELIMITER: Option<fn(char) -> bool> = Some(|ch| matches!(ch, ',' | ']' | ';'));

    match tokens.next(DELIMITER).transpose()? {
        // Empty list ('[]') with no type specifier is treated as an empty NBT tag list
        Some(TokenData {
            token: Token::ClosedSquare,
            ..
        }) => Ok(NbtList::new().into()),

        // A string as the first "element" can either be a type specifier such as in [I; 1, 2], or
        // a regular string in a tag list, such as in ['i', 'j', 'k'].
        Some(TokenData {
            token:
                Token::String {
                    value: string,
                    quoted,
                },
            index,
            char_width,
        }) => {
            // Peek at the next token to see if it's a semicolon, which would indicate a primitive vector
            match tokens.peek(DELIMITER) {
                // Parse as a primitive vector
                Some(Ok(TokenData {
                    token: Token::Semicolon,
                    ..
                })) => {
                    if quoted {
                        return Err(SnbtError::unexpected_token_at(
                            tokens.raw,
                            index,
                            char_width,
                            "'B', 'I', or 'L'",
                        ));
                    }

                    // Moves past the peeked semicolon
                    tokens.next(None);

                    // Determine the primitive type and parse it
                    match string.as_str() {
                        "b" | "B" => parse_prim_list::<u8>(tokens, open_square),
                        "i" | "I" => parse_prim_list::<i32>(tokens, open_square),
                        "l" | "L" => parse_prim_list::<i64>(tokens, open_square),
                        _ => Err(SnbtError::unexpected_token_at(
                            tokens.raw,
                            index,
                            char_width,
                            "'B', 'I', or 'L'",
                        )),
                    }
                }

                // Parse as a tag list (token errors are delegated to this function)
                _ => parse_tag_list(tokens, NbtTag::String(string)).map(Into::into),
            }
        }

        // Any other pattern is delegated to the general tag list parser
        td @ _ => {
            let first_element = parse_value(tokens, td)?;
            parse_tag_list(tokens, first_element).map(Into::into)
        }
    }
}

fn parse_prim_list<'a, T>(
    tokens: &mut Lexer<'a>,
    open_square: &TokenData,
) -> Result<NbtTag, SnbtError>
where
    Token: Into<Result<T, Token>>,
    NbtTag: From<Vec<T>>,
{
    let mut list: Vec<T> = Vec::new();
    // Zero is used as a niche value so the first iteration of the loop runs correctly
    let mut comma: Option<usize> = Some(0);

    loop {
        match tokens.next(Some(|ch| ch == ',' || ch == ']')).transpose()? {
            // Finish off the list
            Some(TokenData {
                token: Token::ClosedSquare,
                ..
            }) => match comma {
                Some(0) | None => return Ok(list.into()),
                Some(index) => return Err(SnbtError::trailing_comma(tokens.raw, index)),
            },

            // Indicates another value should be parsed
            Some(TokenData {
                token: Token::Comma,
                index,
                ..
            }) => comma = Some(index),

            // Attempt to convert the token into a value
            Some(td @ _) => {
                // Make sure a value was expected
                match comma {
                    Some(_) => {
                        match td.into_value::<T>() {
                            Ok(value) => list.push(value),
                            Err(td) =>
                                return Err(SnbtError::non_homogenous_list(
                                    tokens.raw,
                                    td.index,
                                    td.char_width,
                                )),
                        }

                        comma = None;
                    }

                    None =>
                        return Err(SnbtError::unexpected_token(
                            tokens.raw,
                            Some(&td),
                            Token::Comma.as_expectation(),
                        )),
                }
            }

            None => return Err(SnbtError::unmatched_brace(tokens.raw, open_square.index)),
        }
    }
}

fn parse_tag_list<'a>(tokens: &mut Lexer<'a>, first_element: NbtTag) -> Result<NbtList, SnbtError> {
    const DELIMITER: Option<fn(char) -> bool> = Some(|ch| ch == ',' || ch == ']');

    // Construct the list and use the first element to determine the list's type
    let mut list = NbtList::new();
    let descrim = mem::discriminant(&first_element);
    list.push(first_element);

    loop {
        // No delimiter needed since we only expect ']' and ','
        match tokens.next(None).transpose()? {
            // Finish off the list
            Some(TokenData {
                token: Token::ClosedSquare,
                ..
            }) => return Ok(list),

            // Indicates another value should be parsed
            Some(TokenData {
                token: Token::Comma,
                ..
            }) => {
                let (index, char_width) = match tokens.peek(DELIMITER) {
                    Some(&Ok(TokenData {
                        index, char_width, ..
                    })) => (index, char_width),
                    _ => (0, 0),
                };
                let element = parse_next_value(tokens, DELIMITER)?;

                // Ensure type homogeneity
                if mem::discriminant(&element) != descrim {
                    return Err(SnbtError::non_homogenous_list(
                        tokens.raw, index, char_width,
                    ));
                }

                list.push(element);
            }

            // Some invalid token
            td @ _ =>
                return Err(SnbtError::unexpected_token(
                    tokens.raw,
                    td.as_ref(),
                    "',' or ']'",
                )),
        }
    }
}

fn parse_compound_tag<'a>(
    tokens: &mut Lexer<'a>,
    open_curly: &TokenData,
) -> Result<NbtCompound, SnbtError> {
    let mut compound = NbtCompound::new();
    // Zero is used as a niche value so the first iteration of the loop runs correctly
    let mut comma: Option<usize> = Some(0);

    loop {
        match tokens.next(Some(|ch| ch == ':')).transpose()? {
            // Finish off the compound tag
            Some(TokenData {
                token: Token::ClosedCurly,
                ..
            }) => {
                match comma {
                    // First loop iteration or no comma
                    Some(0) | None => return Ok(compound),
                    // Later iteration with a trailing comma
                    Some(index) => return Err(SnbtError::trailing_comma(tokens.raw, index)),
                }
            }

            // Parse a new key-value pair
            Some(TokenData {
                token: Token::String { value: key, .. },
                index,
                char_width,
            }) => {
                match comma {
                    // First loop iteration or a comma indicated that more data is present
                    Some(_) => {
                        tokens.assert_next(Token::Colon)?;
                        compound.insert(
                            key,
                            parse_next_value(tokens, Some(|ch| ch == ',' || ch == '}'))?,
                        );
                        comma = None;
                    }

                    // There was not a comma before this string so therefore the token is unexpected
                    None =>
                        return Err(SnbtError::unexpected_token_at(
                            tokens.raw,
                            index,
                            char_width,
                            Token::Comma.as_expectation(),
                        )),
                }
            }

            // Denote that another key-value pair is anticipated
            Some(TokenData {
                token: Token::Comma,
                index,
                ..
            }) => comma = Some(index),

            // Catch-all for unexpected tokens
            Some(td @ _) =>
                return Err(SnbtError::unexpected_token(
                    tokens.raw,
                    Some(&td),
                    "compound key, '}', or ','",
                )),

            // End of file / unmatched brace
            None => return Err(SnbtError::unmatched_brace(tokens.raw, open_curly.index)),
        }
    }
}

struct Lexer<'a> {
    raw: &'a str,
    chars: Peekable<CharIndices<'a>>,
    index: usize,
    raw_token_buffer: Cow<'a, str>,
    peeked: Option<Option<Result<TokenData, SnbtError>>>,
}

impl<'a> Lexer<'a> {
    fn new(raw: &'a str) -> Self {
        Lexer {
            raw,
            chars: raw.char_indices().peekable(),
            index: 0,
            raw_token_buffer: Cow::Owned(String::new()),
            peeked: None,
        }
    }

    fn peek(
        &mut self,
        delimiter: Option<fn(char) -> bool>,
    ) -> Option<&Result<TokenData, SnbtError>> {
        if self.peeked.is_none() {
            self.peeked = Some(self.next(delimiter));
        }

        self.peeked.as_ref().unwrap().as_ref()
    }

    fn next(
        &mut self,
        delimiter: Option<fn(char) -> bool>,
    ) -> Option<Result<TokenData, SnbtError>> {
        // Manage the peeking function
        match self.peeked.take() {
            Some(item) => return item,
            None => {}
        };

        // Skip whitespace
        while self.peek_ch()?.is_ascii_whitespace() {
            self.next_ch();
        }

        // Manage single-char tokens and pass multi-character tokens to a designated function
        let tk = match self.peek_ch()? {
            '{' => TokenData::new(Token::OpenCurly, self.index, 1),
            '}' => TokenData::new(Token::ClosedCurly, self.index, 1),
            '[' => TokenData::new(Token::OpenSquare, self.index, 1),
            ']' => TokenData::new(Token::ClosedSquare, self.index, 1),
            ',' => TokenData::new(Token::Comma, self.index, 1),
            ':' => TokenData::new(Token::Colon, self.index, 1),
            ';' => TokenData::new(Token::Semicolon, self.index, 1),
            _ => return Some(self.slurp_token(delimiter)),
        };

        self.next_ch();
        Some(Ok(tk))
    }

    #[inline]
    fn peek_ch(&mut self) -> Option<char> {
        self.chars.peek().map(|&(_, ch)| ch)
    }

    #[inline]
    fn next_ch(&mut self) -> Option<char> {
        let next = self.chars.next();
        if let Some((index, ch)) = next {
            self.index = index + ch.len_utf8();
        }
        next.map(|(_, ch)| ch)
    }

    // Asserts that the next token is the same type as the provided token
    fn assert_next(&mut self, token: Token) -> Result<TokenData, SnbtError> {
        match self.next(None).transpose()? {
            // We found a token so check the token type
            Some(td) =>
                if mem::discriminant(&td.token) == mem::discriminant(&token) {
                    Ok(td)
                } else {
                    Err(SnbtError::unexpected_token(
                        self.raw,
                        Some(&td),
                        token.as_expectation(),
                    ))
                },

            // No tokens were left so return an unexpected end of string error
            None => Err(SnbtError::unexpected_eos(token.as_expectation())),
        }
    }

    // Collects a token from the character iterator
    fn slurp_token(&mut self, delimiter: Option<fn(char) -> bool>) -> Result<TokenData, SnbtError> {
        let start = self.index;
        let mut char_width = 1;

        // State of the token slurper
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum State {
            Unquoted,
            InSingleQuotes,
            InDoubleQuotes,
        }

        let (state, ch0) = match self.next_ch() {
            Some('\'') => (State::InSingleQuotes, '\''),
            Some('"') => (State::InDoubleQuotes, '"'),
            Some(ch0) => (State::Unquoted, ch0),
            None => unreachable!("slurp_token called on an empty token"),
        };

        match state {
            State::Unquoted => {
                // Last non-whitespace character and its index
                let mut last_nws_char = ch0;
                let mut last_nws_char_pos = start;

                // Keep a rudimentary record of SNBT syntax within a string
                let mut curly_count = 0;
                let mut square_count = 0;
                let mut quotes = 0;

                loop {
                    match self.peek_ch() {
                        // No characters left means we just finish the token
                        None => break,

                        // Ignore any subsequent quotations
                        Some('\\') => {
                            quotes |= 0b100;
                            self.next_ch();
                            continue;
                        }

                        // Manage quote counts
                        Some('\'') =>
                            if (quotes & !0b001) == 0 {
                                quotes ^= 0b001;
                            },
                        Some('"') =>
                            if (quotes & !0b010) == 0 {
                                quotes ^= 0b010;
                            },

                        // Default handler
                        Some(ch) => {
                            // We allow SNBT within SNBT strings, so make sure we're not in nested SNBT
                            if (curly_count + square_count + (quotes & 0b11)) == 0 {
                                match delimiter {
                                    // Break if the delimiter matches
                                    Some(delimiter) =>
                                        if delimiter(ch) {
                                            break;
                                        },

                                    // Default delimiter to expedite halting
                                    None =>
                                        if matches!(ch, '{' | '}' | '[' | ']' | ',' | ';') {
                                            break;
                                        },
                                }
                            }

                            // Manage brace counts
                            match ch {
                                '{' => curly_count += 1,
                                '}' =>
                                    if curly_count > 0 {
                                        curly_count -= 1;
                                    },
                                '[' => square_count += 1,
                                ']' =>
                                    if square_count > 0 {
                                        square_count -= 1;
                                    },
                                _ => {}
                            }

                            // Ensure that we don't include trailing whitespace in an unquoted token
                            if !ch.is_ascii_whitespace() {
                                char_width += 1;
                                last_nws_char = ch;
                                last_nws_char_pos = self.index;
                            }
                        }
                    }

                    // Read the character we peeked and unset the escape flag
                    self.next_ch();
                    quotes &= !0b100;
                }

                // Set the token
                self.raw_token_buffer =
                    Cow::Borrowed(&self.raw[start .. last_nws_char_pos + last_nws_char.len_utf8()]);
            }

            State::InSingleQuotes | State::InDoubleQuotes => loop {
                char_width += 1;

                match self.next_ch() {
                    Some('\\') => {
                        // One additional
                        char_width += 1;

                        if self.raw_token_buffer.is_empty() {
                            // Skip leading quote and exclude the backslash we just read
                            self.raw_token_buffer =
                                Cow::Borrowed(&self.raw[start + 1 .. self.index - 1]);
                        }

                        // Handle escape characters
                        match self.next_ch() {
                            // These are just directly quoted
                            Some(ch @ ('\'' | '"' | '\\')) =>
                                self.raw_token_buffer.to_mut().push(ch),

                            // Convert to the rust equivalent
                            Some('n') => self.raw_token_buffer.to_mut().push('\n'),
                            Some('r') => self.raw_token_buffer.to_mut().push('\r'),
                            Some('t') => self.raw_token_buffer.to_mut().push('\t'),

                            // Parse a unicode escape sequence
                            Some('u') => {
                                // Four additional
                                char_width += 4;

                                let mut buffer = [0u8; 4];
                                for by in buffer.iter_mut() {
                                    let ch = self.next_ch().ok_or(SnbtError::unexpected_eos(
                                        "four-character hex unicode value",
                                    ))?;
                                    if !ch.is_digit(16) {
                                        return Err(SnbtError::unexpected_token_at(
                                            self.raw,
                                            self.index - ch.len_utf8(),
                                            1,
                                            "a hexadecimal digit",
                                        ));
                                    }
                                    *by = ch as u8;
                                }

                                // All the characters are checked
                                let ch = u32::from_str_radix(
                                    str::from_utf8(buffer.as_ref()).unwrap(),
                                    16,
                                )
                                .ok()
                                .map(|n| char::from_u32(n))
                                .flatten()
                                .ok_or(
                                    SnbtError::unknown_escape_sequence(self.raw, self.index - 6, 6),
                                )?;

                                self.raw_token_buffer.to_mut().push(ch);
                            }

                            // Unknown sequence
                            Some(_) => {
                                return Err(SnbtError::unknown_escape_sequence(
                                    self.raw,
                                    self.index - 2,
                                    2,
                                ));
                            }

                            // Unexpected end of string / unmatched quotation
                            None => {
                                return Err(SnbtError::unmatched_quote(self.raw, start));
                            }
                        }
                    }

                    // Close off the string if the quote type matches
                    Some(ch @ ('\'' | '"')) => match (ch, state) {
                        ('\'', State::InSingleQuotes) | ('"', State::InDoubleQuotes) => {
                            if self.raw_token_buffer.is_empty() {
                                // Exclude surrounding quotes
                                self.raw_token_buffer =
                                    Cow::Borrowed(&self.raw[start + 1 .. self.index - 1]);
                            }
                            break;
                        }
                        _ => {}
                    },

                    // Directly quote a character
                    Some(..) => {}

                    // Unexpected end of string / unmatched quotation
                    None => {
                        return Err(SnbtError::unmatched_quote(self.raw, start));
                    }
                }
            },
        }

        self.parse_token(
            start,
            char_width,
            matches!(state, State::InSingleQuotes | State::InDoubleQuotes),
        )
    }

    // Parses an isolated token
    fn parse_token(
        &mut self,
        start: usize,
        char_width: usize,
        quoted: bool,
    ) -> Result<TokenData, SnbtError> {
        let token_string = mem::replace(&mut self.raw_token_buffer, Cow::Owned(String::new()));

        // Get the first and last characters
        let first = match token_string.chars().next() {
            Some(ch) => ch,

            // Only strings can be empty tokens
            None =>
                return Ok(TokenData::new(
                    Token::String {
                        value: String::new(),
                        quoted,
                    },
                    start,
                    2,
                )),
        };
        let last = token_string.chars().rev().next().unwrap();

        // Identify if the token is not a number (a string)
        if !(first == '-' || (first.is_ascii() && first.is_numeric())) {
            return Ok(TokenData::new(
                Token::String {
                    value: token_string.into_owned(),
                    quoted,
                },
                start,
                char_width,
            ));
        }

        // Determine whether to parse as an integer or decimal
        if token_string.contains('.') {
            // Parse with highest precision ignoring the type suffix
            let value: Option<f64> = match last {
                'f' | 'F' | 'd' | 'D' => token_string[.. token_string.len() - 1].parse().ok(),
                _ =>
                    if last.is_numeric() {
                        token_string.parse().ok()
                    } else {
                        None
                    },
            };

            // Apply the type suffix if it is valid
            match value {
                Some(value) => match last {
                    'f' | 'F' => Ok(TokenData::new(Token::Float(value), start, char_width)),
                    _ => Ok(TokenData::new(Token::Double(value), start, char_width)),
                },
                _ => Err(SnbtError::invalid_number(self.raw, start, char_width)),
            }
        } else {
            // Parse with highest precision ignoring the type suffix
            let value: Option<i64> = match last {
                'b' | 'B' | 's' | 'S' | 'l' | 'L' | 'f' | 'F' | 'd' | 'D' =>
                    token_string[.. token_string.len() - 1].parse().ok(),
                _ =>
                    if last.is_numeric() {
                        token_string.parse().ok()
                    } else {
                        None
                    },
            };

            // Apply the type suffix if it is valid
            match value {
                Some(value) => match last {
                    'b' | 'B' => Ok(TokenData::new(Token::Byte(value), start, char_width)),
                    's' | 'S' => Ok(TokenData::new(Token::Short(value), start, char_width)),
                    'l' | 'L' => Ok(TokenData::new(Token::Long(value), start, char_width)),
                    'f' | 'F' => Ok(TokenData::new(
                        Token::Float(value as f64),
                        start,
                        char_width,
                    )),
                    'd' | 'D' => Ok(TokenData::new(
                        Token::Double(value as f64),
                        start,
                        char_width,
                    )),
                    _ => Ok(TokenData::new(Token::Int(value), start, char_width)),
                },
                _ => Err(SnbtError::invalid_number(self.raw, start, char_width)),
            }
        }
    }
}

#[derive(Debug)]
struct TokenData {
    token: Token,
    index: usize,
    char_width: usize,
}

impl TokenData {
    fn new(token: Token, index: usize, char_width: usize) -> Self {
        TokenData {
            token,
            index,
            char_width,
        }
    }

    fn into_tag(self) -> Result<NbtTag, Self> {
        match self.token.into_tag() {
            Ok(tag) => Ok(tag),
            Err(tk) => Err(Self::new(tk, self.index, self.char_width)),
        }
    }

    fn into_value<T>(self) -> Result<T, Self>
    where Token: Into<Result<T, Token>> {
        match self.token.into() {
            Ok(value) => Ok(value),
            Err(tk) => Err(Self::new(tk, self.index, self.char_width)),
        }
    }
}

#[derive(Debug)]
enum Token {
    OpenCurly,
    ClosedCurly,
    OpenSquare,
    ClosedSquare,
    Comma,
    Colon,
    Semicolon,
    String { value: String, quoted: bool },
    Byte(i64),
    Short(i64),
    Int(i64),
    Long(i64),
    Float(f64),
    Double(f64),
}

impl Token {
    fn as_expectation(&self) -> &'static str {
        match self {
            Token::OpenCurly => "'{'",
            Token::ClosedCurly => "'}'",
            Token::OpenSquare => "'['",
            Token::ClosedSquare => "']'",
            Token::Comma => "','",
            Token::Colon => "':'",
            Token::Semicolon => "';'",
            _ => "value",
        }
    }

    fn into_tag(self) -> Result<NbtTag, Self> {
        match self {
            Token::String { value, .. } => Ok(NbtTag::String(value)),
            Token::Byte(value) => Ok(NbtTag::Byte(value as i8)),
            Token::Short(value) => Ok(NbtTag::Short(value as i16)),
            Token::Int(value) => Ok(NbtTag::Int(value as i32)),
            Token::Long(value) => Ok(NbtTag::Long(value)),
            Token::Float(value) => Ok(NbtTag::Float(value as f32)),
            Token::Double(value) => Ok(NbtTag::Double(value)),
            tk @ _ => Err(tk),
        }
    }
}

impl From<Token> for Result<String, Token> {
    fn from(tk: Token) -> Self {
        match tk {
            Token::String { value, .. } => Ok(value),
            tk @ _ => Err(tk),
        }
    }
}

macro_rules! opt_int_from_token {
    ($int:ty) => {
        impl From<Token> for Result<$int, Token> {
            fn from(tk: Token) -> Self {
                match tk {
                    Token::Byte(x) => Ok(x as $int),
                    Token::Short(x) => Ok(x as $int),
                    Token::Int(x) => Ok(x as $int),
                    Token::Long(x) => Ok(x as $int),
                    tk @ _ => Err(tk),
                }
            }
        }
    };
}

opt_int_from_token!(i8);
opt_int_from_token!(u8);
opt_int_from_token!(i16);
opt_int_from_token!(i32);
opt_int_from_token!(i64);

macro_rules! opt_float_from_token {
    ($float:ty) => {
        impl From<Token> for Result<$float, Token> {
            fn from(tk: Token) -> Self {
                match tk {
                    Token::Float(x) => Ok(x as $float),
                    Token::Double(x) => Ok(x as $float),
                    tk @ _ => Err(tk),
                }
            }
        }
    };
}

opt_float_from_token!(f32);
opt_float_from_token!(f64);

/// An error that occurs during the parsing process. This error contains a copy of a segment
/// of the input where the error occurred as well as metadata about the specific error. See
/// [`ParserErrorType`](crate::snbt::ParserErrorType) for the different error types.
pub struct SnbtError {
    segment: String,
    error: ParserErrorType,
}

impl SnbtError {
    fn unmatched_quote(input: &str, index: usize) -> Self {
        SnbtError {
            segment: Self::segment(input, index, 1, 7, 7),
            error: ParserErrorType::UnmatchedQuote { index },
        }
    }

    fn unknown_escape_sequence(input: &str, index: usize, char_width: usize) -> Self {
        SnbtError {
            segment: Self::segment(input, index, char_width, 0, 0),
            error: ParserErrorType::UnknownEscapeSequence,
        }
    }

    fn invalid_number(input: &str, index: usize, char_width: usize) -> Self {
        SnbtError {
            segment: Self::segment(input, index, char_width, 0, 0),
            error: ParserErrorType::InvalidNumber,
        }
    }

    fn unexpected_token(input: &str, token: Option<&TokenData>, expected: &'static str) -> Self {
        match token {
            Some(token) =>
                Self::unexpected_token_at(input, token.index, token.char_width, expected),
            None => Self::unexpected_eos(expected),
        }
    }

    fn unexpected_token_at(
        input: &str,
        index: usize,
        char_width: usize,
        expected: &'static str,
    ) -> Self {
        SnbtError {
            segment: Self::segment(input, index, char_width, 15, 0),
            error: ParserErrorType::UnexpectedToken { index, expected },
        }
    }

    fn unexpected_eos(expected: &'static str) -> Self {
        SnbtError {
            segment: String::new(),
            error: ParserErrorType::UnexpectedEOS { expected },
        }
    }

    fn trailing_comma(input: &str, index: usize) -> Self {
        SnbtError {
            segment: Self::segment(input, index, 1, 15, 1),
            error: ParserErrorType::TrailingComma { index },
        }
    }

    fn unmatched_brace(input: &str, index: usize) -> Self {
        SnbtError {
            segment: Self::segment(input, index, 1, 0, 15),
            error: ParserErrorType::UnmatchedBrace { index },
        }
    }

    fn non_homogenous_list(input: &str, index: usize, char_width: usize) -> Self {
        SnbtError {
            segment: Self::segment(input, index, char_width, 15, 0),
            error: ParserErrorType::NonHomogenousList { index },
        }
    }

    fn segment(
        input: &str,
        index: usize,
        char_width: usize,
        before: usize,
        after: usize,
    ) -> String {
        let start = input[.. index]
            .char_indices()
            .rev()
            .skip(before.checked_sub(1).unwrap_or(0))
            .next()
            .map(|(index, _)| index)
            .unwrap_or(0);
        let end = index
            + input[index ..]
                .char_indices()
                .skip(char_width.min(20) + after)
                .next()
                .map(|(index, _)| index)
                .unwrap_or(input.len());
        input[start .. end].to_owned()
    }
}

impl Display for SnbtError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.error {
            &ParserErrorType::UnmatchedQuote { index } => write!(
                f,
                "Unmatched quote: column {} near '{}'",
                index, self.segment
            ),
            &ParserErrorType::UnknownEscapeSequence =>
                write!(f, "Unknown escape sequence: '{}'", self.segment),
            &ParserErrorType::InvalidNumber => write!(f, "Invalid number: {}", self.segment),
            &ParserErrorType::UnexpectedToken { index, expected } => write!(
                f,
                "Unexpected token at column {} near '{}', expected {}",
                index, self.segment, expected
            ),
            &ParserErrorType::UnexpectedEOS { expected } =>
                write!(f, "Reached end of input but expected {}", expected),
            &ParserErrorType::TrailingComma { index } =>
                write!(f, "Trailing comma at column {}: '{}'", index, self.segment),
            &ParserErrorType::UnmatchedBrace { index } => write!(
                f,
                "Unmatched brace at column {} near '{}'",
                index, self.segment
            ),
            &ParserErrorType::NonHomogenousList { index } => write!(
                f,
                "Non-homogenous typed list at column {} near '{}'",
                index, self.segment
            ),
        }
    }
}

impl Debug for SnbtError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.error, f)
    }
}

impl Error for SnbtError {}

/// A specific type of parser error. This enum includes metadata about each specific error.
#[derive(Clone, Debug)]
pub enum ParserErrorType {
    /// An unmatched single or double quote.
    UnmatchedQuote {
        /// The index of the unmatched quote.
        index: usize,
    },
    /// An unknown or invalid escape sequence.
    UnknownEscapeSequence,
    /// An invalid number.
    InvalidNumber,
    /// An unexpected token was encountered.
    UnexpectedToken {
        /// The index of the token.
        index: usize,
        /// The expected token or sequence of tokens.
        expected: &'static str,
    },
    /// The end of the string (EOS) was encountered before it was expected.
    UnexpectedEOS {
        /// The expected token or sequence of tokens.
        expected: &'static str,
    },
    /// A trailing comma was encountered in a list or compound.
    TrailingComma {
        /// The index of the trailing comma.
        index: usize,
    },
    /// An unmatched curly or square bracket was encountered.
    UnmatchedBrace {
        /// The index of the unmatched brace.
        index: usize,
    },
    /// A non-homogenous list was encountered.
    NonHomogenousList {
        /// The index where the invalid list value was encountered.
        index: usize,
    },
}
