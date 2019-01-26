use std::fmt::{Display, Error as FormatterError, Formatter};

const COOKIE_LEXER_ERROR_DESCRIPTION: &'static str = "Cookie Lexer Error";

#[derive(Debug)]
pub struct CookieLexerError;

impl PartialEq<CookieLexerError> for CookieLexerError {
    fn eq(&self, _other: &CookieLexerError) -> bool {
        false
    }

    fn ne(&self, _other: &CookieLexerError) -> bool {
        false
    }
}

impl Display for CookieLexerError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
        f.write_str(COOKIE_LEXER_ERROR_DESCRIPTION)
    }
}

impl std::error::Error for CookieLexerError {
    fn description(&self) -> &str {
        COOKIE_LEXER_ERROR_DESCRIPTION
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }

    fn source(&self) -> Option<&(std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum CookieToken {
    CookieOctets,
    TokenOrCookieOctets,
    Equals,
    Semicolon,
    Whitespace,
    Space,
    DoubleQuote,
}

impl CookieToken {
    fn as_str(&self) -> &'static str {
        match self {
            CookieToken::CookieOctets => "CookieToken::CookieOctets",
            CookieToken::TokenOrCookieOctets => "CookieToken::TokenOrCookieOctets",
            CookieToken::Equals => "CookieToken::Equals",
            CookieToken::Semicolon => "CookieToken::Semicolon",
            CookieToken::Whitespace => "CookieToken::Whitespace",
            CookieToken::Space => "CookieToken::Space",
            CookieToken::DoubleQuote => "CookieToken::DoubleQuote",
        }
    }
}

impl Display for CookieToken {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
        f.write_str(self.as_str())
    }
}

macro_rules! try_str_match {
    ($token:path, $pattern:expr, $data:expr, $cursor:expr) => {{
        const PATTERN_STR: &'static str = $pattern;
        if matching::is_str_match($data, PATTERN_STR) {
            let token_idx = $cursor;
            let token_end = token_idx + PATTERN_STR.len();
            $cursor = token_end;
            return Some(Ok((token_idx, $token, token_end)));
        }
    };};
}

macro_rules! try_fn_match {
    ($token:path, $fn:path, $data:expr, $cursor:expr) => {{
        let mut is_match = true;
        let mut last_cursor_char: Option<(usize, char)> = None;
        for (cursor_char_idx, cursor_char) in $data[$cursor..].iter() {
            if $fn(*cursor_char) {
                last_cursor_char = Some((*cursor_char_idx, *cursor_char));
            } else {
                if last_cursor_char == None {
                    is_match = false;
                }

                break;
            }
        }

        if is_match {
            if let Some((last_cursor_char_idx_val, last_cursor_char_val)) = last_cursor_char {
                let token_idx = $cursor;
                let token_end = last_cursor_char_idx_val + last_cursor_char_val.len_utf8();
                $cursor = token_end;
                return Some(Ok((token_idx, $token, token_end)));
            }
        }
    };};
}

macro_rules! try_nonrepeating_char_match {
    ($token:path, $chr:expr, $data:expr, $cursor:expr) => {{
        let (_, char_val) = $data[$cursor];
        if char_val == $chr
            && ($data.len() == $cursor + 1 || {
                let (_, next_char) = $data[$cursor + 1];
                next_char != $chr
            })
        {
            let token_idx = $cursor;
            let token_end = token_idx + char_val.len_utf8();
            $cursor = token_end;
            return Some(Ok((token_idx, $token, token_end)));
        }
    };};
}

pub(crate) struct CookieLexer<'input> {
    cursor: usize,
    data: &'input str,
    char_indices: Vec<(usize, char)>,
}

impl<'input> CookieLexer<'input> {
    pub fn new(data: &'input str) -> CookieLexer<'input> {
        CookieLexer {
            cursor: 0,
            data: data,
            char_indices: data.char_indices().collect(),
        }
    }

    fn substr_at_cursor(&self) -> Option<&'input str> {
        self.data.get(self.cursor..)
    }

    fn get_next_token(&mut self) -> Option<Result<(usize, CookieToken, usize), CookieLexerError>> {
        let cursor_str = match self.substr_at_cursor() {
            Some(val) => val,
            None => return None,
        };

        if cursor_str.len() == 0 {
            return None;
        }

        try_nonrepeating_char_match!(CookieToken::Space, ' ', self.char_indices, self.cursor);

        try_str_match!(CookieToken::Equals, "=", cursor_str, self.cursor);
        try_str_match!(CookieToken::Semicolon, ";", cursor_str, self.cursor);
        try_str_match!(CookieToken::DoubleQuote, "\"", cursor_str, self.cursor);

        try_fn_match!(
            CookieToken::Whitespace,
            matching::is_whitespace_char,
            self.char_indices,
            self.cursor
        );

        self.get_next_pattern_token()
    }

    fn get_next_pattern_token(
        &mut self,
    ) -> Option<Result<(usize, CookieToken, usize), CookieLexerError>> {
        let mut can_be_token = true;
        let mut token_end_idx = 0_usize;

        for (_, cursor_char) in self.char_indices[self.cursor..].iter() {
            match CookieLexer::char_token_class(*cursor_char) {
                CharTokenClass::TokenOrCookieOctets => {
                    token_end_idx += cursor_char.len_utf8();
                }
                CharTokenClass::CookieOctets => {
                    can_be_token = false;
                    token_end_idx += cursor_char.len_utf8();
                }
                CharTokenClass::None => {
                    if token_end_idx > 0_usize {
                        break;
                    } else {
                        return None;
                    }
                }
            };
        }

        if token_end_idx > 0 {
            let token = if can_be_token {
                CookieToken::TokenOrCookieOctets
            } else {
                CookieToken::CookieOctets
            };

            let token_start = self.cursor;
            let token_end = self.cursor + token_end_idx;
            self.cursor = token_end;

            Some(Ok((token_start, token, token_end)))
        } else {
            None
        }
    }

    fn char_token_class(c: char) -> CharTokenClass {
        match c {
            '\x21'
            | '\x23'...'\x27'
            | '\x2a'
            | '\x2b'
            | '\x2d'
            | '\x2e'
            | '\x30'...'\x39'
            | '\x41'...'\x5a'
            | '\x5e'...'\x7a'
            | '\x7c'
            | '\x7e' => CharTokenClass::TokenOrCookieOctets,
            '\x28'
            | '\x29'
            | '\x2f'
            | '\x3a'
            | '\x3c'
            | '\x3e'...'\x40'
            | '\x5b'
            | '\x5d'
            | '\x7b'
            | '\x7d' => CharTokenClass::CookieOctets, // excludes = (x3d)
            _ => CharTokenClass::None,
        }
    }
}

impl<'input> Display for CookieLexer<'input> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
        f.write_fmt(format_args!(
            "Cookie Lexer, cursor at position {}.",
            self.cursor
        ))
    }
}

enum CharTokenClass {
    None,
    CookieOctets,
    TokenOrCookieOctets,
}

impl<'input> Iterator for CookieLexer<'input> {
    type Item = Result<(usize, CookieToken, usize), CookieLexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.get_next_token()
    }
}

mod matching {
    pub fn is_str_match(data: &str, pattern: &str) -> bool {
        data.starts_with(pattern)
    }

    pub fn is_whitespace_char(c: char) -> bool {
        match c {
            '\x09' | '\x20' => true,
            _ => false,
        }
    }

    #[cfg(test)]
    mod tests {
        mod is_str_match {
            use super::super::is_str_match;

            #[test]
            fn yes_equal() {
                assert!(is_str_match("abc", "abc"));
            }

            #[test]
            fn yes_prefix() {
                assert!(is_str_match("abcdef", "abc"));
            }

            #[test]
            fn no_same_length() {
                assert!(!is_str_match("abc", "def"));
            }

            #[test]
            fn no_shared_prefix() {
                assert!(!is_str_match("abcxyx", "abcijkl"));
            }

            #[test]
            fn no_prefix_other_way_around() {
                assert!(!is_str_match("abc", "abcdef"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    mod get_next_token {
        use super::super::{CookieLexer, CookieToken};

        #[test]
        fn equals() {
            assert_eq!(
                Some(Ok((0, CookieToken::Equals, 1))),
                CookieLexer::new("=").get_next_token()
            );
        }

        #[test]
        fn equals_and_token() {
            let mut lexer = CookieLexer::new("=Hello");

            assert_eq!(
                Some(Ok((0, CookieToken::Equals, 1))),
                lexer.get_next_token()
            );

            assert_eq!(
                Some(Ok((1, CookieToken::TokenOrCookieOctets, 6))),
                lexer.get_next_token()
            );
        }

        #[test]
        fn token() {
            assert_eq!(
                Some(Ok((0, CookieToken::TokenOrCookieOctets, 5))),
                CookieLexer::new("Hello").get_next_token()
            );
        }

        #[test]
        fn token_and_equals() {
            let mut lexer = CookieLexer::new("Hello=");

            assert_eq!(
                Some(Ok((0, CookieToken::TokenOrCookieOctets, 5))),
                lexer.get_next_token()
            );

            assert_eq!(
                Some(Ok((5, CookieToken::Equals, 6))),
                lexer.get_next_token()
            );
        }

        #[test]
        fn cookie_octets() {
            assert_eq!(
                Some(Ok((0, CookieToken::CookieOctets, 6))),
                CookieLexer::new("(test)").get_next_token()
            );
        }

        #[test]
        fn equals_and_cookie_octets() {
            let mut lexer = CookieLexer::new("=Hello");

            assert_eq!(
                Some(Ok((0, CookieToken::Equals, 1))),
                lexer.get_next_token()
            );

            assert_eq!(
                Some(Ok((1, CookieToken::TokenOrCookieOctets, 6))),
                lexer.get_next_token()
            );
        }

        #[test]
        fn whitespace() {
            assert_eq!(
                Some(Ok((0, CookieToken::Whitespace, 5))),
                CookieLexer::new("  \x09  ").get_next_token()
            );
        }

        #[test]
        fn whitespace_two_spaces() {
            assert_eq!(
                Some(Ok((0, CookieToken::Whitespace, 2))),
                CookieLexer::new("  ").get_next_token()
            );
        }

        #[test]
        fn whitespace_and_token_or_cookie_octets() {
            let mut lexer = CookieLexer::new("  \x09  test");

            assert_eq!(
                Some(Ok((0, CookieToken::Whitespace, 5))),
                lexer.get_next_token()
            );

            assert_eq!(
                Some(Ok((5, CookieToken::TokenOrCookieOctets, 9))),
                lexer.get_next_token()
            );
        }

        #[test]
        fn equals_and_whitespace_and_token_or_cookie_octets() {
            let mut lexer = CookieLexer::new("=  \x09  test");

            assert_eq!(
                Some(Ok((0, CookieToken::Equals, 1))),
                lexer.get_next_token()
            );

            assert_eq!(
                Some(Ok((1, CookieToken::Whitespace, 6))),
                lexer.get_next_token()
            );

            assert_eq!(
                Some(Ok((6, CookieToken::TokenOrCookieOctets, 10))),
                lexer.get_next_token()
            );
        }

        #[test]
        fn space() {
            assert_eq!(
                Some(Ok((0, CookieToken::Space, 1))),
                CookieLexer::new(" ").get_next_token()
            );
        }

        #[test]
        fn space_and_equals_and_space() {
            let mut lexer = CookieLexer::new(" = ");

            assert_eq!(Some(Ok((0, CookieToken::Space, 1))), lexer.get_next_token());

            assert_eq!(
                Some(Ok((1, CookieToken::Equals, 2))),
                lexer.get_next_token()
            );

            assert_eq!(Some(Ok((2, CookieToken::Space, 3))), lexer.get_next_token());
        }
    }
}
