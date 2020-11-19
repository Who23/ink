//! Tools for creating diffs, done through the `Diff` struct
mod algo;
mod edit;
mod parser;

use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use edit::{Edit, Operation};

/// Struct that holds the diff of two files.
///
/// Constructs and holds a sequence of `Edit`.
pub struct Diff {
    edits: Vec<Edit>,
}

impl Diff {
    /// Create a diff from two files using the Myers' Diff Algorithm.
    pub fn from<S: AsRef<str>>(a: &[S], b: &[S]) -> Diff {
        let edits = algo::myers::from(a, b);

        Diff { edits }
    }

    /// Deserialize an edit script to create a diff
    pub fn from_edit_script<S: AsRef<str>>(edit_script: S) -> Result<Diff, Box<dyn Error>> {
        let mut remainder = edit_script.as_ref();
        let mut edits = Vec::new();

        while !remainder.is_empty() {
            let (r, e) = Edit::parse_edit_script(remainder)?;
            edits.push(e);
            remainder = r;
        }

        Ok(Diff { edits })
    }

    /// Serialize an 'edit script' for the diff.
    /// The changes in the edit script are thought to happen simultaneously.
    pub fn edit_script(&self) -> String {
        self.edits
            .iter()
            .map(|e| e.to_edit_script())
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Applies a series of edits to a file
    /// Goes line by line through the file to add edits in a tmp file,
    /// then overwriting the normal file with the tmp file.
    fn apply_edits(edits: &[Edit], file_path: &Path) -> Result<(), Box<dyn Error>> {
        // check if there are any edits
        if edits.is_empty() {
            return Ok(());
        }

        // open up the original file and the temp file which we are writing to
        let file = BufReader::new(File::open(file_path)?);

        // TODO: use NamedTempFile here
        let tmp_path = file_path.with_extension(".tmp");
        let mut tmp = BufWriter::new(File::create(&tmp_path)?);

        let mut edit_index = 0;
        let mut skipped_lines_left = 0;

        for (line_number, line) in file.lines().enumerate() {
            let line = line?;
            let edit = &edits[edit_index];

            // if previous edits had us delete this line, don't write it
            // and move to the next line
            if skipped_lines_left > 0 {
                skipped_lines_left -= 1;
                continue;
            }

            // check if there is an edit operating on this line.
            if edit.original.line == line_number {
                match edit.op {
                    Operation::Insert => {
                        // nothing to delete, only add the original line + inserted lines
                        tmp.write_all((line + "\n").as_bytes())?;
                        tmp.write_all((edit.modified.content.join("\n") + "\n").as_bytes())?;
                    }
                    Operation::Delete => {
                        // skip adding both this line and future lines.
                        // Subtract one because we are also not writing this line.
                        skipped_lines_left = edit.original.content.len() - 1;
                    }
                    Operation::Replace => {
                        // skip adding both this line and future lines, instead add inserted lines.
                        // Subtract one because we are also not writing this line.
                        skipped_lines_left = edit.original.content.len() - 1;
                        tmp.write_all((edit.modified.content.join("\n") + "\n").as_bytes())?;
                    }
                }
                edit_index += 1;
            } else {
                // write line to file
                tmp.write_all((line + "\n").as_bytes())?;
            }
        }

        // sometimes theres an insert edit left over, in which case we apply it. Also check for a few errors
        if edit_index == edits.len() - 1 && edits[edit_index].op == Operation::Insert {
            tmp.write_all((edits[edit_index].modified.content.join("\n") + "\n").as_bytes())?;
        } else if edit_index < edits.len() - 1 {
            return Err("Too many edits left over".into());
        } else if edit_index == edits.len() - 1 && edits[edit_index].op != Operation::Insert {
            return Err("Wrong edit type left over".into());
        }

        // drop the writer to the tmp file
        std::mem::drop(tmp);

        // overwrite the main file with the tmp file
        fs::rename(tmp_path, file_path)?;

        Ok(())
    }

    /// Apply a diff to a file
    pub fn apply(&self, file_path: &Path) -> Result<(), Box<dyn Error>> {
        Diff::apply_edits(&self.edits, file_path)
    }

    /// Rollback a diff on a file by applying the reverse diff
    pub fn rollback(&self, file_path: &Path) -> Result<(), Box<dyn Error>> {
        let mut rollback_edits = Vec::with_capacity(self.edits.len());

        // reverse the op and content for each edit.
        for edit in &self.edits {
            let new_op = match &edit.op {
                Operation::Insert => Operation::Delete,
                Operation::Delete => Operation::Insert,
                Operation::Replace => Operation::Replace,
            };

            rollback_edits.push(Edit {
                op: new_op,
                original: edit.modified.clone(),
                modified: edit.original.clone(),
            });
        }

        Diff::apply_edits(&rollback_edits, file_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::edit::HalfEdit;
    use tempfile::NamedTempFile;

    #[test]
    fn test_diff_apply() {
        const A: [&str; 8] = [
            "The small cactus sat in a",
            "pot full of sand and dirt",
            "",
            "Next to it was a small basil",
            "plant in a similar pot",
            "",
            "Everyday, the plants got plenty",
            "of sunshine and water",
        ];

        const B: [&str; 9] = [
            "The small green cactus sat in a",
            "pot full of sand and dirt",
            "",
            "In another part of the house,",
            "another house plant grew in a",
            "much bigger pot",
            "",
            "Everyday, the plants got plenty",
            "of water and sunshine",
        ];

        let diff = Diff::from(&A, &B);

        let mut f = NamedTempFile::new_in("./test_tmp_files").unwrap();
        write!(f, "{}", A.join("\n")).unwrap();

        let f_path = f.into_temp_path();

        diff.apply(&f_path).unwrap();

        let f = BufReader::new(File::open(&f_path).unwrap());
        let mut file_len = 0;

        for (index, line) in f.lines().enumerate() {
            assert_eq!(line.unwrap(), B[index]);
            file_len += 1;
        }

        assert_eq!(file_len, B.len());
    }

    #[test]
    fn test_diff_rollback() {
        const A: [&str; 8] = [
            "The small cactus sat in a",
            "pot full of sand and dirt",
            "",
            "Next to it was a small basil",
            "plant in a similar pot",
            "",
            "Everyday, the plants got plenty",
            "of sunshine and water",
        ];

        const B: [&str; 9] = [
            "The small green cactus sat in a",
            "pot full of sand and dirt",
            "",
            "In another part of the house,",
            "another house plant grew in a",
            "much bigger pot",
            "",
            "Everyday, the plants got plenty",
            "of water and sunshine",
        ];

        let diff = Diff::from(&A, &B);

        let mut f = NamedTempFile::new_in("./test_tmp_files").unwrap();
        write!(f, "{}", B.join("\n")).unwrap();

        let f_path = f.into_temp_path();

        diff.rollback(&f_path).unwrap();

        let f = BufReader::new(File::open(&f_path).unwrap());
        let mut file_len = 0;

        for (index, line) in f.lines().enumerate() {
            assert_eq!(line.unwrap(), A[index]);
            file_len += 1;
        }

        assert_eq!(file_len, A.len())
    }

    #[test]
    fn to_edit_script() {
        const A: [&str; 8] = [
            "The small cactus sat in a",
            "pot full of sand and dirt",
            "",
            "Next to it was a small basil",
            "plant in a similar pot",
            "",
            "Everyday, the plants got plenty",
            "of sunshine and water",
        ];

        const B: [&str; 9] = [
            "The small green cactus sat in a",
            "pot full of sand and dirt",
            "",
            "In another part of the house,",
            "another house plant grew in a",
            "much bigger pot",
            "",
            "Everyday, the plants got plenty",
            "of water and sunshine",
        ];

        let diff = Diff::from(&A, &B);

        let es = diff.edit_script();

        let expected_es = [
            "0,0r0,0",
            "< The small cactus sat in a",
            "---",
            "> The small green cactus sat in a",
            "3,4r3,5",
            "< Next to it was a small basil",
            "< plant in a similar pot",
            "---",
            "> In another part of the house,",
            "> another house plant grew in a",
            "> much bigger pot",
            "7,7r8,8",
            "< of sunshine and water",
            "---",
            "> of water and sunshine",
        ];

        assert_eq!(es, expected_es.join("\n"));
    }

    #[test]
    fn from_edit_script() {
        let es = [
            "0,0r0,0",
            "< The small cactus sat in a",
            "---",
            "> The small green cactus sat in a",
            "3,4r3,5",
            "< Next to it was a small basil",
            "< plant in a similar pot",
            "---",
            "> In another part of the house,",
            "> another house plant grew in a",
            "> much bigger pot",
            "7,7r8,8",
            "< of sunshine and water",
            "---",
            "> of water and sunshine",
        ];

        let diff = Diff::from_edit_script(&es.join("\n")).unwrap();

        assert_eq!(
            diff.edits,
            vec![
                Edit {
                    op: Operation::Replace,
                    original: HalfEdit {
                        line: 0,
                        content: vec!["The small cactus sat in a".to_string()]
                    },
                    modified: HalfEdit {
                        line: 0,
                        content: vec!["The small green cactus sat in a".to_string()]
                    }
                },
                Edit {
                    op: Operation::Replace,
                    original: HalfEdit {
                        line: 3,
                        content: vec![
                            "Next to it was a small basil".to_string(),
                            "plant in a similar pot".to_string()
                        ]
                    },
                    modified: HalfEdit {
                        line: 3,
                        content: vec![
                            "In another part of the house,".to_string(),
                            "another house plant grew in a".to_string(),
                            "much bigger pot".to_string()
                        ]
                    }
                },
                Edit {
                    op: Operation::Replace,
                    original: HalfEdit {
                        line: 7,
                        content: vec!["of sunshine and water".to_string()]
                    },
                    modified: HalfEdit {
                        line: 8,
                        content: vec!["of water and sunshine".to_string()]
                    }
                },
            ]
        );

        assert_eq!(diff.edits.len(), 3);
    }

    /*
    #[test]
    fn to_and_from_edit_script() {
        const A : [&str; 8] = ["The small cactus sat in a",
                 "pot full of sand and dirt",
                 "",
                 "Next to it was a small basil",
                 "plant in a similar pot",
                 "",
                 "Everyday, the plants got plenty",
                 "of sunshine and water"];

        const B : [&str; 9] = ["The small green cactus sat in a",
                 "pot full of sand and dirt",
                 "",
                 "In another part of the house,",
                 "another house plant grew in a",
                 "much bigger pot",
                 "",
                 "Everyday, the plants got plenty",
                 "of water and sunshine"];

        let diff = Diff::from(&A, &B);
        let edits = diff.edit_script();
        let edit_lines = edits.lines().collect::<Vec<&str>>();

        let second_diff = Diff::from_edit_script(&edit_lines).unwrap();

        assert_eq!(diff.edits, second_diff.edits);
    }
    */

    // ---------- A ------------
    // The small cactus sat in a
    // pot full of sand and dirt

    // Next to it was a small basil
    // plant in a similar pot

    // Everyday, the plants got plenty
    // of sunshine and water

    // ---------- B -------------
    // The small green cactus sat in a
    // pot full of sand and dirt

    // In another part of the house,
    // another house plant grew in a
    // much bigger pot

    // Everyday, the plants got plenty
    // of water and sunshine
}
