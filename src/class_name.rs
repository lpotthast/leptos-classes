use std::borrow::Cow;
use std::ops::Deref;

/// Possible reasons why a candidate name is not a valid `class` name.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidClassName {
    /// The name was empty or contained only whitespace (any Unicode whitespace character,
    /// including ASCII space/tab/newline and characters such as the non-breaking space
    /// `U+00A0`).
    #[error("Class name is empty or whitespace-only: '{name}'")]
    Empty {
        /// The offending input (itself blank).
        name: Cow<'static, str>,
    },

    /// The name contained whitespace, suggesting the caller passed a whitespace-separated
    /// class list rather than a single token. "Whitespace" here is the Unicode definition
    /// (`char::is_whitespace`), which covers ASCII space/tab/newline and characters such as
    /// the non-breaking space `U+00A0` that paste-from-rich-text sources sometimes inject.
    #[error("Class names must not be whitespace-separated. Got: '{name}'.")]
    ContainsWhitespace {
        /// The offending input.
        name: Cow<'static, str>,
    },
}

/// A validated single CSS class token.
///
/// A `ClassName` always holds a non-empty string with no whitespace (Unicode definition: any
/// character for which [`char::is_whitespace`] returns `true`). Use [`ClassName::try_new`] for
/// runtime input you want to handle without panicking, or one of the `From` impls, which always
/// panic on invalid input, for known-valid literals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassName(Cow<'static, str>);

impl ClassName {
    /// Validates `name` and returns a [`ClassName`] on success.
    ///
    /// Returns [`InvalidClassName::Empty`] if `name` is empty or only whitespace, and
    /// [`InvalidClassName::ContainsWhitespace`] if it contains any whitespace. Both checks use
    /// the Unicode definition of whitespace ([`char::is_whitespace`]), so non-breaking spaces
    /// (`U+00A0`) and line/paragraph separators are rejected just like ASCII whitespace.
    pub fn try_new(name: impl Into<Cow<'static, str>>) -> Result<Self, InvalidClassName> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(InvalidClassName::Empty { name });
        }
        if name.chars().any(char::is_whitespace) {
            return Err(InvalidClassName::ContainsWhitespace { name });
        }
        Ok(Self(name))
    }

    /// Borrows the underlying class token as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for ClassName {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ClassName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<&'static str> for ClassName {
    fn from(s: &'static str) -> Self {
        Self::try_new(s).unwrap_or_else(|err| panic!("{err}"))
    }
}

impl From<String> for ClassName {
    fn from(s: String) -> Self {
        Self::try_new(s).unwrap_or_else(|err| panic!("{err}"))
    }
}

impl From<Cow<'static, str>> for ClassName {
    fn from(s: Cow<'static, str>) -> Self {
        Self::try_new(s).unwrap_or_else(|err| panic!("{err}"))
    }
}

#[cfg(test)]
mod try_new {
    use assertr::prelude::*;

    use super::{ClassName, InvalidClassName};

    #[test]
    fn accepts_plain_ascii_token() {
        let name = ClassName::try_new("btn-primary").unwrap();
        assert_that!(name.as_str()).is_equal_to("btn-primary");
    }

    #[test]
    fn accepts_non_whitespace_unicode_token() {
        // CSS3 allows Unicode identifiers; tokens with non-whitespace Unicode letters are valid.
        let name = ClassName::try_new("h\u{00E9}ros").unwrap();
        assert_that!(name.as_str()).is_equal_to("h\u{00E9}ros");
    }

    #[test]
    fn rejects_empty_input() {
        assert_that!(ClassName::try_new("")).is_err();
    }

    #[test]
    fn rejects_ascii_whitespace_only_input() {
        let result = ClassName::try_new(" \t\n");
        assert_that!(result)
            .is_err()
            .is_equal_to(InvalidClassName::Empty {
                name: " \t\n".into(),
            });
    }

    #[test]
    fn rejects_unicode_whitespace_only_input() {
        // U+00A0 NO-BREAK SPACE alone should classify as empty/blank.
        let result = ClassName::try_new("\u{00A0}\u{00A0}");
        assert_that!(result)
            .is_err()
            .is_equal_to(InvalidClassName::Empty {
                name: "\u{00A0}\u{00A0}".into(),
            });
    }

    #[test]
    fn rejects_token_with_ascii_whitespace_in_middle() {
        let result = ClassName::try_new("foo bar");
        assert_that!(result)
            .is_err()
            .is_equal_to(InvalidClassName::ContainsWhitespace {
                name: "foo bar".into(),
            });
    }

    #[test]
    fn rejects_token_with_non_breaking_space_in_middle() {
        // Defends the contract change: NBSP between letters is whitespace under the Unicode
        // definition and must fail validation even though its bytes are not ASCII whitespace.
        let result = ClassName::try_new("foo\u{00A0}bar");
        assert_that!(result)
            .is_err()
            .is_equal_to(InvalidClassName::ContainsWhitespace {
                name: "foo\u{00A0}bar".into(),
            });
    }

    #[test]
    fn rejects_token_with_line_separator_in_middle() {
        // U+2028 LINE SEPARATOR is Unicode-whitespace.
        let result = ClassName::try_new("foo\u{2028}bar");
        assert_that!(result)
            .is_err()
            .is_equal_to(InvalidClassName::ContainsWhitespace {
                name: "foo\u{2028}bar".into(),
            });
    }
}
