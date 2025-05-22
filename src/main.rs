use std::collections::BTreeMap;
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
    Object(BTreeMap<String, Value>),
}

impl Value {
    // TODO: maybe move the format implementation to another struct
    pub fn format(&self) -> String {
        let mut buf = String::new();
        self.format_in(&mut buf);
        buf
    }

    fn format_in(&self, buf: &mut String) {
        match self {
            Self::Null => buf.push_str("null"),
            Self::Bool(true) => buf.push_str("true"),
            Self::Bool(false) => buf.push_str("false"),
            Self::String(s) => {
                buf.push('"');
                buf.push_str(s);
                buf.push('"');
            }
            Self::Number(n) => buf.push_str(&n.to_string()),
            Self::Array(arr) => {
                buf.push('[');
                for (idx, v) in arr.iter().enumerate() {
                    v.format_in(buf);
                    if idx != arr.len() - 1 {
                        buf.push_str(", ");
                    }
                }
                buf.push(']');
            }
            Self::Object(map) => {
                buf.push('{');
                for (idx, (key, value)) in map.iter().enumerate() {
                    buf.push('"');
                    buf.push_str(key);
                    buf.push_str(r#"": "#);
                    value.format_in(buf);
                    if idx != map.len() - 1 {
                        buf.push_str(", ");
                    }
                }
                buf.push('}');
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fmt = self.format();
        write!(f, "{fmt}")
    }
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

    // TODO: actually check if all ascii whitepace are valid whitespaces
    fn is_whitespace(&self, ch: char) -> bool {
        ch.is_ascii_whitespace()
    }

    fn next_pos(&mut self, ch: char) {
        if ch == '\n' {
            self.col = 1;
            self.line += 1;
        } else {
            self.col += 1;
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.src.peek().copied() {
            if self.is_whitespace(ch) {
                let space = self.src.next().unwrap();
                self.next_pos(space);
            } else {
                break;
            }
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
        let mut buf = String::new();
        if let Some('-') = self.src.peek().copied() {
            let minus = self.src.next().unwrap();
            self.next_pos(minus);
            buf.push(minus);
        }

        // TODO: add support for exponential format
        let Some(ch @ '0'..='9') = self.src.next() else {
            return Err(self.eof());
        };
        self.next_pos(ch);
        buf.push(ch);
        while let Some('0'..='9') = self.src.peek().copied() {
            let d = self.src.next().unwrap();
            self.next_pos(d);
            buf.push(d);
        }

        if let Some('.') = self.src.peek().copied() {
            let separator = self.src.next().expect("should be a decimal separator");
            self.next_pos(separator);
            buf.push(separator);
            let Some(ch @ '0'..='9') = self.src.next() else {
                return Err(self.eof());
            };
            self.next_pos(ch);
            buf.push(ch);
            while let Some('0'..='9') = self.src.peek().copied() {
                let d = self.src.next().unwrap();
                self.next_pos(d);
                buf.push(d);
            }
        }

        buf.parse::<f64>()
            .map(Value::Number)
            .map_err(|err| self.error(err.to_string()))
    }

    fn parse_string(&mut self) -> Result<Value, JsonParserError> {
        let quotes = self.src.next();
        assert_eq!(quotes, Some('"'));
        let quotes = quotes.unwrap();
        self.next_pos(quotes);

        let mut buf = String::new();
        loop {
            match self.src.next() {
                Some('"') => break,
                Some(ch) => buf.push(ch),
                None => return Err(self.eof()),
            }
        }

        Ok(Value::String(buf))
    }

    fn parse_array(&mut self) -> Result<Value, JsonParserError> {
        let bracket = self.src.next();
        assert_eq!(bracket, Some('['));
        let bracket = bracket.unwrap();
        self.next_pos(bracket);

        let mut values = Vec::<Value>::new();
        loop {
            match self.src.peek().copied() {
                Some(']') => {
                    let bracket = self.src.next().unwrap();
                    self.next_pos(bracket);
                    break;
                }
                Some(ch) if self.is_whitespace(ch) => {
                    let space = self.src.next().unwrap();
                    self.next_pos(space);
                    continue;
                }
                Some(_) => {
                    let value = self.parse()?;
                    values.push(value);
                    self.skip_whitespace();
                    let Some(ch) = self.src.next() else {
                        return Err(self.eof());
                    };
                    self.next_pos(ch);
                    match ch {
                        ',' => {}
                        ']' => break,
                        ch => {
                            let msg = format!(
                                "expected either array value separator ',' or end of array character ']', but received '{ch}'"
                            );
                            return Err(self.error(msg));
                        }
                    }
                }
                None => return Err(self.eof()),
            };
        }

        Ok(Value::Array(values))
    }

    // TODO: refactor this code, it actually looks like crap right now
    fn parse_object(&mut self) -> Result<Value, JsonParserError> {
        let brace = self.src.next();
        assert_eq!(brace, Some('{'));
        let brace = brace.unwrap();
        self.next_pos(brace);

        let mut values = BTreeMap::<String, Value>::new();
        loop {
            match self.src.peek().copied() {
                Some('}') => {
                    let brace = self.src.next().unwrap();
                    self.next_pos(brace);
                    break;
                }
                Some(ch) if self.is_whitespace(ch) => {
                    let space = self.src.next().unwrap();
                    self.next_pos(space);
                    continue;
                }
                Some(_) => {
                    let key = match self.parse()? {
                        Value::String(key) => key,
                        _ => {
                            // TODO: improve error message
                            let msg = "expected object key to be a string";
                            return Err(self.error(msg));
                        }
                    };
                    self.skip_whitespace();
                    let Some(ch) = self.src.next() else {
                        return Err(self.eof());
                    };
                    self.next_pos(ch);
                    if ch != ':' {
                        let msg = format!(
                            "expected character ':' after an object key but received '{ch}'"
                        );
                        return Err(self.error(msg));
                    }

                    self.skip_whitespace();
                    let value = self.parse()?;
                    values.insert(key, value);

                    self.skip_whitespace();
                    let Some(ch) = self.src.next() else {
                        return Err(self.eof());
                    };
                    self.next_pos(ch);
                    match ch {
                        '}' => break,
                        ',' => {}
                        ch => {
                            let msg = format!(
                                "expected either object key value separator ',' or end of character '}}', but received '{ch}'"
                            );
                            return Err(self.error(msg));
                        }
                    }
                }
                None => return Err(self.eof()),
            };
        }

        Ok(Value::Object(values))
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

    #[test]
    fn parse_array_works() {
        let src = r#"[1, 1.0, true, false, null, "name", "hironha", "123", ["nested_array"]]"#;
        let mut parser = JsonParser::new(src.chars());
        let parsed = parser.parse_array();
        assert!(parsed.is_ok(), "should be able to parse array");

        let array = parsed.unwrap();
        let Value::Array(arr) = array else {
            panic!("should have parsed an array");
        };
        let mut iter = arr.into_iter();
        assert_eq!(iter.next(), Some(Value::Number(1.0)));
        assert_eq!(iter.next(), Some(Value::Number(1.0)));
        assert_eq!(iter.next(), Some(Value::Bool(true)));
        assert_eq!(iter.next(), Some(Value::Bool(false)));
        assert_eq!(iter.next(), Some(Value::Null));
        assert_eq!(iter.next(), Some(Value::String(String::from("name"))));
        assert_eq!(iter.next(), Some(Value::String(String::from("hironha"))));
        assert_eq!(iter.next(), Some(Value::String(String::from("123"))));

        let Value::Array(nested) = iter.next().unwrap() else {
            panic!("should have parsed a nested array");
        };
        let mut nested_iter = nested.into_iter();
        assert_eq!(
            nested_iter.next(),
            Some(Value::String(String::from("nested_array")))
        );
    }

    #[test]
    fn parse_object_works() {
        let src = r#"{
            "name": "test",
            "wife": null,
            "age": 23,
            "happy": false,
            "weight": 56.50,
            "traits": ["male", "nerd"],
            "pets": {
                "name": "nina"
            }
        }"#
        .trim();
        let mut parser = JsonParser::new(src.chars());
        let parsed = parser.parse_object();
        if let Err(ref err) = parsed {
            println!("{err}");
        }
        assert!(parsed.is_ok(), "should be able to parse object");

        let Value::Object(map) = parsed.unwrap() else {
            panic!("should have parsed an object");
        };
        let name = map.get("name").unwrap().clone();
        assert_eq!(name, Value::String(String::from("test")));

        let wife = map.get("wife").unwrap().clone();
        assert_eq!(wife, Value::Null);

        let age = map.get("age").unwrap().clone();
        assert_eq!(age, Value::Number(23.0));

        let happy = map.get("happy").unwrap().clone();
        assert_eq!(happy, Value::Bool(false));

        let weight = map.get("weight").unwrap().clone();
        assert_eq!(weight, Value::Number(56.50));

        let Value::Array(traits) = map.get("traits").unwrap().clone() else {
            panic!("traits should be an array");
        };
        let mut traits = traits.into_iter();
        assert_eq!(traits.next().unwrap(), Value::String(String::from("male")));
        assert_eq!(traits.next().unwrap(), Value::String(String::from("nerd")));
        assert!(traits.next().is_none());

        let Value::Object(pets) = map.get("pets").unwrap().clone() else {
            panic!("pets should be an object");
        };
        let pet_name = pets.get("name").unwrap().clone();
        assert_eq!(pet_name, Value::String(String::from("nina")));
    }

    #[test]
    fn value_format_works() {
        let value = Value::Null;
        assert_eq!(value.format(), "null");

        let value = Value::Bool(true);
        assert_eq!(value.format(), "true");

        let value = Value::Bool(false);
        assert_eq!(value.format(), "false");

        let value = Value::String(String::from("test"));
        assert_eq!(value.format(), r#""test""#);

        let value = Value::Number(12.345);
        assert_eq!(value.format(), "12.345");

        let arr = vec![Value::Null, Value::Bool(false), Value::Number(1.23)];
        let value = Value::Array(arr);
        assert_eq!(value.format(), "[null, false, 1.23]");

        let mut map = BTreeMap::new();
        map.insert(String::from("alive"), Value::Bool(true));
        map.insert(String::from("times_cried"), Value::Number(123.0));
        map.insert(String::from("wife"), Value::Null);
        let value = Value::Object(map);
        assert_eq!(
            value.format(),
            r#"{"alive": true, "times_cried": 123, "wife": null}"#
        );
    }
}
