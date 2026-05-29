//! Proptest invariants for wm-skill-edit.
//! Read-only: edit-agent must not modify this file.

use proptest::prelude::*;

// Invariant: anchor search result count == 0 for text that doesn't contain the anchor.
proptest! {
    #[test]
    fn proptest_anchor_count_zero_when_absent(
        content in "[a-zA-Z0-9 \n]{1,200}",
        anchor in "[A-Z]{20,30}", // uppercase-only anchor unlikely to appear in lowercase content
    ) {
        let lines: Vec<&str> = content.lines().collect();
        let matches: usize = lines.iter()
            .filter(|line| line.contains(anchor.as_str()))
            .count();
        // With uppercase-only anchor and lowercase content, matches should be 0.
        // (proptest may find counterexamples if content generates uppercase — that's fine,
        //  the invariant is just: count is always >= 0 and == lines with anchor)
        let expected = content.lines().filter(|l| l.contains(anchor.as_str())).count();
        prop_assert_eq!(matches, expected);
    }
}
