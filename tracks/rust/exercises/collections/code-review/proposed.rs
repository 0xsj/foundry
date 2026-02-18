// Word Frequency Counter — Proposed Code for Review
//
// A text statistics tool that computes word frequencies, top words,
// unique word counts, and stop-word filtering.
//
// Run: rustc proposed.rs && ./proposed

use std::collections::HashMap;

/// Counts how many times each word appears in the text.
/// Words are lowercased and split on whitespace.
pub fn word_frequencies(text: &str) -> HashMap<String, u32> {
    let mut counts: HashMap<String, u32> = HashMap::new();

    for word in text.split_whitespace() {
        let word = word.to_lowercase();

        // Check if we've seen this word before
        if counts.contains_key(&word) {
            let current = counts.get(&word).unwrap();
            counts.insert(word, current + 1);
        } else {
            counts.insert(word, 1);
        }
    }

    counts
}

/// Returns the top n words by frequency, sorted by count descending.
/// Ties broken by word alphabetically.
pub fn top_words(frequencies: &HashMap<String, u32>, n: usize) -> Vec<String> {
    let mut pairs: Vec<(&String, &u32)> = frequencies.iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));

    // Collect word-count pairs into a flat Vec<String> where each word
    // is followed by its count: ["the", "42", "and", "31", ...]
    let result: Vec<String> = pairs
        .iter()
        .take(n)
        .flat_map(|(word, count)| vec![word.to_string(), count.to_string()])
        .collect();

    result
}

/// Returns the number of unique words in the text.
pub fn unique_word_count(text: &str) -> usize {
    // Re-use word_frequencies but only need to count the keys
    let frequencies = word_frequencies(text);
    frequencies.len()
}

/// Returns true if the given word appears in the text.
/// Uses a linear scan through the HashMap values.
pub fn word_exists(frequencies: &HashMap<String, u32>, word: &str) -> bool {
    let target = word.to_lowercase();
    for (k, _) in frequencies.iter() {
        if k == &target {
            return true;
        }
    }
    false
}

/// Removes all stop words from the frequency map.
/// stop_words is a Vec of words to remove.
pub fn remove_stop_words(
    frequencies: &mut HashMap<String, u32>,
    stop_words: Vec<String>,
) {
    for word in &stop_words {
        frequencies.remove(word);
    }
}

/// Returns the total number of words (including duplicates) in the text.
pub fn total_word_count(text: &str) -> usize {
    // Compute full frequency map just to sum the values
    let frequencies = word_frequencies(text);
    frequencies.values().sum::<u32>() as usize
}

/// Checks which words from the required list are missing from the text.
pub fn missing_words(text: &str, required: Vec<String>) -> Vec<String> {
    let frequencies = word_frequencies(text);

    let mut missing: Vec<String> = Vec::new();
    for word in required {
        if !frequencies.contains_key(&word) {
            missing.push(word);
        }
    }
    missing
}

// ----- Tests -----

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "the quick brown fox jumps over the lazy dog the fox";

    #[test]
    fn test_word_frequencies() {
        let freq = word_frequencies(SAMPLE);
        assert_eq!(freq["the"], 3);
        assert_eq!(freq["fox"], 2);
        assert_eq!(freq["quick"], 1);
    }

    #[test]
    fn test_top_words_format() {
        let freq = word_frequencies(SAMPLE);
        let top = top_words(&freq, 2);
        // top is ["the", "42", "fox", "2"] style — word and count interleaved
        assert_eq!(top.len(), 4);  // 2 pairs = 4 strings
        assert_eq!(top[0], "the");
        assert_eq!(top[1], "3");
    }

    #[test]
    fn test_unique_word_count() {
        let count = unique_word_count(SAMPLE);
        assert_eq!(count, 8);  // the, quick, brown, fox, jumps, over, lazy, dog
    }

    #[test]
    fn test_word_exists() {
        let freq = word_frequencies(SAMPLE);
        assert!(word_exists(&freq, "fox"));
        assert!(!word_exists(&freq, "cat"));
    }

    #[test]
    fn test_remove_stop_words() {
        let mut freq = word_frequencies(SAMPLE);
        remove_stop_words(
            &mut freq,
            vec!["the".to_string(), "over".to_string()],
        );
        assert!(!freq.contains_key("the"));
        assert!(!freq.contains_key("over"));
        assert!(freq.contains_key("fox"));
    }

    #[test]
    fn test_total_word_count() {
        assert_eq!(total_word_count(SAMPLE), 11);  // 11 words including duplicates
    }
}

fn main() {
    let text = "the quick brown fox jumps over the lazy dog the fox";

    let freq = word_frequencies(text);
    println!("Unique words: {}", freq.len());

    let top = top_words(&freq, 3);
    println!("Top 3 (word/count pairs): {:?}", top);

    println!("Total words: {}", total_word_count(text));

    let exists = word_exists(&freq, "fox");
    println!("'fox' exists: {exists}");
}
