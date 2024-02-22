mod utils;

#[cfg(test)]
mod tests;

use crate::utils::{find_any_substring, LocatedStr, TextLocation};
use core::fmt::Display;
#[cfg(feature = "std")]
use std::error::Error;
use utils::CppCommentIter;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct LocatedStrMacroTokenIter<'a, 'b> {
    source_remaining: LocatedStr<'a>,
    syntax_settings: SyntaxSettings<'b>,
}

impl<'a, 'b> LocatedStrMacroTokenIter<'a, 'b> {
    pub(crate) fn new_with_default_syntax(
        source: LocatedStr<'a>,
    ) -> LocatedStrMacroTokenIter<'a, 'static> {
        LocatedStrMacroTokenIter {
            source_remaining: source,
            syntax_settings: SyntaxSettings::default(),
        }
    }
}

impl<'a, 'b> Iterator for LocatedStrMacroTokenIter<'a, 'b> {
    type Item = MacroTokenResult<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.source_remaining = self.source_remaining.trim_start();
        if self.source_remaining.inner_str.is_empty() {
            return None;
        }
        let end_ident = self.syntax_settings.macro_end_ident;
        let maybe_end_ident = self.source_remaining.inner_str.get(..end_ident.len());
        if maybe_end_ident == Some(end_ident) {
            let terminator_loc = self.source_remaining.start_location;
            self.source_remaining = self.source_remaining.get_unchecked(
                self.syntax_settings.macro_end_ident.len()..self.source_remaining.inner_str.len(),
            );
            return Some(MacroTokenResult::Terminator(terminator_loc));
        }
        let source_len = self.source_remaining.inner_str.len();

        let mut char_indices = self.source_remaining.inner_str.char_indices();

        // Symbol handling (for now, non-alphanumeric tokens are char-by-char).
        // SAFETY: We know that there is a char in the str otherwise the above would have caught an
        // empty str.
        let (_, first_char) = char_indices.next().unwrap();
        if !first_char.is_alphanumeric() {
            let char_len = first_char.len_utf8();
            let output = Some(MacroTokenResult::SymbolToken(
                self.source_remaining.get_unchecked(0..char_len),
            ));
            self.source_remaining = self
                .source_remaining
                .get_unchecked(char_len..self.source_remaining.inner_str.len());
            return output;
        }

        // Must be alphanum. Find the end.
        for (i, c) in char_indices {
            if !c.is_alphanumeric() {
                let output = Some(MacroTokenResult::AlphanumStringToken(
                    self.source_remaining.get_unchecked(0..i),
                ));
                self.source_remaining = self.source_remaining.get_unchecked(i..source_len);
                return output;
            }
            let maybe_end_ident = self.source_remaining.inner_str.get(..end_ident.len());
            if maybe_end_ident == Some(end_ident) {
                let output = Some(MacroTokenResult::AlphanumStringToken(
                    self.source_remaining.get_unchecked(0..i),
                ));
                self.source_remaining = self.source_remaining.get_unchecked(i..source_len);
                return output;
            }
        }

        // Must have hit end of source str. Source string is assumed to end on token boundery.
        return Some(MacroTokenResult::AlphanumStringToken(
            self.source_remaining.clone(),
        ));
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SyntaxSettings<'a> {
    marcro_start_ident: &'a str,
    macro_end_ident: &'a str,
}

impl<'a> Default for SyntaxSettings<'a> {
    fn default() -> Self {
        SyntaxSettings {
            marcro_start_ident: "#",
            macro_end_ident: ";",
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TotalMacroTokenIter<'a, 'b> {
    source_remaining: LocatedStr<'a>,
    maybe_current_cpp_comment_iter: Option<CppCommentIter<'a, 'b>>,
    current_macro_token_iter: LocatedStrMacroTokenIter<'a, 'b>,
    syntax_settings: SyntaxSettings<'b>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum MacroTokenResult<'a> {
    AlphanumStringToken(LocatedStr<'a>),
    SymbolToken(LocatedStr<'a>),
    Terminator(TextLocation),
}
