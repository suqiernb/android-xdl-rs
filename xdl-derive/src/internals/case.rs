//! Code to convert the Rust-styled field/variant (e.g. `my_field`, `MyType`) to the
//! case of the source (e.g. `my-field`, `MY_FIELD`).

use self::RenameRule::*;
use std::fmt::{self, Debug, Display};

/// The different possible ways to change case of fields in a struct, or variants in an enum.
#[derive(Copy, Clone, PartialEq, Default)]
pub enum RenameRule {
    /// Don't apply a default rename rule.
    #[default]
    None,
    /// Rename direct children to "lowercase" style.
    LowerCase,
    /// Rename direct children to "UPPERCASE" style.
    UpperCase,
    /// Rename direct children to "PascalCase" style, as typically used for
    /// enum variants.
    PascalCase,
    /// Rename direct children to "camelCase" style.
    CamelCase,
    /// Rename direct children to "snake_case" style, as commonly used for
    /// fields.
    SnakeCase,
    /// Rename direct children to "SCREAMING_SNAKE_CASE" style, as commonly
    /// used for constants.
    ScreamingSnakeCase,
}

static RENAME_RULES: &[(&str, RenameRule)] = &[
    ("lowercase", LowerCase),
    ("UPPERCASE", UpperCase),
    ("PascalCase", PascalCase),
    ("camelCase", CamelCase),
    ("snake_case", SnakeCase),
    ("SCREAMING_SNAKE_CASE", ScreamingSnakeCase),
];

impl RenameRule {
    pub fn from_str(rename_all_str: &str) -> Result<Self, ParseError<'_>> {
        for (name, rule) in RENAME_RULES {
            if rename_all_str == *name {
                return Ok(*rule);
            }
        }
        Err(ParseError {
            unknown: rename_all_str,
        })
    }

    /// Apply a renaming rule to a struct field, returning the version expected in the source.
    pub fn apply_to_field(self, field: &str) -> String {
        match self {
            None | SnakeCase => field.to_owned(),
            LowerCase => field.to_ascii_lowercase(),
            UpperCase => field.to_ascii_uppercase(),
            PascalCase => {
                let mut pascal = String::new();
                let mut capitalize = true;
                for ch in field.chars() {
                    if ch == '_' {
                        capitalize = true;
                    } else if capitalize {
                        pascal.push(ch.to_ascii_uppercase());
                        capitalize = false;
                    } else {
                        pascal.push(ch);
                    }
                }
                pascal
            }
            CamelCase => {
                let pascal = PascalCase.apply_to_field(field);
                pascal[..1].to_ascii_lowercase() + &pascal[1..]
            }
            ScreamingSnakeCase => field.to_ascii_uppercase(),
        }
    }

    /// Returns the `RenameRule` if it is not `None`, `rule_b` otherwise.
    pub fn or(self, rule_b: Self) -> Self {
        match self {
            None => rule_b,
            _ => self,
        }
    }
}

pub struct ParseError<'a> {
    unknown: &'a str,
}

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("unknown rename rule `rename = ")?;
        Debug::fmt(self.unknown, f)?;
        f.write_str("`, expected one of ")?;
        for (i, (name, _rule)) in RENAME_RULES.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            Debug::fmt(name, f)?;
        }
        Ok(())
    }
}
