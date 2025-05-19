use std::collections::HashMap;
use std::error;
use std::fmt;
use std::iter::Peekable;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

#[derive(Clone, Debug)]
pub struct JsonParserError {
    msg: String,
    col: u32,
    line: u32,
}

impl fmt::Display for JsonParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Parse json error at line {} column {}: {}",
            self.line, self.col, self.msg
        )
    }
}

impl error::Error for JsonParserError {}

pub struct JsonParser<T: Iterator<Item = char>> {
    src: Peekable<T>,
    col: u32,
    line: u32,
}

impl<T: Iterator<Item = char>> JsonParser<T> {
    pub fn new(src: T) -> Self {
        Self {
            src: src.peekable(),
            col: 1,
            line: 1,
        }
    }

    pub fn parse(&mut self) -> Result<Value, JsonParserError> {
        match self.src.peek().copied() {
            Some('t') => self.parse_true(),
            Some('f') => self.parse_false(),
            Some('n') => self.parse_null(),
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string(),
            Some(ch) if ch.is_ascii_digit() => self.parse_number(),
            Some(ch) => {
                let msg = format!("unexpected character '{ch}'");
                Err(self.error(msg))
            }
            None => Err(self.eof()),
        }
    }

    fn eof(&self) -> JsonParserError {
        JsonParserError {
            msg: String::from("unexpected end of line"),
            col: self.col,
            line: self.line,
        }
    }

    fn error(&self, msg: impl Into<String>) -> JsonParserError {
        JsonParserError {
            msg: msg.into(),
            col: self.col,
            line: self.line,
        }
    }

    fn is_whitespace(&self, ch: char) -> bool {
        ch == ' '
    }

    fn next_pos(&mut self, ch: char) {
        if ch == '\n' {
            self.col = 1;
            self.line += 1;
        } else {
            self.col += 1;
        }
    }

    fn read_word(&mut self, word: &str) -> Result<(), JsonParserError> {
        for w in word.chars() {
            let Some(ch) = self.src.next() else {
                return Err(self.eof());
            };
            self.next_pos(ch);
            if ch != w {
                let msg = format!("expected character '{w}' but received '{ch}'");
                return Err(self.error(msg));
            }
        }
        Ok(())
    }

    fn parse_null(&mut self) -> Result<Value, JsonParserError> {
        self.read_word("null")
            .map(|_| Value::Null)
            .map_err(|mut err| {
                err.msg.insert_str(0, "failed parsing null - ");
                err
            })
    }

    fn parse_true(&mut self) -> Result<Value, JsonParserError> {
        self.read_word("true")
            .map(|_| Value::Bool(true))
            .map_err(|mut err| {
                err.msg.insert_str(0, "failed parsing true - ");
                err
            })
    }

    fn parse_false(&mut self) -> Result<Value, JsonParserError> {
        self.read_word("false")
            .map(|_| Value::Bool(false))
            .map_err(|mut err| {
                err.msg.insert_str(0, "failed parsing false - ");
                err
            })
    }

    fn parse_number(&mut self) -> Result<Value, JsonParserError> {
        let first_digit = self.src.next();
        assert!(first_digit.as_ref().is_some_and(char::is_ascii_digit));
        let first_digit = first_digit.unwrap();
        self.next_pos(first_digit);

        let mut number = String::from(first_digit);
        while let Some(ch) = self.src.peek().copied() {
            match ch {
                '.' => break,
                ch if ch.is_ascii_digit() => {
                    number.push(ch);
                    let digit = self.src.next().expect("digit exists due peek");
                    self.next_pos(digit);
                }
                ch => {
                    let msg = format!("expected a digit but received '{ch}'");
                    return Err(self.error(msg));
                }
            }
        }

        if let Some('.') = self.src.peek().copied() {
            let decimal_separator = self.src.next().expect("should be a decimal separator");
            self.next_pos(decimal_separator);
            number.push(decimal_separator);
            while let Some(ch) = self.src.next() {
                self.next_pos(ch);
                match ch {
                    ch if ch.is_ascii_digit() => number.push(ch),
                    ch if self.is_whitespace(ch) => break,
                    ch => {
                        let msg = format!("expected a digit but received '{ch}'");
                        return Err(self.error(msg));
                    }
                }
            }
        }

        number
            .parse::<f64>()
            .map(Value::Number)
            .map_err(|err| self.error(err.to_string()))
    }

    fn parse_string(&mut self) -> Result<Value, JsonParserError> {
        let quotes = self.src.next();
        assert_eq!(quotes, Some('"'));
        let quotes = quotes.unwrap();
        self.next_pos(quotes);

        let mut closed = false;
        let mut buf = String::new();
        while let Some(ch) = self.src.next() {
            self.next_pos(ch);
            match ch {
                '"' => {
                    closed = true;
                    break;
                }
                ch => buf.push(ch),
            }
        }

        if !closed {
            return Err(self.eof());
        }

        Ok(Value::String(buf))
    }

    fn parse_array(&mut self) -> Result<Value, JsonParserError> {
        todo!()
    }

    fn parse_object(&mut self) -> Result<Value, JsonParserError> {
        todo!()
    }
}

fn main() {
    let mut parser = JsonParser::new("null".chars());
    println!("{:?}", parser.parse_null());

    let mut parser = JsonParser::new("true".chars());
    println!("{:?}", parser.parse_true());

    let mut parser = JsonParser::new("false".chars());
    println!("{:?}", parser.parse_false());

    let mut parser = JsonParser::new("123".chars());
    println!("{:?}", parser.parse_number());

    let mut parser = JsonParser::new("123.1".chars());
    println!("{:?}", parser.parse_number());

    let mut parser = JsonParser::new("123.123".chars());
    println!("{:?}", parser.parse_number());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_null_works() {
        let mut parser = JsonParser::new("null".chars());
        let parsed = parser.parse_null();
        assert!(parsed.is_ok(), "should be able to parse null");

        let value = parsed.unwrap();
        assert_eq!(value, Value::Null);
    }

    #[test]
    fn parse_true_works() {
        let mut parser = JsonParser::new("true".chars());
        let parsed = parser.parse_true();
        assert!(parsed.is_ok(), "should be able to parse true");

        let value = parsed.unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn parse_false_works() {
        let mut parser = JsonParser::new("false".chars());
        let parsed = parser.parse_false();
        assert!(parsed.is_ok(), "should be able to parse false");

        let value = parsed.unwrap();
        assert_eq!(value, Value::Bool(false));
    }

    #[test]
    fn parse_int_works() {
        let ints = [1, 2, 3, 4, 10, 123, 1234];
        for int in ints {
            let src = int.to_string();
            let mut parser = JsonParser::new(src.chars());
            let parsed = parser.parse_number();
            assert!(parsed.is_ok(), "should be able to parse int");

            let value = parsed.unwrap();
            assert_eq!(value, Value::Number(f64::from(int)));
        }
    }

    #[test]
    fn parse_float_works() {
        let floats = [1.0, 1.1, 1.2, 2.12, 1.123, 1.1234, 1234.1234];
        for float in floats {
            let src = float.to_string();
            let mut parser = JsonParser::new(src.chars());
            let parsed = parser.parse_number();
            assert!(parsed.is_ok(), "should be able to parse float");

            let value = parsed.unwrap();
            assert_eq!(value, Value::Number(float));
        }
    }

    #[test]
    fn parse_string_works() {
        let strs = [
            (r#""test""#, String::from("test")),
            (r#""hironha""#, String::from("hironha")),
            (r#""a""#, String::from("a")),
        ];
        for (src, out) in strs {
            let mut parser = JsonParser::new(src.chars());
            let parsed = parser.parse_string();
            assert!(parsed.is_ok(), "should be able to parse strign");

            let value = parsed.unwrap();
            assert_eq!(value, Value::String(out));
        }
    }
}
