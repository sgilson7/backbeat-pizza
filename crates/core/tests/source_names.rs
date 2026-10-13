//! The person's request: nothing in the game, and no human-facing file in
//! this repository, names the game it is based on, its characters or its
//! publisher. This test reads every text file in the repository (the build
//! output and tool folders aside) and fails on any of those names.
//!
//! The names are kept in ROT13 so that this file does not name them either.

use std::fs;
use std::path::{Path, PathBuf};

/// ROT13 of the names that must not appear, matched without regard to case.
/// Single words match only as whole words.
const ENCODED: &[&str] = &[
    "mbbzovav",
    "gebyy",
    "cvmmn cnff",
    "neab",
    "ful thl",
    "oebqreohaq",
    "yrneavat pbzcnal",
    "syrra",
    "ohooyrjbbq",
];

fn rot13(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='m' | 'A'..='M' => (c as u8 + 13) as char,
            'n'..='z' | 'N'..='Z' => (c as u8 - 13) as char,
            _ => c,
        })
        .collect()
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

const SKIP_DIRS: &[&str] = &["target", "dist", ".venv", ".git", "node_modules", "__pycache__"];

fn text_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(read) = fs::read_dir(dir) else {
        return;
    };
    for entry in read {
        let path = entry.expect("entry").path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            if !SKIP_DIRS.contains(&name) {
                text_files(&path, out);
            }
        } else if let Ok(text) = fs::read_to_string(&path) {
            let _ = text;
            out.push(path);
        }
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric()
}

fn names_in(text: &str) -> Vec<String> {
    let hay = text.to_lowercase();
    let mut found = Vec::new();
    for enc in ENCODED {
        let name = rot13(enc);
        let whole_word = !name.contains(' ');
        let mut from = 0;
        while let Some(at) = hay[from..].find(&name) {
            let start = from + at;
            let end = start + name.len();
            let before = hay[..start].chars().last().map(is_word_char).unwrap_or(false);
            // a plural or possessive still names it
            let after = hay[end..]
                .trim_start_matches(['s', '\''])
                .chars()
                .next()
                .map(is_word_char)
                .unwrap_or(false);
            let prefix_ok = name != rot13("mbbzovav");
            if !whole_word || !prefix_ok || (!before && !after) {
                found.push(name.clone());
                break;
            }
            from = end;
        }
    }
    found
}

#[test]
fn no_file_names_the_source_game_its_characters_or_its_publisher() {
    let mut files = Vec::new();
    text_files(&root(), &mut files);
    assert!(files.len() > 20, "the scan found the repository's files");
    let mut problems = Vec::new();
    for f in files {
        let text = fs::read_to_string(&f).unwrap_or_default();
        for name in names_in(&text) {
            problems.push(format!(
                "{}: names \"{}\" (ROT13 {})",
                f.strip_prefix(root()).unwrap_or(&f).display(),
                name,
                rot13(&name)
            ));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn the_scan_catches_each_name_and_leaves_ordinary_words_alone() {
    for enc in ENCODED {
        let name = rot13(enc);
        assert_eq!(names_in(&format!("A {name} here.")), vec![name.clone()]);
        assert_eq!(names_in(&format!("THE {}S.", name.to_uppercase())), vec![name]);
    }
    assert!(names_in("Barnone trolley, a fleeting arnold.").is_empty());
}
