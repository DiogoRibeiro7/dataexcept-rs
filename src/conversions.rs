//! Conversions from common Rust and serde errors into [`DataError`].

use std::{
    io,
    num::{ParseFloatError, ParseIntError},
    str::Utf8Error,
    string::FromUtf8Error,
};

use serde_json::json;

use crate::{DataError, FailureMetadata};

fn base_error(code: &str, module: &str, message: String) -> DataError {
    DataError::new(code, message)
        .with_module(module)
        .with_failure(FailureMetadata::unknown())
}

impl From<io::Error> for DataError {
    fn from(value: io::Error) -> Self {
        let kind = format!("{:?}", value.kind());
        let raw_os_error = value.raw_os_error();
        let mut error =
            base_error("IoError", "std::io", value.to_string()).with_attribute("kind", json!(kind));

        if let Some(raw_os_error) = raw_os_error {
            error = error.with_attribute("raw_os_error", json!(raw_os_error));
        }

        error
    }
}

impl From<serde_json::Error> for DataError {
    fn from(value: serde_json::Error) -> Self {
        let category = match value.classify() {
            serde_json::error::Category::Io => "io",
            serde_json::error::Category::Syntax => "syntax",
            serde_json::error::Category::Data => "data",
            serde_json::error::Category::Eof => "eof",
        };
        let line = value.line();
        let column = value.column();

        base_error("JsonError", "serde_json", value.to_string())
            .with_attribute("category", json!(category))
            .with_attribute("line", json!(line))
            .with_attribute("column", json!(column))
    }
}

impl From<ParseIntError> for DataError {
    fn from(value: ParseIntError) -> Self {
        let kind = format!("{:?}", value.kind());

        base_error("ParseIntError", "std::num", value.to_string())
            .with_attribute("kind", json!(kind))
    }
}

impl From<ParseFloatError> for DataError {
    fn from(value: ParseFloatError) -> Self {
        base_error("ParseFloatError", "std::num", value.to_string())
    }
}

impl From<Utf8Error> for DataError {
    fn from(value: Utf8Error) -> Self {
        let valid_up_to = value.valid_up_to();
        let error_len = value.error_len();
        let mut error = base_error("Utf8Error", "std::str", value.to_string())
            .with_attribute("valid_up_to", json!(valid_up_to));

        if let Some(error_len) = error_len {
            error = error.with_attribute("error_len", json!(error_len));
        }

        error
    }
}

impl From<FromUtf8Error> for DataError {
    fn from(value: FromUtf8Error) -> Self {
        let utf8_error = value.utf8_error();
        let valid_up_to = utf8_error.valid_up_to();
        let error_len = utf8_error.error_len();
        let mut error = base_error("FromUtf8Error", "std::string", value.to_string())
            .with_attribute("valid_up_to", json!(valid_up_to));

        if let Some(error_len) = error_len {
            error = error.with_attribute("error_len", json!(error_len));
        }

        error
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use serde_json::Value;

    use crate::DataError;

    #[test]
    fn io_error_preserves_kind_and_os_error_when_available() {
        let source = io::Error::from_raw_os_error(2);
        let envelope = DataError::from(source).to_envelope();
        let attributes = envelope.attributes.expect("attributes should exist");

        assert_eq!(envelope.error_type, "IoError");
        assert_eq!(envelope.module, "std::io");
        assert_eq!(attributes["raw_os_error"], 2);
    }

    #[test]
    fn json_error_preserves_location_and_category() {
        let source = serde_json::from_str::<Value>("{")
            .expect_err("truncated JSON must fail to parse");
        let envelope = DataError::from(source).to_envelope();
        let attributes = envelope.attributes.expect("attributes should exist");

        assert_eq!(envelope.error_type, "JsonError");
        assert_eq!(attributes["category"], "eof");
        assert!(attributes["line"].as_u64().is_some());
        assert!(attributes["column"].as_u64().is_some());
    }

    #[test]
    fn parse_int_error_preserves_kind() {
        let source = "not-an-int"
            .parse::<i64>()
            .expect_err("non-numeric input must fail integer parsing");
        let envelope = DataError::from(source).to_envelope();
        let attributes = envelope.attributes.expect("attributes should exist");

        assert_eq!(envelope.error_type, "ParseIntError");
        assert_eq!(attributes["kind"], "InvalidDigit");
    }

    #[test]
    fn parse_float_error_has_stable_identity() {
        let source = "not-a-float"
            .parse::<f64>()
            .expect_err("non-numeric input must fail float parsing");
        let envelope = DataError::from(source).to_envelope();

        assert_eq!(envelope.error_type, "ParseFloatError");
        assert_eq!(envelope.module, "std::num");
    }
}
