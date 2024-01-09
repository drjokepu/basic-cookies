use super::{lalrpop_util, CookieLexer, CookieLexerError, CookieToken};
use std::fmt::{Display, Error as FormatterError, Formatter};

const BASIC_COOKIE_ERROR_DESCRIPTION: &'static str = "Cookie Parsing Error";
const INTERNAL_ERROR_DESCRIPTION: &'static str = "Internal Error";
const PARSE_ERROR_DESCRIPTION: &'static str = "Parse Error";

lalrpop_mod!(cookie_grammar);

#[derive(Debug)]
pub struct Cookie<'a> {
    name: &'a str,
    value: &'a str,
}

impl<'a> Cookie<'a> {
    /// Parses an [RFC 6265](https://tools.ietf.org/html/rfc6265.html#section-4.2.1) compliant cookie string.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_cookies::Cookie;
    ///
    /// let parsed_cookies = Cookie::parse("cookie1=value1; cookie2=value2").unwrap();
    ///
    /// assert_eq!("cookie1", parsed_cookies[0].get_name());
    /// assert_eq!("value1", parsed_cookies[0].get_value());
    ///
    /// assert_eq!("cookie2", parsed_cookies[1].get_name());
    /// assert_eq!("value2", parsed_cookies[1].get_value());
    /// ```
    pub fn parse(input: &'a str) -> Result<Vec<Cookie<'a>>, Error> {
        Ok(cookie_grammar::CookiesParser::new()
            .parse(CookieLexer::new(input))
            .map_err(ParseError::from_lalrpop_parse_error_to_error)?
            .clone_to_vec()
            .iter()
            .rev()
            .map(|tok| tok.with_str(input))
            .collect::<Result<Vec<Cookie>, Error>>()?)
    }

    /// Gets the name of the cookie.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_cookies::Cookie;
    ///
    /// let parsed_cookies = Cookie::parse("name=value").unwrap();
    /// assert_eq!("name", parsed_cookies[0].get_name());
    /// ```
    pub fn get_name(&self) -> &'a str {
        self.name
    }

    /// Gets the value of the cookie.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_cookies::Cookie;
    ///
    /// let parsed_cookies = Cookie::parse("name=value").unwrap();
    /// assert_eq!("value", parsed_cookies[0].get_value());
    /// ```
    pub fn get_value(&self) -> &'a str {
        self.value
    }
}

#[derive(Debug)]
pub enum Error {
    InternalError(InternalError),
    ParseError(ParseError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
        f.write_str(BASIC_COOKIE_ERROR_DESCRIPTION)?;
        f.write_str(": ")?;
        match self {
            Error::InternalError(err) => err.fmt(f),
            Error::ParseError(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        BASIC_COOKIE_ERROR_DESCRIPTION
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::InternalError(err) => Some(err),
            Error::ParseError(err) => Some(err),
        }
    }
}

#[derive(Debug)]
pub struct InternalError(InternalErrorKind);

impl InternalError {
    pub(crate) fn to_error(self) -> Error {
        Error::InternalError(self)
    }
}

impl Display for InternalError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
        f.write_str(INTERNAL_ERROR_DESCRIPTION)
    }
}

impl std::error::Error for InternalError {
    fn description(&self) -> &str {
        INTERNAL_ERROR_DESCRIPTION
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug)]
enum InternalErrorKind {
    NonTerminalIndexBeyondBoundaries,
}

type LalrpopError = lalrpop_util::ParseError<usize, CookieToken, CookieLexerError>;

#[derive(Debug)]
pub struct ParseError {
    lalrpop_error: LalrpopError,
}

impl ParseError {
    pub(crate) fn from_lalrpop_parse_error_to_error(src: LalrpopError) -> Error {
        ParseError { lalrpop_error: src }.to_error()
    }

    fn to_error(self) -> Error {
        Error::ParseError(self)
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
        f.write_str(PARSE_ERROR_DESCRIPTION)?;
        f.write_str(": ")?;
        self.lalrpop_error.fmt(f)
    }
}

impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        PARSE_ERROR_DESCRIPTION
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        Some(&self.lalrpop_error)
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.lalrpop_error)
    }
}

mod terminals {
    use super::nonterminals::NonTerminalSpan;
    use super::Cookie as FullyParsedCookie;
    use super::{Error, InternalError};

    #[derive(Clone, Debug)]
    pub struct Cookie {
        pub(super) key: NonTerminalSpan,
        pub(super) value: NonTerminalSpan,
    }

    impl Cookie {
        pub(super) fn with_str<'a>(&self, data: &'a str) -> Result<FullyParsedCookie<'a>, Error> {
            Ok(FullyParsedCookie {
                name: self.key.as_str(data).map_err(InternalError::to_error)?,
                value: self.value.as_str(data).map_err(InternalError::to_error)?,
            })
        }
    }
}

mod nonterminals {
    use super::{InternalError, InternalErrorKind};

    #[derive(Clone, Debug)]
    pub struct NonTerminalSpan {
        start: usize,
        end: usize,
    }

    impl NonTerminalSpan {
        pub(crate) fn new(start: usize, end: usize) -> NonTerminalSpan {
            NonTerminalSpan {
                start: start,
                end: end,
            }
        }

        pub(crate) fn as_str<'a>(&self, data: &'a str) -> Result<&'a str, InternalError> {
            match data.get(self.start..self.end) {
                Some(res) => Ok(res),
                None => Err(InternalError(
                    InternalErrorKind::NonTerminalIndexBeyondBoundaries,
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cookie;

    #[test]
    fn get_name() {
        const COOKIE_KEY: &'static str = "cookie_key";
        const COOKIE_VALUE: &'static str = "cookie_value";

        let cookie = Cookie {
            name: COOKIE_KEY,
            value: COOKIE_VALUE,
        };

        assert_eq!(COOKIE_KEY, cookie.get_name());
    }

    #[test]
    fn get_value() {
        const COOKIE_KEY: &'static str = "cookie_key";
        const COOKIE_VALUE: &'static str = "cookie_value";

        let cookie = Cookie {
            name: COOKIE_KEY,
            value: COOKIE_VALUE,
        };

        assert_eq!(COOKIE_VALUE, cookie.get_value());
    }

    #[test]
    fn single_cookie() {
        const COOKIE_STR: &'static str = "test=1234";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("test", parsed_cookie.name);
        assert_eq!("1234", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_quoted() {
        const COOKIE_STR: &'static str = "quoted_test=\"quotedval\"";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("quoted_test", parsed_cookie.name);
        assert_eq!("quotedval", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_with_equals_in_value() {
        const COOKIE_STR: &'static str = "test=abc=123";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("test", parsed_cookie.name);
        assert_eq!("abc=123", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_before() {
        const COOKIE_STR: &'static str = " \x09 ztest=9876";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("ztest", parsed_cookie.name);
        assert_eq!("9876", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_with_single_space_before() {
        const COOKIE_STR: &'static str = " qtest=9878";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("qtest", parsed_cookie.name);
        assert_eq!("9878", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_after() {
        const COOKIE_STR: &'static str = "abcde=77766test \x09\x09    ";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("abcde", parsed_cookie.name);
        assert_eq!("77766test", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_with_single_space_after() {
        const COOKIE_STR: &'static str = "xyzzz=test3 ";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("xyzzz", parsed_cookie.name);
        assert_eq!("test3", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_before_and_after() {
        const COOKIE_STR: &'static str = " \x09 ztest=9876       ";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("ztest", parsed_cookie.name);
        assert_eq!("9876", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_name() {
        const COOKIE_STR: &'static str = "=nokey";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("", parsed_cookie.name);
        assert_eq!("nokey", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_name_with_ows_before() {
        const COOKIE_STR: &'static str = " =nokey";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("", parsed_cookie.name);
        assert_eq!("nokey", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_value() {
        const COOKIE_STR: &'static str = "noval=";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("noval", parsed_cookie.name);
        assert_eq!("", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_value_with_ows_after() {
        const COOKIE_STR: &'static str = "noval= ";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("noval", parsed_cookie.name);
        assert_eq!("", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_name_and_val() {
        const COOKIE_STR: &'static str = "=";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("", parsed_cookie.name);
        assert_eq!("", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_name_no_equals() {
        const COOKIE_STR: &'static str = "nokey";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("", parsed_cookie.name);
        assert_eq!("nokey", parsed_cookie.value);
    }

    #[test]
    fn two_cookies() {
        const COOKIE_STR: &'static str = "test1=01234; test2=testval";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(2, parsed_cookies.len());

        let parsed_cookie_0 = &parsed_cookies[0];
        assert_eq!("test1", parsed_cookie_0.name);
        assert_eq!("01234", parsed_cookie_0.value);

        let parsed_cookie_1 = &parsed_cookies[1];
        assert_eq!("test2", parsed_cookie_1.name);
        assert_eq!("testval", parsed_cookie_1.value);
    }

    #[test]
    fn three_cookies() {
        const COOKIE_STR: &'static str = "test1=0x1234; test2=test2; third_val=v4lue";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(3, parsed_cookies.len());

        let parsed_cookie_0 = &parsed_cookies[0];
        assert_eq!("test1", parsed_cookie_0.name);
        assert_eq!("0x1234", parsed_cookie_0.value);

        let parsed_cookie_1 = &parsed_cookies[1];
        assert_eq!("test2", parsed_cookie_1.name);
        assert_eq!("test2", parsed_cookie_1.value);

        let parsed_cookie_2 = &parsed_cookies[2];
        assert_eq!("third_val", parsed_cookie_2.name);
        assert_eq!("v4lue", parsed_cookie_2.value);
    }

    #[test]
    fn three_cookies_ows_before() {
        const COOKIE_STR: &'static str = " test1=0x1234; test2=test2; third_val=v4lue";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(3, parsed_cookies.len());

        let parsed_cookie_0 = &parsed_cookies[0];
        assert_eq!("test1", parsed_cookie_0.name);
        assert_eq!("0x1234", parsed_cookie_0.value);

        let parsed_cookie_1 = &parsed_cookies[1];
        assert_eq!("test2", parsed_cookie_1.name);
        assert_eq!("test2", parsed_cookie_1.value);

        let parsed_cookie_2 = &parsed_cookies[2];
        assert_eq!("third_val", parsed_cookie_2.name);
        assert_eq!("v4lue", parsed_cookie_2.value);
    }

    #[test]
    fn three_cookies_ows_after() {
        const COOKIE_STR: &'static str = "test1=0x1234; test2=test2; third_val=v4lue   ";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(3, parsed_cookies.len());

        let parsed_cookie_0 = &parsed_cookies[0];
        assert_eq!("test1", parsed_cookie_0.name);
        assert_eq!("0x1234", parsed_cookie_0.value);

        let parsed_cookie_1 = &parsed_cookies[1];
        assert_eq!("test2", parsed_cookie_1.name);
        assert_eq!("test2", parsed_cookie_1.value);

        let parsed_cookie_2 = &parsed_cookies[2];
        assert_eq!("third_val", parsed_cookie_2.name);
        assert_eq!("v4lue", parsed_cookie_2.value);
    }

    #[test]
    fn three_cookies_ows_before_and_after() {
        const COOKIE_STR: &'static str = "   test1=0x1234; test2=test2; third_val=v4lue ";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(3, parsed_cookies.len());

        let parsed_cookie_0 = &parsed_cookies[0];
        assert_eq!("test1", parsed_cookie_0.name);
        assert_eq!("0x1234", parsed_cookie_0.value);

        let parsed_cookie_1 = &parsed_cookies[1];
        assert_eq!("test2", parsed_cookie_1.name);
        assert_eq!("test2", parsed_cookie_1.value);

        let parsed_cookie_2 = &parsed_cookies[2];
        assert_eq!("third_val", parsed_cookie_2.name);
        assert_eq!("v4lue", parsed_cookie_2.value);
    }

    #[test]
    fn three_cookies_no_spacing() {
        const COOKIE_STR: &'static str = "test1=0x1234;test2=test2;third_val=v4lue";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(3, parsed_cookies.len());

        let parsed_cookie_0 = &parsed_cookies[0];
        assert_eq!("test1", parsed_cookie_0.name);
        assert_eq!("0x1234", parsed_cookie_0.value);

        let parsed_cookie_1 = &parsed_cookies[1];
        assert_eq!("test2", parsed_cookie_1.name);
        assert_eq!("test2", parsed_cookie_1.value);

        let parsed_cookie_2 = &parsed_cookies[2];
        assert_eq!("third_val", parsed_cookie_2.name);
        assert_eq!("v4lue", parsed_cookie_2.value);
    }
}
