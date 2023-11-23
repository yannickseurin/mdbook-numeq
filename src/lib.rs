//! An [mdBook](https://github.com/rust-lang/mdBook) preprocessor for automatically numbering centered equations.

use log::warn;
use mdbook::book::{Book, BookItem};
use mdbook::errors::Result;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use pathdiff::diff_paths;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The preprocessor name.
const NAME: &str = "numeq";

/// A preprocessor for automatically numbering centered equations.
#[derive(Default)]
pub struct NumEqPreprocessor {
    /// Whether equation numbers must be prefixed by the section number.
    with_prefix: bool,
}

/// The `LabelInfo` structure contains information for formatting the hyperlink to a specific equation.
#[derive(Debug, PartialEq)]
struct LabelInfo {
    /// The number associated with the labeled equation.
    num: String,
    /// The path to the file containing the environment with the label.
    path: PathBuf,
}

impl NumEqPreprocessor {
    pub fn new(ctx: &PreprocessorContext) -> Self {
        let mut pre = Self::default();

        if let Some(toml::Value::Boolean(b)) = ctx.config.get("preprocessor.numeq.prefix") {
            pre.with_prefix = *b;
        }

        pre
    }
}

impl Preprocessor for NumEqPreprocessor {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        // a hashmap mapping labels to `LabelInfo` structs
        let mut refs: HashMap<String, LabelInfo> = HashMap::new();

        book.for_each_mut(|item: &mut BookItem| {
            if let BookItem::Chapter(chapter) = item {
                if !chapter.is_draft_chapter() {
                    // one can safely unwrap chapter.path which must be Some(...)
                    let prefix = if self.with_prefix {
                        match &chapter.number {
                            Some(sn) => sn.to_string(),
                            None => String::new(),
                        }
                    } else {
                        String::new()
                    };
                    let path = chapter.path.as_ref().unwrap();
                    chapter.content =
                        find_and_replace_eqs(&chapter.content, &prefix, path, &mut refs);
                }
            }
        });

        book.for_each_mut(|item: &mut BookItem| {
            if let BookItem::Chapter(chapter) = item {
                if !chapter.is_draft_chapter() {
                    // one can safely unwrap chapter.path which must be Some(...)
                    let path = chapter.path.as_ref().unwrap();
                    chapter.content = find_and_replace_refs(&chapter.content, path, &refs);
                }
            }
        });

        Ok(book)
    }
}

/// Finds all patterns `{{numeq}}{mylabel}` (where `{mylabel}` is optional) and replaces them by `\label{mylabel} \tag{ctr}`;
/// if a label is provided, updates the hashmap `refs` with an entry (label, LabelInfo) allowing to format links to the equation.
fn find_and_replace_eqs(
    s: &str,
    prefix: &str,
    path: &Path,
    refs: &mut HashMap<String, LabelInfo>,
) -> String {
    let mut ctr = 0;

    // see https://regex101.com/ for an explanation of the regex
    let re: Regex = Regex::new(r"\{\{numeq\}\}(\{(?P<label>.*?)\})?").unwrap();

    re.replace_all(s, |caps: &regex::Captures| {
        ctr += 1;
        match caps.name("label") {
            Some(lb) => {
                // if a label is given, we must update the hashmap
                let label = lb.as_str().to_string();
                if refs.contains_key(&label) {
                    // if the same label has already been used we emit a warning and don't update the hashmap
                    warn!("Eq. {prefix}{ctr}: Label `{label}' already used");
                } else {
                    refs.insert(
                        label.clone(),
                        LabelInfo {
                            num: format!("{prefix}{ctr}"),
                            path: path.to_path_buf(),
                        },
                    );
                }
                format!("\\htmlId{{{label}}}{{}} \\tag{{{prefix}{ctr}}}")
            }
            None => {
                format!("\\tag{{{prefix}{ctr}}}")
            }
        }
    })
    .to_string()
}

/// Finds and replaces all patterns {{eqref: label}} where label is an existing key in hashmap `refs`
/// with link towards the relevant theorem.
fn find_and_replace_refs(
    s: &str,
    chap_path: &PathBuf,
    refs: &HashMap<String, LabelInfo>,
) -> String {
    // see https://regex101.com/ for an explanation of the regex
    let re: Regex = Regex::new(r"\{\{eqref:\s*(?P<label>.*?)\}\}").unwrap();

    re.replace_all(s, |caps: &regex::Captures| {
        let label = caps.name("label").unwrap().as_str().to_string();
        if refs.contains_key(&label) {
            let text = &refs.get(&label).unwrap().num;
            let path_to_ref = &refs.get(&label).unwrap().path;
            let rel_path = compute_rel_path(chap_path, path_to_ref);
            format!("[({text})]({rel_path}#{label})")
        } else {
            warn!("Unknown equation reference: {}", label);
            "**[??]**".to_string()
        }
    })
    .to_string()
}

/// Computes the relative path from the folder containing `chap_path` to the file `path_to_ref`.
fn compute_rel_path(chap_path: &PathBuf, path_to_ref: &PathBuf) -> String {
    if chap_path == path_to_ref {
        return "".to_string();
    }
    let mut local_chap_path = chap_path.clone();
    local_chap_path.pop();
    format!(
        "{}",
        diff_paths(path_to_ref, &local_chap_path).unwrap().display()
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use lazy_static::lazy_static;

    const SECNUM: &str = "1.2.";

    lazy_static! {
        static ref PATH: PathBuf = "crypto/groups.md".into();
    }

    #[test]
    fn no_label() {
        let mut refs = HashMap::new();
        let input = String::from(r"{{numeq}}");
        let output = find_and_replace_eqs(&input, SECNUM, &PATH, &mut refs);
        let expected = String::from("\\tag{1.2.1}");
        assert_eq!(output, expected);
        assert!(refs.is_empty());
    }

    #[test]
    fn with_label() {
        let mut refs = HashMap::new();
        let input = String::from(r"{{numeq}}{eq:test}");
        let output = find_and_replace_eqs(&input, SECNUM, &PATH, &mut refs);
        let expected = String::from("\\htmlId{eq:test}{} \\tag{1.2.1}");
        assert_eq!(output, expected);
        assert_eq!(
            *refs.get("eq:test").unwrap(),
            LabelInfo {
                num: "1.2.1".to_string(),
                path: "crypto/groups.md".into(),
            }
        )
    }
}
