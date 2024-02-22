use crate::utils::{LocatedStr,find_any_substring};
use super::SyntaxSettings;

// Stops when it encounters end or something that isn't a c++ style comment.
// Expects first non-whitespace chars to be //
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct CppCommentIter<'a, 'b> {
    source_remaining: LocatedStr<'a>,
    syntax_settings: SyntaxSettings<'b>,
}

impl<'a, 'b> CppCommentIter<'a, 'b> {
    pub(crate) fn new_with_default_syntax(source: LocatedStr<'a>) -> CppCommentIter<'a, 'static> {
        CppCommentIter {
            source_remaining: source,
            syntax_settings: SyntaxSettings::default(),
        }
    }

    pub(crate) fn source_remaining(&self) -> &LocatedStr<'a> {
        &self.source_remaining
    }
}

impl<'a, 'b> Iterator for CppCommentIter<'a, 'b> {
    type Item = LocatedStr<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.source_remaining = self.source_remaining.trim_start();
        if self.source_remaining.inner_str.get(..2) != Some("//") {
            return None;
        }
        self.source_remaining = self
            .source_remaining
            .get_unchecked(2..self.source_remaining.inner_str.len());
        let inner_len = self.source_remaining.inner_str.len();

        let (line_end_idx, next_line_idx) =
            find_any_substring(self.source_remaining.inner_str, &["\r\n", "\n"])
                .map(|(idx, needle)| match needle {
                    0 => (idx, idx + 2),
                    1 => (idx, idx + 1),
                    _ => unreachable!(),
                })
                .unwrap_or((inner_len, inner_len));
        let output = self.source_remaining.get_unchecked(0..line_end_idx);
        self.source_remaining = self
            .source_remaining
            .get_unchecked(next_line_idx..inner_len);

        Some(output)
    }
}
