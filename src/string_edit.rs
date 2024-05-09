use unicode_segmentation::UnicodeSegmentation;

pub trait StringEdit {
    fn backspace(&mut self, cursor: usize) -> usize;
    fn ctrl_backspace_unicode_word(&mut self, cursor: usize) -> usize;
    fn left_arrow(&mut self, cursor: usize) -> usize;
    fn right_arrow(&mut self, cursor: usize) -> usize;
}

impl StringEdit for String {


    fn backspace(&mut self, cursor: usize) -> usize {
        let previous_grapheme = self[0..cursor].grapheme_indices(true).rev().next();
        if let Some((prev_idx, _prev_grapheme)) = previous_grapheme {
            self.replace_range(prev_idx..cursor, "");
            return prev_idx;
        }
        return cursor;
    }

    fn ctrl_backspace_unicode_word(&mut self, cursor: usize) -> usize {
        let previous_grapheme = self[0..cursor].unicode_word_indices().rev().next();
        if let Some((prev_idx, _prev_grapheme)) = previous_grapheme {
            self.replace_range(prev_idx..cursor, "");
            return prev_idx;
        }
        return cursor;
    }

    fn left_arrow(&mut self, cursor: usize) -> usize {
        let previous_grapheme = self[0..cursor].grapheme_indices(true).rev().next();
        if let Some((prev_idx, _prev_grapheme)) = previous_grapheme {
            return prev_idx;
        }
        return cursor;
    }

    fn right_arrow(&mut self, cursor: usize) -> usize {
        let previous_grapheme = self[0..cursor].grapheme_indices(true).next();
        if let Some((prev_idx, _prev_grapheme)) = previous_grapheme {
            return prev_idx;
        }
        return cursor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backspace() {
        let results = ["Sneed", "need", "Seed", "Sned", "Sned", "Snee"];

        for i in 0..5 {
            let mut string = String::from("Sneed");
            let new_cursor = string.backspace(i);
            assert_eq!(string, results[i]);
            let expected_new_cursor = i.saturating_sub(1);
            assert_eq!(new_cursor, expected_new_cursor);
        }

        let results = [
            "種子と飼料",
            "子と飼料",
            "種と飼料",
            "種子飼料",
            "種子と料",
            "種子と飼",
        ];
        let source = "種子と飼料";
        let indices: Vec<usize> = source.char_indices().map(|x| x.0).collect();

        for (i, idx) in indices.iter().enumerate() {
            let mut string = String::from(source);
            let new_cursor = string.backspace(*idx);
            assert_eq!(string, results[i]);
            let expected_new_cursor = indices[i.saturating_sub(1)];
            assert_eq!(new_cursor, expected_new_cursor);
        }
    }
}
