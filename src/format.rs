use super::Value;

use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Formatter {
    spacing: u8,
}

impl Formatter {
    // TODO: maybe rename to something more explicit, since this makes the formatter
    // format without any spaces
    pub fn new() -> Self {
        Self { spacing: 0 }
    }

    pub fn standard() -> Self {
        Self { spacing: 2 }
    }

    pub fn format(&self, value: &Value) -> String {
        let mut buf = String::new();
        self.format_in(&mut buf, value);
        buf
    }

    fn format_in(&self, buf: &mut String, value: &Value) {
        match value {
            Value::Null => buf.push_str("null"),
            Value::Bool(true) => buf.push_str("true"),
            Value::Bool(false) => buf.push_str("false"),
            Value::String(s) => self.format_str(buf, s),
            Value::Number(n) => buf.push_str(&n.to_string()),
            Value::Array(arr) => self.format_arr(buf, arr),
            Value::Object(map) => self.format_object(buf, map),
        }
    }

    fn format_str(&self, buf: &mut String, s: &str) {
        buf.push('"');
        buf.push_str(s);
        buf.push('"');
    }

    fn format_arr(&self, buf: &mut String, arr: &[Value]) {
        if self.spacing > 0 {
            self.format_arr_spaced(buf, arr);
        } else {
            self.format_arr_unspaced(buf, arr);
        }
    }

    fn format_arr_unspaced(&self, buf: &mut String, arr: &[Value]) {
        buf.push('[');

        for (idx, v) in arr.iter().enumerate() {
            self.format_in(buf, v);
            if idx != arr.len() - 1 {
                buf.push(',');
            }
        }

        buf.push(']');
    }

    fn format_arr_spaced(&self, buf: &mut String, arr: &[Value]) {
        buf.push('[');
        buf.push('\n');
        for _ in 0..self.spacing {
            buf.push(' ');
        }

        for (idx, v) in arr.iter().enumerate() {
            self.format_in(buf, v);
            if idx != arr.len() - 1 {
                buf.push(',');
                buf.push('\n');
                for _ in 0..self.spacing {
                    buf.push(' ');
                }
            }
        }

        buf.push(']');
    }

    fn format_object(&self, buf: &mut String, obj: &BTreeMap<String, Value>) {
        if self.spacing > 0 {
            self.format_object_spaced(buf, obj);
        } else {
            self.format_object_unspaced(buf, obj);
        }
    }

    fn format_object_unspaced(&self, buf: &mut String, obj: &BTreeMap<String, Value>) {
        buf.push('{');

        for (idx, (k, v)) in obj.iter().enumerate() {
            buf.push('"');
            buf.push_str(k);
            buf.push('"');
            buf.push(':');

            self.format_in(buf, v);
            if idx != obj.len() - 1 {
                buf.push(',');
            }
        }

        buf.push('}');
    }

    fn format_object_spaced(&self, buf: &mut String, obj: &BTreeMap<String, Value>) {
        buf.push_str("{\n");
        for _ in 0..self.spacing {
            buf.push(' ');
        }

        for (idx, (k, v)) in obj.iter().enumerate() {
            buf.push('"');
            buf.push_str(k);
            buf.push_str(r#"": "#);

            self.format_in(buf, v);
            if idx != obj.len() - 1 {
                buf.push_str(",\n");
                for _ in 0..self.spacing {
                    buf.push(' ');
                }
            }
        }

        buf.push_str("\n}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatter_without_spacing_works() {
        let formatter = Formatter::new();
        let value = Value::Null;
        assert_eq!(formatter.format(&value), "null");

        let value = Value::Bool(true);
        assert_eq!(formatter.format(&value), "true");

        let value = Value::Bool(false);
        assert_eq!(formatter.format(&value), "false");

        let value = Value::String(String::from("test"));
        assert_eq!(formatter.format(&value), r#""test""#);

        let value = Value::Number(12.345);
        assert_eq!(formatter.format(&value), "12.345");

        let arr = vec![Value::Null, Value::Bool(false), Value::Number(1.23)];
        let value = Value::Array(arr);
        assert_eq!(formatter.format(&value), "[null,false,1.23]");

        let mut map = BTreeMap::new();
        map.insert(String::from("alive"), Value::Bool(true));
        map.insert(String::from("times_cried"), Value::Number(123.0));
        map.insert(String::from("wife"), Value::Null);
        let value = Value::Object(map);
        assert_eq!(
            formatter.format(&value),
            r#"{"alive":true,"times_cried":123,"wife":null}"#
        );
    }

    #[test]
    fn formattter_with_spacing_works() {
        let formatter = Formatter::standard();
        let value = Value::Null;
        assert_eq!(formatter.format(&value), "null");

        let value = Value::Bool(true);
        assert_eq!(formatter.format(&value), "true");

        let value = Value::Bool(false);
        assert_eq!(formatter.format(&value), "false");

        let value = Value::String(String::from("test"));
        assert_eq!(formatter.format(&value), r#""test""#);

        let value = Value::Number(12.345);
        assert_eq!(formatter.format(&value), "12.345");

        let arr = vec![Value::Null, Value::Bool(false), Value::Number(1.23)];
        let value = Value::Array(arr);
        assert_eq!(formatter.format(&value), "[\n  null,\n  false,\n  1.23]");

        let mut map = BTreeMap::new();
        map.insert(String::from("alive"), Value::Bool(true));
        map.insert(String::from("times_cried"), Value::Number(123.0));
        map.insert(String::from("wife"), Value::Null);
        let value = Value::Object(map);
        assert_eq!(
            formatter.format(&value),
            "{\n  \"alive\": true,\n  \"times_cried\": 123,\n  \"wife\": null\n}"
        );
    }
}
