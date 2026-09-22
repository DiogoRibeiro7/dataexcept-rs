//! Credential redaction compatible with the DataExcept wire contract.
//!
//! The helpers in this module preserve operational context such as URL scheme,
//! host, port, and optionally path while removing credentials from userinfo,
//! sensitive query parameters, sensitive fragment parameters, and free-form
//! text containing URLs.

use std::{collections::BTreeMap, sync::OnceLock};

use form_urlencoded::Serializer;
use regex::Regex;
use serde_json::Value;
use sha2::{Digest, Sha256};

/// Visible marker used in place of credentials.
pub const PLACEHOLDER: &str = "***";

/// Minimum secret length for safe substring replacement in free-form text.
pub const MIN_REMOVABLE_SECRET_LENGTH: usize = 8;

const SENSITIVE_PARAM_TOKENS: &[&str] = &[
    "apikey",
    "auth",
    "authorization",
    "bearer",
    "credential",
    "credentials",
    "hmac",
    "jwt",
    "key",
    "keys",
    "passphrase",
    "passwd",
    "password",
    "pwd",
    "sas",
    "secret",
    "secrets",
    "session",
    "sig",
    "signature",
    "token",
    "tokens",
];

fn url_regex() -> &'static Regex {
    static URL_REGEX: OnceLock<Regex> = OnceLock::new();
    URL_REGEX.get_or_init(|| {
        Regex::new(
            r#"[a-zA-Z][a-zA-Z0-9+.\-]*://[^\s'"<>,;)\]}]*[^\s'"<>,;)\]}.:!?]"#,
        )
        .expect("DataExcept URL regex must compile")
    })
}

fn tokens(name: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut previous_lower_or_digit = false;

    for character in name.chars() {
        if !character.is_ascii_alphanumeric() {
            if !current.is_empty() {
                result.push(std::mem::take(&mut current));
            }
            previous_lower_or_digit = false;
            continue;
        }

        if character.is_ascii_uppercase() && previous_lower_or_digit && !current.is_empty() {
            result.push(std::mem::take(&mut current));
        }

        current.push(character.to_ascii_lowercase());
        previous_lower_or_digit = character.is_ascii_lowercase() || character.is_ascii_digit();
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

fn is_sensitive(name: &str) -> bool {
    tokens(name)
        .iter()
        .any(|token| SENSITIVE_PARAM_TOKENS.contains(&token.as_str()))
}

/// Returns a short, one-way SHA-256 fingerprint.
#[must_use]
pub fn fingerprint(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    format!("{digest:x}")[..8].to_owned()
}

/// Replaces an optional secret with a placeholder and fingerprint.
#[must_use]
pub fn redact_secret(value: Option<&str>) -> Option<String> {
    value.map(|secret| {
        if secret.is_empty() {
            PLACEHOLDER.to_owned()
        } else {
            format!("{PLACEHOLDER}({})", fingerprint(secret))
        }
    })
}

/// Removes a known secret from free-form text.
#[must_use]
pub fn remove_secret(text: &str, secret: Option<&str>) -> String {
    let Some(secret) = secret else {
        return text.to_owned();
    };

    if secret.is_empty() || text.is_empty() || secret.len() < MIN_REMOVABLE_SECRET_LENGTH {
        return text.to_owned();
    }

    text.replace(
        secret,
        &format!("{PLACEHOLDER}({})", fingerprint(secret)),
    )
}

fn redact_params(query: &str) -> (String, bool) {
    if query.is_empty() {
        return (query.to_owned(), false);
    }

    let pairs: Vec<(String, String)> =
        form_urlencoded::parse(query.as_bytes()).into_owned().collect();

    if !pairs.iter().any(|(key, _)| is_sensitive(key)) {
        return (query.to_owned(), false);
    }

    let mut serializer = Serializer::new(String::new());
    for (key, value) in pairs {
        let replacement = if is_sensitive(&key) {
            PLACEHOLDER
        } else {
            &value
        };
        serializer.append_pair(&key, replacement);
    }

    (serializer.finish(), true)
}

fn split_once_owned(value: &str, delimiter: char) -> (&str, Option<&str>) {
    value
        .split_once(delimiter)
        .map_or((value, None), |(head, tail)| (head, Some(tail)))
}

fn valid_scheme(scheme: &str) -> bool {
    let mut chars = scheme.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    first.is_ascii_alphabetic()
        && chars.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '+' | '.' | '-')
        })
}

/// Strips credentials from a URL while preserving useful routing information.
#[must_use]
pub fn redact_url(url: &str, keep_path: bool) -> String {
    if url.is_empty() {
        return String::new();
    }

    let Some((scheme, remainder)) = url.split_once("://") else {
        return url.to_owned();
    };

    if !valid_scheme(scheme) {
        return url.to_owned();
    }

    let (without_fragment, fragment) = split_once_owned(remainder, '#');
    let (without_query, query) = split_once_owned(without_fragment, '?');

    let (authority, path) = without_query
        .split_once('/')
        .map_or((without_query, ""), |(authority, path)| {
            (authority, &without_query[authority.len()..])
        });

    if authority.is_empty() {
        return url.to_owned();
    }

    let mut redacted = false;
    let mut output_authority = authority.to_owned();

    if let Some((_, host)) = authority.rsplit_once('@') {
        output_authority = format!("{PLACEHOLDER}:{PLACEHOLDER}@{host}");
        redacted = true;
    }

    let output_query = query.map(|value| {
        let (redacted_query, changed) = redact_params(value);
        redacted |= changed;
        redacted_query
    });

    let output_fragment = fragment.map(|value| {
        if value.contains('=') {
            let (redacted_fragment, changed) = redact_params(value);
            redacted |= changed;
            redacted_fragment
        } else {
            value.to_owned()
        }
    });

    let output_path = if !keep_path && !path.trim_matches('/').is_empty() {
        redacted = true;
        format!("/{PLACEHOLDER}")
    } else {
        path.to_owned()
    };

    if !redacted {
        return url.to_owned();
    }

    let mut result = format!("{scheme}://{output_authority}{output_path}");
    if let Some(query) = output_query {
        result.push('?');
        result.push_str(&query);
    }
    if let Some(fragment) = output_fragment {
        result.push('#');
        result.push_str(&fragment);
    }

    result
}

/// Redacts a value only when it contains a URL scheme delimiter.
#[must_use]
pub fn redact_if_url(value: &str, keep_path: bool) -> String {
    if value.contains("://") {
        redact_url(value, keep_path)
    } else {
        value.to_owned()
    }
}

/// Redacts every URL found in free-form text.
#[must_use]
pub fn redact_urls_in_text(text: &str, keep_path: bool) -> String {
    if text.is_empty() || !text.contains("://") {
        return text.to_owned();
    }

    url_regex()
        .replace_all(text, |captures: &regex::Captures<'_>| {
            redact_url(&captures[0], keep_path)
        })
        .into_owned()
}

pub(crate) fn redact_json_value(value: &Value, keep_path: bool) -> Value {
    match value {
        Value::String(text) => Value::String(redact_urls_in_text(text, keep_path)),
        Value::Array(values) => Value::Array(
            values
                .iter()
                .map(|item| redact_json_value(item, keep_path))
                .collect(),
        ),
        Value::Object(values) => Value::Object(
            values
                .iter()
                .map(|(key, item)| (key.clone(), redact_json_value(item, keep_path)))
                .collect(),
        ),
        _ => value.clone(),
    }
}

pub(crate) fn redact_attributes(
    attributes: &BTreeMap<String, Value>,
    keep_path: bool,
) -> BTreeMap<String, Value> {
    attributes
        .iter()
        .map(|(key, value)| (key.clone(), redact_json_value(value, keep_path)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{fingerprint, redact_secret, redact_url, redact_urls_in_text, remove_secret};

    #[test]
    fn fingerprint_matches_sha256_prefix() {
        assert_eq!(fingerprint("secret"), "2bb80d53");
    }

    #[test]
    fn redact_secret_handles_empty_and_present_values() {
        assert_eq!(redact_secret(None), None);
        assert_eq!(redact_secret(Some("")), Some("***".to_owned()));
        assert_eq!(redact_secret(Some("secret")), Some("***(2bb80d53)".to_owned()));
    }

    #[test]
    fn known_short_secret_is_not_removed_from_text() {
        assert_eq!(
            remove_secret("authentication token", Some("tok")),
            "authentication token"
        );
    }

    #[test]
    fn userinfo_is_removed() {
        assert_eq!(
            redact_url("https://user:hunter2@host/path", true),
            "https://***:***@host/path"
        );
    }

    #[test]
    fn sensitive_query_is_removed_and_ordinary_query_survives() {
        assert_eq!(
            redact_url("https://h/p?token=SECRETVALUE&page=2", true),
            "https://h/p?token=***&page=2"
        );
    }

    #[test]
    fn sensitive_fragment_is_removed() {
        assert_eq!(
            redact_url("https://h/p#access_token=SECRETVALUE", true),
            "https://h/p#access_token=***"
        );
    }

    #[test]
    fn path_can_be_removed() {
        assert_eq!(
            redact_url("https://hooks.example.com/a/b/c?team=data", false),
            "https://hooks.example.com/***?team=data"
        );
    }

    #[test]
    fn free_text_keeps_trailing_punctuation() {
        assert_eq!(
            redact_urls_in_text("see https://h/p?token=SECRETVALUE: retry", true),
            "see https://h/p?token=***: retry"
        );
    }

    #[test]
    fn url_after_word_character_is_found() {
        assert_eq!(
            redact_urls_in_text("feature_https://h/p?token=SECRETVALUE", true),
            "feature_https://h/p?token=***"
        );
    }
}
