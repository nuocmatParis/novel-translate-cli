//! The stored run reader uses the producer's exact pinned grapheme rules.
use unicode_segmentation::UnicodeSegmentation;

use crate::SourceExtraction;

impl SourceExtraction {
    /// Check an encoded run against the same extended grapheme boundaries as
    /// extraction, without allocating a vector per grapheme. Source-node and
    /// snapshot identity checks belong to the caller's original-source reader.
    pub fn text_run_lengths_match<E>(
        text: &str,
        lengths: impl IntoIterator<Item = Result<usize, E>>,
    ) -> bool {
        let mut expected = text.graphemes(true);
        for length in lengths {
            match (length, expected.next()) {
                (Ok(length), Some(grapheme)) if length == grapheme.len() => {}
                _ => return false,
            }
        }
        expected.next().is_none()
    }
}
