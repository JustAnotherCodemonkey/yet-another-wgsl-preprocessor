use core::ops::Range;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LocatedStr<'a> {
    pub inner_str: &'a str,
    pub start_location: TextLocation,
}

impl<'a> LocatedStr<'a> {
    // (absolute location, relative byte location, which needle)
    pub fn find_any_substr(
        &self,
        needle_candidates: &[&str],
    ) -> Option<(TextLocation, usize, usize)> {
        let largest_needle_size = needle_candidates.iter().map(|s| s.len()).max()?;
        let mut location_output = self.start_location;

        for (i, c) in self.inner_str.char_indices() {
            if let Some(maybe_needle_match) = self.inner_str.get(i..i + largest_needle_size) {
                for (candidate_index, needle_candidate) in needle_candidates.iter().enumerate() {
                    if maybe_needle_match == *needle_candidate {
                        return Some((location_output, i, candidate_index));
                    }
                }
                // else
                if c == '\n' {
                    location_output.line_num += 1;
                    location_output.col_num = 0;
                } else {
                    location_output.col_num += 1;
                }
                location_output.byte_num += c.len_utf8() as u64;
            } else {
                return None;
            }
        }

        None
    }

    pub fn find_with_fn(
        &self,
        mut needle: impl FnMut(char) -> bool,
    ) -> Option<(TextLocation, usize)> {
        let mut output = self.start_location;

        for (i, c) in self.inner_str.char_indices() {
            if needle(c) {
                return Some((output, i));
            } else if c == '\n' {
                output.line_num += 1;
                output.col_num = 0;
            } else {
                output.col_num += 1;
            }
            output.byte_num += c.len_utf8() as u64;
        }

        None
    }

    pub fn get_unchecked(self, index: Range<usize>) -> LocatedStr<'a> {
        let before_split = &self.inner_str[..index.start];
        let split = &self.inner_str[index.clone()];
        let mut new_txt_loc = self.start_location;
        new_txt_loc.byte_num += index.start as u64;

        let mut last_line_cols_before = 0;
        let mut rchars = before_split.chars().rev();
        for c in &mut rchars {
            if c == '\n' {
                new_txt_loc.line_num += 1;
                new_txt_loc.col_num = 0;
                break;
            } else {
                last_line_cols_before += 1;
            }
        }
        new_txt_loc.col_num += last_line_cols_before;
        for c in rchars {
            if c == '\n' {
                new_txt_loc.line_num += 1;
            }
        }

        LocatedStr {
            inner_str: split,
            start_location: new_txt_loc,
        }
    }

    pub fn new(inner: &'a str) -> LocatedStr<'a> {
        LocatedStr {
            inner_str: inner,
            start_location: TextLocation::default(),
        }
    }

    pub fn new_with_loc(
        inner_str: &'a str,
        line_num: u64,
        col_num: u64,
        byte_num: u64,
    ) -> LocatedStr<'a> {
        LocatedStr {
            inner_str,
            start_location: TextLocation {
                line_num,
                col_num,
                byte_num,
            },
        }
    }

    pub fn trim_start(mut self) -> LocatedStr<'a> {
        for (current_char_idx, current_char) in self.inner_str.char_indices() {
            if current_char.is_whitespace() {
                if current_char == '\n' {
                    self.start_location.col_num = 0;
                    self.start_location.line_num += 1;
                } else {
                    self.start_location.col_num += 1;
                }
            } else {
                self.start_location.byte_num += current_char_idx as u64;
                self.inner_str = &self.inner_str[current_char_idx..];
                return self;
            }
        }

        self.start_location.byte_num += self.inner_str.len() as u64;
        self.inner_str = &self.inner_str[self.inner_str.len()..];
        self
    }
}

impl<'a> From<&'a str> for LocatedStr<'a> {
    fn from(value: &'a str) -> Self {
        LocatedStr::new(value)
    }
}

impl<'a> From<(&'a str, TextLocation)> for LocatedStr<'a> {
    fn from(value: (&'a str, TextLocation)) -> Self {
        LocatedStr {
            inner_str: value.0,
            start_location: value.1,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TextLocation {
    pub line_num: u64,
    pub col_num: u64,
    pub byte_num: u64,
}

impl TextLocation {
    pub fn new(line_num: u64, col_num: u64, byte_num: u64) -> Self {
        TextLocation {
            line_num,
            col_num,
            byte_num,
        }
    }
}

impl From<(u64, u64, u64)> for TextLocation {
    fn from((line_num, col_num, byte_num): (u64, u64, u64)) -> Self {
        Self {
            line_num,
            col_num,
            byte_num,
        }
    }
}

impl From<&(u64, u64, u64)> for TextLocation {
    fn from(value: &(u64, u64, u64)) -> Self {
        Self::from(*value)
    }
}

/// Searches `haystack` for the first match of any needle candidate in `needle_candidates`
///
/// Returns (match location index, matched needle candidate index in candidate list).
///
/// Needle candidates are prioritized in order (first comes first if both match).
pub(crate) fn find_any_substring(
    haystack: &str,
    needle_candidates: &[&str],
) -> Option<(usize, usize)> {
    for (char_index, _current_char) in haystack.char_indices() {
        for (candidate_index, candidate) in needle_candidates.iter().enumerate() {
            if let Some(haystack_comp) = haystack.get(char_index..char_index + candidate.len()) {
                if haystack_comp == *candidate {
                    return Some((char_index, candidate_index));
                }
            }
        }
    }

    return None;
}

#[cfg(test)]
mod tests {
    use super::*;

    mod located_str {
        use super::*;

        #[test]
        fn test_find_any_substr() {
            let test_cases: &[(_, _, &[_], _)] =
                &[("test", (1, 1, 3), &["s", "e"], ((1, 2, 4), 1, 1))];
            for (s, lt, ns, ot) in test_cases {
                assert_eq!(
                    LocatedStr {
                        inner_str: s,
                        start_location: lt.into(),
                    }
                    .find_any_substr(ns),
                    Some((ot.0.into(), ot.1, ot.2))
                );
            }
        }

        #[test]
        fn test_find_with_fn() {
            let test_cases: &[(&str, &dyn Fn(char) -> bool, (u64, u64, u64))] = &[
                ("Test String!", &|c| c == 'S', (0, 5, 5)),
                ("Multi\nLine", &|c| c == 'L', (1, 0, 6)),
            ];

            for (tstr, tneedle, toutput) in test_cases.iter() {
                assert_eq!(
                    LocatedStr {
                        inner_str: tstr,
                        start_location: TextLocation::default(),
                    }
                    .find_with_fn(tneedle)
                    .unwrap(),
                    // Currently no tests begin with a non-zero TextLocation
                    (TextLocation::from(toutput), toutput.2 as usize)
                );
            }
        }

        #[test]
        fn test_get_unchecked() {
            fn t(
                vs: &str,
                vloc: impl Into<TextLocation>,
                range: Range<usize>,
                ts: &str,
                tloc: impl Into<TextLocation>,
            ) {
                assert_eq!(
                    LocatedStr {
                        inner_str: vs,
                        start_location: vloc.into(),
                    }
                    .get_unchecked(range),
                    LocatedStr {
                        inner_str: ts,
                        start_location: tloc.into(),
                    }
                );
            }

            let s = "This is the complete thing!\nLine";
            t(
                s,
                (1, 0, 12),
                0..s.len(),
                "This is the complete thing!\nLine",
                (1, 0, 12),
            );
            let s = "Line 1\nLine 2 desired";
            t(s, (0, 0, 0), 14..s.len(), "desired", (1, 7, 14));
        }

        #[test]
        fn test_trim_start() {
            let test_cases = &[
                (
                    ("   \t\r lol", (0, 0, 0)),
                    LocatedStr {
                        inner_str: "lol",
                        start_location: (0, 6, 6).into(),
                    },
                ),
                (
                    (" \r\n\n start!", (0, 0, 0)),
                    LocatedStr {
                        inner_str: "start!",
                        start_location: (2, 1, 5).into(),
                    },
                ),
                (
                    (" ", (0, 0, 0)),
                    LocatedStr {
                        inner_str: "",
                        start_location: (0, 1, 1).into(),
                    },
                ),
            ];

            for ((tstr, start_loc), desired_result) in test_cases.iter() {
                let ls = LocatedStr {
                    inner_str: tstr,
                    start_location: start_loc.into(),
                }.trim_start();
                assert_eq!(&ls, desired_result);
            }
        }
    }

    #[test]
    fn test_find_any_substring() {
        let test_cases: &[((_, &[_]), _)] = &[
            (("this is a test", &["aa", "a", "e"]), Some((8, 1))),
            (("none", &["z"]), None),
            (("zabp", &["ab", "a"]), Some((1, 0))),
            (
                (
                    "Inspired by a bug in CppCommentIter",
                    &["verylongneedle", "Iter"],
                ),
                Some((31, 1)),
            ),
        ];
        for ((h, n), r) in test_cases {
            assert_eq!(find_any_substring(h, n), *r);
        }
    }
}
