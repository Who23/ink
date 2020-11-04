//! Tools for creating diffs, done through the `Diff` struct
mod algo;
mod edit;

use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
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
    pub fn from_edit_script<S: AsRef<str>>(edit_script: &[S]) -> Result<Diff, Box<dyn Error>> {
        unimplemented!();
        /*
        let mut edits = Vec::new();

        // store what will be in the edit through the loop
        let mut line_start = 0;
        let mut line_end = 0;
        let mut content = String::new();
        let mut content_lines = 0;
        let mut operation = EditOp::Insert;

        for line in edit_script {
            // if we're on a content line, add it to the buffer
            if content_lines > 0 {
                content_lines -= 1;
                content.push_str(line.as_ref());
                content.push_str("\n");
                continue;
            }

            // means we have gotten through the lines of content
            // (last part of the edit). add the edit to the vector
            if !content.is_empty() {
                edits.push(Edit {
                    operation,
                    line_start,
                    line_end,
                    content: content.clone(),
                });
                content.clear();
            }

            // get the starting line #, ending line #, & type of edit
            let components: Vec<&str> = line.as_ref().split(',').collect();
            line_start = components[0].parse::<usize>()?;
            line_end = components[1].parse::<usize>()?;
            operation = match components[2] {
                "a" => Ok(EditOp::Insert),
                "d" => Ok(EditOp::Delete),
                _ => Err("invalid edit script"),
            }?;

            // find how many lines of content there will be
            content_lines = line_end - line_start + 1;
        }

        // add the last edit
        edits.push(Edit {
            operation,
            line_start,
            line_end,
            content,
        });

        Ok(Diff { edits })
        */
    }

    /// Serialize an 'edit script' for the diff.
    /// The changes in the edit script are thought to happen simultaneously.
    pub fn edit_script(&self) -> String {
        self.edits
            .iter()
            .map(|e| e.to_edit_script())
            .collect::<Vec<String>>()
            .join("")
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
                    },
                    Operation::Delete => {
                        // skip adding both this line and future lines. 
                        // Subtract one because we are also not writing this line.
                        skipped_lines_left = edit.original.content.len() - 1;
                    },
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

        if edit_index == edits.len() - 1 && edits[edit_index].op == Operation::Insert {
            tmp.write_all((edits[edit_index].modified.content.join("\n") + "\n").as_bytes())?;

        } else if edit_index < edits.len() - 1 {
            return Err("Too many edits left over".into())

        } else if edit_index == edits.len() - 1 && edits[edit_index].op != Operation::Insert {
            return Err("Wrong edit type left over".into())
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

        for edit in &self.edits {
            let new_op = match &edit.op {
                Operation::Insert => Operation::Delete,
                Operation::Delete => Operation::Insert,
                Operation::Replace => Operation::Replace
            };

            rollback_edits.push(Edit {
                op: new_op,
                original: edit.modified.clone(),
                modified: edit.original.clone()
            });
        }

        Diff::apply_edits(&rollback_edits, file_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_diff_apply() {
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

        let mut f = NamedTempFile::new_in("./test_tmp_files").unwrap();
        write!(f, "{}", A.join("\n"));

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

        let mut f = NamedTempFile::new_in("./test_tmp_files").unwrap();
        write!(f, "{}", B.join("\n"));

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
        /*
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

        let es = diff.edit_script();

        let expected_es = ["0,0,d",
                           "The small cactus sat in a",
                           "1,1,a",
                           "The small green cactus sat in a",
                           "3,4,d",
                           "Next to it was a small basil",
                           "plant in a similar pot",
                           "5,7,a",
                           "In another part of the house,",
                           "another house plant grew in a",
                           "much bigger pot",
                           "7,7,d",
                           "of sunshine and water",
                           "8,8,a",
                           "of water and sunshine",
                           ""];

        assert_eq!(es, expected_es.join("\n"));
        */
        panic!();
    }

    #[test]
    fn from_edit_script() {
        /*
        let es = ["0,0,d",
                 "The small cactus sat in a",
                 "1,1,a",
                 "The small green cactus sat in a",
                 "3,4,d",
                 "Next to it was a small basil",
                 "plant in a similar pot",
                 "5,7,a",
                 "In another part of the house,",
                 "another house plant grew in a",
                 "much bigger pot",
                 "7,7,d",
                 "of sunshine and water",
                 "8,8,a",
                 "of water and sunshine"];

        let diff = Diff::from_edit_script(&es).unwrap();

        let expected_edits = [
            Edit {
                operation: EditOp::Delete,
                line_start: 0,
                line_end: 0,
                content: String::from("The small cactus sat in a\n")
            },
            Edit {
                operation: EditOp::Insert,
                line_start: 1,
                line_end: 1,
                content: String::from("The small green cactus sat in a\n")
            },
            Edit {
                operation: EditOp::Delete,
                line_start: 3,
                line_end: 4,
                content: String::from("Next to it was a small basil\nplant in a similar pot\n")
            },
            Edit {
                operation: EditOp::Insert,
                line_start: 5,
                line_end: 7,
                content: String::from("In another part of the house,\nanother house plant grew in a\nmuch bigger pot\n")
            },
            Edit {
                operation: EditOp::Delete,
                line_start: 7,
                line_end: 7,
                content: String::from("of sunshine and water\n")
            },
            Edit {
                operation: EditOp::Insert,
                line_start: 8,
                line_end: 8,
                content: String::from("of water and sunshine\n")
            },
        ];

        for (index, edit) in diff.edits.iter().enumerate() {
            assert_eq!(edit, &expected_edits[index]);
        }

        assert_eq!(diff.edits.len(), expected_edits.len());
        */
        panic!();
    }

    #[test]
    fn to_and_from_edit_script() {
        /*
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
        */
        panic!();
    }

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
