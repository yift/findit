use std::{iter::Peekable, path::PathBuf};

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, offset::LocalResult};

use crate::{
    parser::{
        ast::{
            access::Access,
            function_name::FunctionName,
            operator::{
                ArithmeticOperator, BinaryOperator, BitwiseOperator, ComparisonOperator,
                LogicalOperator,
            },
        },
        method::MethodName,
    },
    value::Value,
};

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Token {
    Value(Value),
    OpenBrackets,
    CloseBrackets,
    Comma,
    Not,
    BinaryOperator(BinaryOperator),
    SimpleAccess(Access),
    Is,
    Dir,
    File,
    Link,
    None,
    Some,
    If,
    Then,
    Else,
    End,
    Case,
    When,
    Between,
    Format,
    Parse,
    From,
    For,
    Into,
    Spawn,
    Execute,
    Asc,
    Desc,
    As,
    Date,
    Boolean,
    String,
    Number,
    Replace,
    To,
    Pattern,
    ListStart,
    ListEnds,
    FunctionName(FunctionName),
    BindingName(String),
    MethodName(MethodName),
    With,
    Do,
    ClassStarts,
    ClassEnds,
    ClassFieldName(String),
    ClassFieldAccess(String),
}

#[derive(Debug)]
pub(crate) struct TokenError {
    pub(crate) cause: String,
}

impl Token {
    pub(crate) fn new(
        chars: &mut Peekable<impl Iterator<Item = (usize, char)>>,
    ) -> Result<Option<Self>, TokenError> {
        let chr = loop {
            let Some((_, chr)) = chars.peek() else {
                chars.next();
                return Ok(None);
            };
            if !chr.is_ascii_whitespace() {
                break chr;
            } else {
                chars.next();
            }
        };
        match chr {
            '0'..='9' => Ok(Some(Token::Value(Value::Number(read_number(chars))))),
            '"' => Ok(Some(Token::Value(Value::String(read_string(chars)?)))),
            '$' => Ok(Some(Token::BindingName(read_binding_name(chars)?))),
            '@' => Ok(Some(read_path_or_file(chars)?)),
            '(' => {
                chars.next();
                Ok(Some(Token::OpenBrackets))
            }
            ')' => {
                chars.next();
                Ok(Some(Token::CloseBrackets))
            }
            ',' => {
                chars.next();
                Ok(Some(Token::Comma))
            }
            '+' => {
                chars.next();
                Ok(Some(Token::BinaryOperator(BinaryOperator::Arithmetic(
                    ArithmeticOperator::Plus,
                ))))
            }
            '-' => {
                chars.next();
                Ok(Some(Token::BinaryOperator(BinaryOperator::Arithmetic(
                    ArithmeticOperator::Minus,
                ))))
            }
            '*' => {
                chars.next();
                Ok(Some(Token::BinaryOperator(BinaryOperator::Arithmetic(
                    ArithmeticOperator::Multiply,
                ))))
            }
            '/' => {
                chars.next();
                Ok(Some(Token::BinaryOperator(BinaryOperator::Arithmetic(
                    ArithmeticOperator::Divide,
                ))))
            }
            '%' => {
                chars.next();
                Ok(Some(Token::BinaryOperator(BinaryOperator::Arithmetic(
                    ArithmeticOperator::Module,
                ))))
            }
            '&' => {
                chars.next();
                Ok(Some(Token::BinaryOperator(
                    BinaryOperator::BitwiseOperator(BitwiseOperator::And),
                )))
            }
            '|' => {
                chars.next();
                Ok(Some(Token::BinaryOperator(
                    BinaryOperator::BitwiseOperator(BitwiseOperator::Or),
                )))
            }
            '^' => {
                chars.next();
                Ok(Some(Token::BinaryOperator(
                    BinaryOperator::BitwiseOperator(BitwiseOperator::Xor),
                )))
            }
            '.' => {
                chars.next();
                Ok(Some(Token::BinaryOperator(BinaryOperator::Dot)))
            }
            'A'..='Z' | 'a'..='z' => Ok(Some(read_reserved_word(chars)?)),
            '=' | '!' | '<' | '>' => Ok(Some(read_symbol(chars)?)),
            '[' => {
                chars.next();
                Ok(Some(Token::ListStart))
            }
            ']' => {
                chars.next();
                Ok(Some(Token::ListEnds))
            }
            '{' => {
                chars.next();
                Ok(Some(Token::ClassStarts))
            }
            '}' => {
                chars.next();
                Ok(Some(Token::ClassEnds))
            }
            ':' => Ok(Some(read_field_access_or_definition(chars)?)),
            _ => Err(TokenError {
                cause: format!("Unknown character: {}", chr),
            }),
        }
    }
}

fn read_symbol(
    chars: &mut Peekable<impl Iterator<Item = (usize, char)>>,
) -> Result<Token, TokenError> {
    let mut str = String::new();
    loop {
        let Some((_, chr)) = chars.peek() else {
            break;
        };
        let chr = *chr;
        match chr {
            '=' | '!' | '<' | '>' => {
                str.push(chr);
                chars.next();
            }
            _ => break,
        }
    }
    match str.as_str() {
        "=" | "==" => Ok(Token::BinaryOperator(BinaryOperator::Comparison(
            ComparisonOperator::Eq,
        ))),
        "!=" | "<>" => Ok(Token::BinaryOperator(BinaryOperator::Comparison(
            ComparisonOperator::Neq,
        ))),
        "<" => Ok(Token::BinaryOperator(BinaryOperator::Comparison(
            ComparisonOperator::SmallerThen,
        ))),
        "<=" => Ok(Token::BinaryOperator(BinaryOperator::Comparison(
            ComparisonOperator::SmallerThenEq,
        ))),
        ">" => Ok(Token::BinaryOperator(BinaryOperator::Comparison(
            ComparisonOperator::LargerThen,
        ))),
        ">=" => Ok(Token::BinaryOperator(BinaryOperator::Comparison(
            ComparisonOperator::LargerThenEq,
        ))),
        _ => Err(TokenError {
            cause: format!("Unknown comparison symbol: {str}"),
        }),
    }
}

fn read_reserved_word(
    chars: &mut Peekable<impl Iterator<Item = (usize, char)>>,
) -> Result<Token, TokenError> {
    let mut str = String::new();
    loop {
        let Some((_, chr)) = chars.peek() else {
            break;
        };
        let chr = *chr;
        if chr.is_ascii_alphabetic() || chr == '_' {
            chars.next();
            str.push(chr.to_ascii_uppercase());
        } else {
            break;
        }
    }
    match str.as_str() {
        "FALSE" => Ok(Token::Value(Value::Bool(false))),
        "TRUE" => Ok(Token::Value(Value::Bool(true))),
        "NOT" => Ok(Token::Not),
        "AND" => Ok(Token::BinaryOperator(BinaryOperator::Logical(
            LogicalOperator::And,
        ))),
        "OR" => Ok(Token::BinaryOperator(BinaryOperator::Logical(
            LogicalOperator::Or,
        ))),
        "XOR" => Ok(Token::BinaryOperator(BinaryOperator::Logical(
            LogicalOperator::Xor,
        ))),
        "IS" => Ok(Token::Is),
        "SOME" => Ok(Token::Some),
        "NONE" => Ok(Token::None),
        "FILE" => Ok(Token::File),
        "DIR" => Ok(Token::Dir),
        "LINK" => Ok(Token::Link),
        "OF" => Ok(Token::BinaryOperator(BinaryOperator::Of)),
        "MATCHES" => Ok(Token::BinaryOperator(BinaryOperator::Matches)),
        "IF" => Ok(Token::If),
        "THEN" => Ok(Token::Then),
        "ELSE" => Ok(Token::Else),
        "CASE" => Ok(Token::Case),
        "WHEN" => Ok(Token::When),
        "END" => Ok(Token::End),
        "BETWEEN" => Ok(Token::Between),
        "FORMAT" | "FORMATDATE" => Ok(Token::Format),
        "FOR" => Ok(Token::For),
        "FROM" => Ok(Token::From),
        "PARSE" | "PARSEDATE" => Ok(Token::Parse),
        "SPAWN" | "FIRE" => Ok(Token::Spawn),
        "EXECUTE" | "EXEC" => Ok(Token::Execute),
        "INTO" => Ok(Token::Into),
        "ASC" => Ok(Token::Asc),
        "DESC" => Ok(Token::Desc),
        "AS" => Ok(Token::As),
        "WITH" => Ok(Token::With),
        "DO" => Ok(Token::Do),
        "DATE" | "TIME" | "TIMESTAMP" => Ok(Token::Date),
        "BOOL" | "BOOLEAN" => Ok(Token::Boolean),
        "STRING" | "TEXT" | "STR" => Ok(Token::String),
        "REPLACE" => Ok(Token::Replace),
        "TO" => Ok(Token::To),
        "PATTERN" => Ok(Token::Pattern),
        "NUMBER" | "NUM" | "INT" | "INTEGER" => Ok(Token::Number),
        _ => {
            if let Some(access) = Access::from_str(&str) {
                Ok(Token::SimpleAccess(access))
            } else if let Some(f) = FunctionName::from_str(&str) {
                Ok(Token::FunctionName(f))
            } else if let Some(n) = MethodName::from_str(&str) {
                Ok(Token::MethodName(n))
            } else {
                Err(TokenError {
                    cause: format!("Unknown reserved word: {str}"),
                })
            }
        }
    }
}

fn read_field_access_or_definition(
    chars: &mut Peekable<impl Iterator<Item = (usize, char)>>,
) -> Result<Token, TokenError> {
    // eat the open :
    chars.next();
    let access = if let Some((_, ':')) = chars.peek() {
        chars.next();
        true
    } else {
        false
    };
    let mut str = String::new();
    loop {
        let chr = chars.peek();
        match chr {
            Some((_, ch)) if ch.is_alphanumeric() => {
                str.push(*ch);
                chars.next();
            }
            _ => break,
        };
    }
    if str.is_empty() {
        return Err(TokenError {
            cause: "Empty Field name".into(),
        });
    }
    if access {
        Ok(Token::ClassFieldAccess(str))
    } else {
        Ok(Token::ClassFieldName(str))
    }
}

fn read_path_or_file(
    chars: &mut Peekable<impl Iterator<Item = (usize, char)>>,
) -> Result<Token, TokenError> {
    // eat the open @
    chars.next();

    if let Some((_, '(')) = chars.peek() {
        let date = read_date(chars)?;
        Ok(Token::Value(Value::Date(date)))
    } else {
        let path = read_path(chars)?;
        Ok(Token::Value(Value::Path(path)))
    }
}

fn read_path(
    chars: &mut Peekable<impl Iterator<Item = (usize, char)>>,
) -> Result<PathBuf, TokenError> {
    if let Some((_, '"')) = chars.peek() {
        return read_quoted_path(chars);
    };
    let mut str = String::new();
    loop {
        let chr = chars.peek();
        match chr {
            None => break,
            Some((_, c)) if "[](){}\":|,@".contains(*c) || c.is_whitespace() => break,
            Some((_, ch)) => {
                str.push(*ch);
                chars.next();
            }
        };
    }
    Ok(PathBuf::from(&str))
}

fn read_quoted_path(
    chars: &mut impl Iterator<Item = (usize, char)>,
) -> Result<PathBuf, TokenError> {
    // eat the open quote
    chars.next();
    let mut str = String::new();

    loop {
        let chr = chars.next();
        match chr {
            None => {
                return Err(TokenError {
                    cause: "Unended path".into(),
                });
            }
            Some((_, '\"')) => break,
            Some((_, ch)) => str.push(ch),
        };
    }
    Ok(PathBuf::from(&str))
}

fn read_date(
    chars: &mut impl Iterator<Item = (usize, char)>,
) -> Result<DateTime<Local>, TokenError> {
    // eat the brackets
    chars.next();
    let mut str = String::new();
    loop {
        let Some((_, chr)) = chars.next() else {
            return Err(TokenError {
                cause: "Unended date".into(),
            });
        };
        match chr {
            ')' => break,
            _ => str.push(chr),
        }
    }
    parse_date(&str)
}

fn parse_date(val: &str) -> Result<DateTime<Local>, TokenError> {
    if let Ok(date) = DateTime::parse_from_rfc3339(val) {
        return Ok(date.into());
    }

    let naive_date_formats = ["%d/%b/%Y", "%Y-%m-%d"];
    for format in naive_date_formats {
        if let Ok(date) = NaiveDate::parse_from_str(val, format)
            && let LocalResult::Single(date) =
                date.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Local)
        {
            return Ok(date);
        }
    }

    let naive_date_formats = [
        "%d/%b/%Y %H:%M",
        "%d/%b/%Y %H:%M:%S",
        "%d/%b/%Y %H:%M:%S%.f",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
    ];

    for format in naive_date_formats {
        if let Ok(date) = NaiveDateTime::parse_from_str(val, format)
            && let LocalResult::Single(date) = date.and_local_timezone(Local)
        {
            return Ok(date);
        }
    }

    let naive_date_formats_with_tz = [
        "%d/%b/%Y %H:%M %z",
        "%d/%b/%Y %H:%M:%S %z",
        "%d/%b/%Y %H:%M:%S%.f %z",
        "%Y-%m-%d %H:%M %z",
        "%Y-%m-%d %H:%M:%S %z",
        "%Y-%m-%d %H:%M:%S%.f %z",
    ];

    for format in naive_date_formats_with_tz {
        if let Ok(date) = DateTime::parse_from_str(val, format) {
            return Ok(date.into());
        }
    }
    Err(TokenError {
        cause: format!("Invalid date: {}, try using RFC-3339", val),
    })
}

fn read_string(chars: &mut impl Iterator<Item = (usize, char)>) -> Result<String, TokenError> {
    // eat the double quote
    chars.next();
    let mut str = String::new();
    loop {
        let Some((_, chr)) = chars.next() else {
            return Err(TokenError {
                cause: "Unended string".into(),
            });
        };
        match chr {
            '"' => break,
            '\\' => str.push(read_escape(chars)?),
            _ => str.push(chr),
        }
    }
    Ok(str)
}
fn read_escape(chars: &mut impl Iterator<Item = (usize, char)>) -> Result<char, TokenError> {
    let Some((_, chr)) = chars.next() else {
        return Err(TokenError {
            cause: "Unended escape sequence".into(),
        });
    };
    match chr {
        'n' => Ok('\n'),
        'r' => Ok('\r'),
        't' => Ok('\t'),
        'u' => read_hex_char(chars),
        _ => Ok(chr),
    }
}
fn read_hex_char(chars: &mut impl Iterator<Item = (usize, char)>) -> Result<char, TokenError> {
    let mut num = 0;
    for _ in 0..4 {
        let Some((_, chr)) = chars.next() else {
            return Err(TokenError {
                cause: "Unended unicode number".into(),
            });
        };
        let Some(digit) = chr.to_digit(16) else {
            return Err(TokenError {
                cause: format!("not a valid HEX digit: '{}'", chr),
            });
        };
        num = num * 16 + digit;
    }
    let Some(chr) = char::from_u32(num) else {
        return Err(TokenError {
            cause: format!("not a valid unicode character: '{:#x}'", num),
        });
    };
    Ok(chr)
}

fn read_number(chars: &mut Peekable<impl Iterator<Item = (usize, char)>>) -> u64 {
    let mut number = 0;
    let mut index = 0;
    loop {
        let Some((_, char)) = chars.peek() else {
            break;
        };
        let Some(digit) = char.to_digit(10) else {
            if index == 1 && number == 0 {
                let radix = match char.to_ascii_uppercase() {
                    'X' => Some(16),
                    'O' => Some(8),
                    'B' => Some(2),
                    _ => None,
                };
                if let Some(radix) = radix {
                    chars.next();
                    return read_number_with_radix(chars, radix);
                }
            }
            break;
        };
        chars.next();
        index += 1;
        number = number * 10 + (digit as u64);
    }
    number
}

fn read_number_with_radix(
    chars: &mut Peekable<impl Iterator<Item = (usize, char)>>,
    radix: u32,
) -> u64 {
    let mut number = 0;
    loop {
        let Some((_, char)) = chars.peek() else {
            break;
        };
        let Some(digit) = char.to_digit(radix) else {
            break;
        };
        chars.next();
        number = number * (radix as u64) + (digit as u64);
    }
    number
}

fn read_binding_name(
    chars: &mut Peekable<impl Iterator<Item = (usize, char)>>,
) -> Result<String, TokenError> {
    // eat the dollar
    chars.next();
    let mut str = String::new();
    loop {
        let chr = chars.peek();
        match chr {
            Some((_, c)) if c.is_ascii_alphanumeric() || "_-".contains(*c) => {
                str.push(*c);
                chars.next();
            }
            _ => break,
        };
    }
    if str.is_empty() {
        return Err(TokenError {
            cause: "Empty Binding".into(),
        });
    }
    Ok(str)
}

#[cfg(test)]
mod tests {
    use chrono::{FixedOffset, MappedLocalTime, NaiveTime, TimeZone, Utc};

    use crate::parser::ast::function_name::EnvFunctionName;

    use super::*;

    #[test]
    fn read_simple_number() -> Result<(), TokenError> {
        let str = "3211";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(token, Some(Token::Value(Value::Number(3211))));

        Ok(())
    }

    #[test]
    fn read_number_with_x_in_the_middle() -> Result<(), TokenError> {
        let str = "32x11";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(token, Some(Token::Value(Value::Number(32))));

        Ok(())
    }

    #[test]
    fn read_hex_number() -> Result<(), TokenError> {
        let str = "0x11Aa";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(token, Some(Token::Value(Value::Number(0x11aa))));

        Ok(())
    }

    #[test]
    fn read_oct_number() -> Result<(), TokenError> {
        let str = "0o452327";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(token, Some(Token::Value(Value::Number(0o452327))));

        Ok(())
    }

    #[test]
    fn read_unknown_radix() -> Result<(), TokenError> {
        let str = "0q123";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(token, Some(Token::Value(Value::Number(0))));

        Ok(())
    }

    #[test]
    fn read_binary_number() -> Result<(), TokenError> {
        let str = "0b110011";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(token, Some(Token::Value(Value::Number(0b110011))));

        Ok(())
    }

    #[test]
    fn read_binary_number_with_invalid_digit() -> Result<(), TokenError> {
        let str = "0b11004";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(token, Some(Token::Value(Value::Number(0b1100))));

        Ok(())
    }

    #[test]
    fn read_simple_text() -> Result<(), TokenError> {
        let str = "\"test\"";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(token, Some(Token::Value(Value::String("test".into()))));

        Ok(())
    }

    #[test]
    fn read_text_with_escape_quotes() -> Result<(), TokenError> {
        let str = "\"test \\\"this\"";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(
            token,
            Some(Token::Value(Value::String("test \"this".into())))
        );

        Ok(())
    }

    #[test]
    fn read_text_with_escape_newlines() -> Result<(), TokenError> {
        let str = "\"test \\nthis\"";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(
            token,
            Some(Token::Value(Value::String("test \nthis".into())))
        );

        Ok(())
    }

    #[test]
    fn read_text_with_escape_slash() -> Result<(), TokenError> {
        let str = "\"test \\\\this\"";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(
            token,
            Some(Token::Value(Value::String("test \\this".into())))
        );

        Ok(())
    }

    #[test]
    fn read_text_with_escape_tab() -> Result<(), TokenError> {
        let str = "\"test \\tthis\"";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(
            token,
            Some(Token::Value(Value::String("test \tthis".into())))
        );

        Ok(())
    }

    #[test]
    fn read_text_with_escape_cr() -> Result<(), TokenError> {
        let str = "\"test \\rthis\"";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(
            token,
            Some(Token::Value(Value::String("test \rthis".into())))
        );

        Ok(())
    }

    #[test]
    fn read_empty_text() -> Result<(), TokenError> {
        let str = "\"\"";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(token, Some(Token::Value(Value::String("".into()))));

        Ok(())
    }

    #[test]
    fn read_unicode() -> Result<(), TokenError> {
        let str = "\"A\\u03B1B\"";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(token, Some(Token::Value(Value::String("AαB".into()))));

        Ok(())
    }

    #[test]
    fn read_invalid_unicode_character() {
        let str = "\"\\ud83d\"";
        let mut chars = str.chars().enumerate().peekable();

        let err = Token::new(&mut chars).err();

        assert!(err.is_some());
    }

    #[test]
    fn read_invalid_character_in_unicode() {
        let str = "\"\\u03BT\"";
        let mut chars = str.chars().enumerate().peekable();

        let err = Token::new(&mut chars).err();

        assert!(err.is_some());
    }

    #[test]
    fn read_too_short_unicode() {
        let str = "\"\\u03B";
        let mut chars = str.chars().enumerate().peekable();

        let err = Token::new(&mut chars).err();

        assert!(err.is_some());
    }

    #[test]
    fn read_never_ending_string() {
        let str = "\"tests";
        let mut chars = str.chars().enumerate().peekable();

        let err = Token::new(&mut chars).err();

        assert!(err.is_some());
    }

    #[test]
    fn read_never_escape() {
        let str = "\"\\";
        let mut chars = str.chars().enumerate().peekable();

        let err = Token::new(&mut chars).err();

        assert!(err.is_some());
    }

    fn date_literal_tz(
        date_as_text: &str,
        expected_date: NaiveDate,
        expected_time: NaiveTime,
        time_zone: impl TimeZone,
    ) -> Result<(), TokenError> {
        let MappedLocalTime::Single(expected_date) = expected_date
            .and_time(expected_time)
            .and_local_timezone(time_zone)
        else {
            panic!("Invalid date");
        };
        let expected_date = expected_date.with_timezone(&Local);
        let str = format!("@({date_as_text})");

        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(token, Some(Token::Value(Value::Date(expected_date))));

        Ok(())
    }

    fn date_literal(
        date_as_text: &str,
        expected_date: NaiveDate,
        expected_time: NaiveTime,
    ) -> Result<(), TokenError> {
        date_literal_tz(date_as_text, expected_date, expected_time, Local)
    }

    #[test]
    fn date_literal_with_slash() -> Result<(), TokenError> {
        date_literal(
            "20/Jan/2025",
            NaiveDate::from_ymd_opt(2025, 1, 20).unwrap(),
            NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        )
    }

    #[test]
    fn date_literal_with_dash() -> Result<(), TokenError> {
        date_literal(
            "2025-03-17",
            NaiveDate::from_ymd_opt(2025, 3, 17).unwrap(),
            NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        )
    }

    #[test]
    fn date_literal_with_slash_hm() -> Result<(), TokenError> {
        date_literal(
            "20/Jan/2025 11:43",
            NaiveDate::from_ymd_opt(2025, 1, 20).unwrap(),
            NaiveTime::from_hms_opt(11, 43, 0).unwrap(),
        )
    }

    #[test]
    fn date_literal_with_dash_hm() -> Result<(), TokenError> {
        date_literal(
            "2025-03-17 15:21",
            NaiveDate::from_ymd_opt(2025, 3, 17).unwrap(),
            NaiveTime::from_hms_opt(15, 21, 0).unwrap(),
        )
    }

    #[test]
    fn date_literal_with_slash_hms() -> Result<(), TokenError> {
        date_literal(
            "20/Jan/2025 11:43:14",
            NaiveDate::from_ymd_opt(2025, 1, 20).unwrap(),
            NaiveTime::from_hms_opt(11, 43, 14).unwrap(),
        )
    }

    #[test]
    fn date_literal_with_dash_hms() -> Result<(), TokenError> {
        date_literal(
            "2025-03-17 15:21:54",
            NaiveDate::from_ymd_opt(2025, 3, 17).unwrap(),
            NaiveTime::from_hms_opt(15, 21, 54).unwrap(),
        )
    }

    #[test]
    fn date_literal_with_dash_hmsms() -> Result<(), TokenError> {
        date_literal(
            "2025-03-17 15:21:54.3",
            NaiveDate::from_ymd_opt(2025, 3, 17).unwrap(),
            NaiveTime::from_hms_milli_opt(15, 21, 54, 300).unwrap(),
        )
    }

    #[test]
    fn date_literal_with_slash_hmsms() -> Result<(), TokenError> {
        date_literal(
            "11/Aug/1976 17:45:00.421",
            NaiveDate::from_ymd_opt(1976, 8, 11).unwrap(),
            NaiveTime::from_hms_milli_opt(17, 45, 0, 421).unwrap(),
        )
    }

    #[test]
    fn date_literal_with_slash_hmtz() -> Result<(), TokenError> {
        let offset = FixedOffset::east_opt(4 * 3600).unwrap();

        date_literal_tz(
            "21/Nov/2031 12:21 +0400",
            NaiveDate::from_ymd_opt(2031, 11, 21).unwrap(),
            NaiveTime::from_hms_opt(12, 21, 0).unwrap(),
            offset,
        )
    }

    #[test]
    fn date_literal_with_slash_hmstz() -> Result<(), TokenError> {
        let offset = FixedOffset::west_opt(5 * 3600).unwrap();

        date_literal_tz(
            "12/May/1986 14:31:12 -0500",
            NaiveDate::from_ymd_opt(1986, 5, 12).unwrap(),
            NaiveTime::from_hms_opt(14, 31, 12).unwrap(),
            offset,
        )
    }

    #[test]
    fn date_literal_with_slash_hmsmtz() -> Result<(), TokenError> {
        let offset = FixedOffset::west_opt(3600).unwrap();

        date_literal_tz(
            "12/Feb/2025 14:31:12.40 -0100",
            NaiveDate::from_ymd_opt(2025, 2, 12).unwrap(),
            NaiveTime::from_hms_milli_opt(14, 31, 12, 400).unwrap(),
            offset,
        )
    }

    #[test]
    fn date_literal_with_dash_hmtz() -> Result<(), TokenError> {
        let offset = FixedOffset::west_opt(3 * 3600).unwrap();

        date_literal_tz(
            "2024-10-09 16:12 -0300",
            NaiveDate::from_ymd_opt(2024, 10, 9).unwrap(),
            NaiveTime::from_hms_opt(16, 12, 0).unwrap(),
            offset,
        )
    }

    #[test]
    fn date_literal_with_dash_hmstz() -> Result<(), TokenError> {
        let offset = FixedOffset::east_opt(5 * 3600).unwrap();

        date_literal_tz(
            "1986-5-12 14:31:12 +0500",
            NaiveDate::from_ymd_opt(1986, 5, 12).unwrap(),
            NaiveTime::from_hms_opt(14, 31, 12).unwrap(),
            offset,
        )
    }

    #[test]
    fn date_literal_with_dash_hmsmtz() -> Result<(), TokenError> {
        let offset = FixedOffset::east_opt(3600).unwrap();

        date_literal_tz(
            "2025-02-12 14:31:12.40 +0100",
            NaiveDate::from_ymd_opt(2025, 2, 12).unwrap(),
            NaiveTime::from_hms_milli_opt(14, 31, 12, 400).unwrap(),
            offset,
        )
    }

    #[test]
    fn date_literal_with_rfc3339() -> Result<(), TokenError> {
        date_literal_tz(
            "1985-04-12T23:20:50.52Z",
            NaiveDate::from_ymd_opt(1985, 4, 12).unwrap(),
            NaiveTime::from_hms_milli_opt(23, 20, 50, 520).unwrap(),
            Utc,
        )
    }

    #[test]
    fn invalid_date_format() {
        let str = "@(2024-71-41)".to_string();

        let mut chars = str.chars().enumerate().peekable();

        let err = Token::new(&mut chars).err();

        assert!(err.is_some());
    }

    #[test]
    fn never_ending_date() {
        let str = "@(2025-02-12 14:31:12.40 +0100".to_string();

        let mut chars = str.chars().enumerate().peekable();

        let err = Token::new(&mut chars).err();

        assert!(err.is_some());
    }

    #[test]
    fn read_path_simple() -> Result<(), TokenError> {
        let str = "@\\home\\user\\";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(
            token,
            Some(Token::Value(Value::Path(PathBuf::from("\\home\\user\\"))))
        );

        Ok(())
    }

    #[test]
    fn read_path_ends_with_ws() -> Result<(), TokenError> {
        let str = "@\\home\\user\\   ";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(
            token,
            Some(Token::Value(Value::Path(PathBuf::from("\\home\\user\\"))))
        );

        Ok(())
    }

    #[test]
    fn read_path_ends_with_comma() -> Result<(), TokenError> {
        let str = "@\\home\\user\\,   ";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(
            token,
            Some(Token::Value(Value::Path(PathBuf::from("\\home\\user\\"))))
        );

        Ok(())
    }

    #[test]
    fn read_path_quote() -> Result<(), TokenError> {
        let str = "@\"\\home\\user\\My Files\"";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(
            token,
            Some(Token::Value(Value::Path(PathBuf::from(
                "\\home\\user\\My Files"
            ))))
        );

        Ok(())
    }

    #[test]
    fn never_ending_path() {
        let str = "@\"home".to_string();

        let mut chars = str.chars().enumerate().peekable();

        let err = Token::new(&mut chars).err();

        assert!(err.is_some());
    }

    #[test]
    fn open_brackets() -> Result<(), TokenError> {
        let str = "(";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(token, Some(Token::OpenBrackets));

        Ok(())
    }

    #[test]
    fn close_brackets() -> Result<(), TokenError> {
        let str = ")";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(token, Some(Token::CloseBrackets));

        Ok(())
    }

    #[test]
    fn comma() -> Result<(), TokenError> {
        let str = ",";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(token, Some(Token::Comma));

        Ok(())
    }

    #[test]
    fn bool_true() -> Result<(), TokenError> {
        let str = "True";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(token, Some(Token::Value(Value::Bool(true))));

        Ok(())
    }

    #[test]
    fn bool_false() -> Result<(), TokenError> {
        let str = "false";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(token, Some(Token::Value(Value::Bool(false))));

        Ok(())
    }

    #[test]
    fn unexpected_character() {
        let str = "?".to_string();

        let mut chars = str.chars().enumerate().peekable();

        let err = Token::new(&mut chars).err();

        assert!(err.is_some());
    }

    #[test]
    fn unknown_reserved_word() {
        let str = "notaword".to_string();

        let mut chars = str.chars().enumerate().peekable();

        let err = Token::new(&mut chars).err();

        assert!(err.is_some());
    }

    #[test]
    fn function_name() -> Result<(), TokenError> {
        let str = "rand".to_string();

        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(
            token,
            Some(Token::FunctionName(FunctionName::Env(
                EnvFunctionName::Rand
            )))
        );

        Ok(())
    }

    #[test]
    fn bad_symbols() {
        let str = "<<".to_string();

        let mut chars = str.chars().enumerate().peekable();

        let err = Token::new(&mut chars).err();

        assert!(err.is_some());
    }

    #[test]
    fn eq_one() -> Result<(), TokenError> {
        let str = "=";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(
            token,
            Some(Token::BinaryOperator(BinaryOperator::Comparison(
                ComparisonOperator::Eq
            )))
        );

        Ok(())
    }

    #[test]
    fn eq_two() -> Result<(), TokenError> {
        let str = "=";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(
            token,
            Some(Token::BinaryOperator(BinaryOperator::Comparison(
                ComparisonOperator::Eq
            )))
        );

        Ok(())
    }

    #[test]
    fn neq_one() -> Result<(), TokenError> {
        let str = "!=";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(
            token,
            Some(Token::BinaryOperator(BinaryOperator::Comparison(
                ComparisonOperator::Neq
            )))
        );

        Ok(())
    }

    #[test]
    fn neq_two() -> Result<(), TokenError> {
        let str = "<>";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(
            token,
            Some(Token::BinaryOperator(BinaryOperator::Comparison(
                ComparisonOperator::Neq
            )))
        );

        Ok(())
    }

    #[test]
    fn bw_and() -> Result<(), TokenError> {
        let str = "&";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(
            token,
            Some(Token::BinaryOperator(BinaryOperator::BitwiseOperator(
                BitwiseOperator::And
            )))
        );

        Ok(())
    }

    #[test]
    fn bw_or() -> Result<(), TokenError> {
        let str = "|";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(
            token,
            Some(Token::BinaryOperator(BinaryOperator::BitwiseOperator(
                BitwiseOperator::Or
            )))
        );

        Ok(())
    }

    #[test]
    fn bw_xor() -> Result<(), TokenError> {
        let str = "^";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(
            token,
            Some(Token::BinaryOperator(BinaryOperator::BitwiseOperator(
                BitwiseOperator::Xor
            )))
        );

        Ok(())
    }

    #[test]
    fn binding_name() -> Result<(), TokenError> {
        let str = "$test-this";
        let mut chars = str.chars().enumerate().peekable();

        let token = Token::new(&mut chars)?;

        assert_eq!(token, Some(Token::BindingName("test-this".into())));

        Ok(())
    }

    #[test]
    fn binding_name_empty() -> Result<(), TokenError> {
        let str = "$,";
        let mut chars = str.chars().enumerate().peekable();

        let err = Token::new(&mut chars).err();

        assert!(err.is_some());

        Ok(())
    }

    #[test]
    fn complex_just_started() -> Result<(), TokenError> {
        let str = ":";
        let mut chars = str.chars().enumerate().peekable();

        let err = Token::new(&mut chars).err();

        assert!(err.is_some());

        Ok(())
    }

    #[test]
    fn complex_unknown_char() -> Result<(), TokenError> {
        let str = ":-";
        let mut chars = str.chars().enumerate().peekable();

        let err = Token::new(&mut chars).err();

        assert!(err.is_some());

        Ok(())
    }
}
