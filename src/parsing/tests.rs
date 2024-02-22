use super::{*, utils::*};

mod cpp_comment_iter {
    use super::*;

    #[test]
    fn test_iter_impl() {
        let test_cases = [
            (
                "//Starts at line 1\nNAL (Not A Line)",
                (1, 0, 1),
                [("Starts at line 1", (1, 2, 3))].as_slice(),
            ),
            (
                "// Line 1\n// Line 2\nNAL (Not A Line)\n",
                (0, 0, 0),
                &[(" Line 1", (0, 2, 2)), (" Line 2", (1, 2, 12))],
            ),
            (
                " // Barely indented
                    // Very indented",
                (0, 0, 0),
                &[
                    (" Barely indented", (0, 3, 3)),
                    (" Very indented", (1, 22, 42)),
                ],
            ),
        ];
        for (ts, tsl, ess) in test_cases {
            let mut tested_iter = CppCommentIter::new_with_default_syntax(LocatedStr {
                inner_str: ts,
                start_location: tsl.into(),
            });
            let mut expected_iter = ess.into_iter();
            while let (Some(tsr), Some(es)) = (tested_iter.next(), expected_iter.next()) {
                assert_eq!(
                    tsr,
                    LocatedStr {
                        inner_str: es.0,
                        start_location: es.1.into(),
                    }
                );
            }
            assert_eq!((tested_iter.next(), expected_iter.next()), (None, None));
        }
    }
}

mod located_str_macro_token_iter {
    use super::*;

    #[test]
    fn test_alphanum_and_symbol_tokens() {
        use MacroTokenResult::{AlphanumStringToken, SymbolToken, Terminator};

        let mut iter = LocatedStrMacroTokenIter::new_with_default_syntax(LocatedStr::new(
            "hi! Test+\nLine2 2Line;",
        ));
        let expected_tokens = [
            AlphanumStringToken(LocatedStr::new("hi")),
            SymbolToken(LocatedStr::new_with_loc("!", 0, 2, 2)),
            AlphanumStringToken(LocatedStr::new_with_loc("Test", 0, 4, 4)),
            SymbolToken(LocatedStr::new_with_loc("+", 0, 8, 8)),
            AlphanumStringToken(LocatedStr::new_with_loc("Line2", 1, 0, 10)),
            AlphanumStringToken(LocatedStr::new_with_loc("2Line", 1, 6, 16)),
            Terminator(TextLocation::new(1, 11, 21)),
        ];
        let results = core::array::from_fn(|_| iter.next().unwrap());
        assert_eq!(iter.next(), None);
        assert_eq!(results, expected_tokens);
    }

    #[test]
    fn test_empty_source_conditions() {
        let test_cases = &["", " ", "\n", "\r\n"];
        for case in test_cases.iter() {
            assert_eq!(
                LocatedStrMacroTokenIter::new_with_default_syntax(LocatedStr::new(case)).next(),
                None
            );
        }
    }

    #[test]
    fn test_find_end_ident_at_start() {
        assert_eq!(
            LocatedStrMacroTokenIter::new_with_default_syntax(LocatedStr::new(";")).next(),
            Some(MacroTokenResult::Terminator(TextLocation::default()))
        );
    }

    #[test]
    fn test_get_symbol_token() {
        let mut iter =
            LocatedStrMacroTokenIter::new_with_default_syntax(LocatedStr::new("+§&\n-😀_;"));
        let mut expected_tokens_iter = [
            ("+", 0, 0, 0),
            ("§", 0, 1, 1),
            ("&", 0, 2, 3),
            ("-", 1, 0, 5),
            ("😀", 1, 1, 6),
            ("_", 1, 2, 10),
            (";", 1, 3, 11),
        ]
        .iter()
        .map(|(s, line, col, byte)| {
            if *s != ";" {
                MacroTokenResult::SymbolToken(LocatedStr {
                    inner_str: s,
                    start_location: (*line, *col, *byte).into(),
                })
            } else {
                MacroTokenResult::Terminator((*line, *col, *byte).into())
            }
        });
        let results: [_; 7] = core::array::from_fn(|_| iter.next());
        let expected_tokens: [_; 7] = core::array::from_fn(|_| expected_tokens_iter.next());
        assert!(iter.next() == None && expected_tokens_iter.next() == None);
        assert_eq!(results, expected_tokens);
    }
}
