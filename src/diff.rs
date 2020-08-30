//! Tools for creating diffs, done through the `Diff` struct
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

/// The type of edit - Insertion or Deletion
#[derive(PartialEq, Eq, Debug)]
enum EditOp {
    Insert,
    Delete,
}

/// One section of a diff which involves adding or removing one
/// or more lines.
///
/// This should only be created via the `Diff` struct
#[derive(Debug, PartialEq, Eq)]
pub struct Edit {
    operation: EditOp,
    line_start: usize,
    line_end: usize,
    content: String,
}

impl Edit {
    /// Creating an 'edit script' from a single edit,
    /// based on the UNIX diff utility's edit script,
    /// though this is not an 'ed' compatible edit script
    fn to_edit_script(&self) -> String {
        let op = match self.operation {
            EditOp::Insert => "a",
            EditOp::Delete => "d",
        };

        format!(
            "{},{},{}\n{}",
            self.line_start, self.line_end, op, self.content
        )
    }
}

/// Struct that holds the diff of two files.
///
/// Constructs and holds a sequence of `Edit`.
pub struct Diff {
    pub edits: Vec<Edit>,
}

impl Diff {
    /// Create a diff from two files using the Myers' Diff Algorithm.
    pub fn from<S: AsRef<str>>(a: &[S], b: &[S]) -> Diff {
        let trace = explore_paths(a, b);
        let path = find_path(&trace, a.len(), b.len());
        let edits = create_edits(&path, a, b);

        Diff { edits }
    }

    /// Deserialize an edit script to create a diff
    pub fn from_edit_script<S: AsRef<str>>(edit_script: &[S]) -> Result<Diff, Box<dyn Error>> {
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
    fn apply_edits(edits: &[Edit], file_path: &Path) -> io::Result<()> {
        let tmp_path = file_path.with_extension(".tmp");

        // check if there are any edits
        if edits.is_empty() {
            return Ok(());
        }

        // open up the file to read and a tmp file to write to
        let file = BufReader::new(File::open(file_path)?);
        let mut tmp = BufWriter::new(File::create(&tmp_path)?);

        let mut lines_to_delete = 0; // if deleting lines, how many are left to delete
        let mut current_edit_index = 0; // current edit we're on
        let mut current_edit: &Edit;
        let mut last_edit = &edits[current_edit_index]; // the previous edit
        let mut next_edit: &Edit; // the next edit

        for (line_number, line) in file.lines().enumerate() {
            let line = line?;

            // firstly, if there are lines to delete, 'delete'
            // the line by not writing it to the tmp file & going to the next
            if lines_to_delete > 0 {
                lines_to_delete -= 1;
                continue;
            }

            current_edit = &edits[current_edit_index];

            // check if this line has an edit on it (edits are always ordered by line #)
            if line_number == current_edit.line_start {
                match current_edit.operation {
                    EditOp::Insert => {
                        // write the inserted lines into the tmp file
                        tmp.write_all(current_edit.content.as_bytes())?;

                        // decides whether to write the current line of the normal file into the tmp file
                        // if this is not the last edit & the next edit would delete this line
                        // don't write it
                        if current_edit_index + 1 != edits.len() {
                            next_edit = &edits[current_edit_index + 1];

                            if !(next_edit.operation == EditOp::Delete
                                && current_edit.line_end + 1 == next_edit.line_start)
                            {
                                tmp.write_all((line + "\n").as_bytes())?;
                            }
                        } else {
                            tmp.write_all((line + "\n").as_bytes())?;
                        }

                        last_edit = current_edit;
                    }
                    EditOp::Delete => {
                        // how many lines we should be deleting? if the end == the start, then we
                        // only delete this line (sets to 0)
                        lines_to_delete = current_edit.line_end - current_edit.line_start;

                        // checks if we write the current line into the tmp file
                        // checks if the last edit was an insert that already deleted the line that
                        // we needed to delete, so we should write this next line into the tmp file
                        if last_edit.operation == EditOp::Insert
                            && last_edit.line_end + 1 == current_edit.line_start
                        {
                            tmp.write_all((line + "\n").as_bytes())?;

                            if lines_to_delete > 0 {
                                lines_to_delete -= 1;
                            }
                        }

                        last_edit = current_edit;
                    }
                }

                current_edit_index += 1;
            } else {
                // if there wasn't an edit, write the current line
                tmp.write_all((line + "\n").as_bytes())?;
            }
        }

        // there might be a insert edit left over.
        // Add that to the tmp file
        if current_edit_index == edits.len() - 1 {
            current_edit = &edits[current_edit_index];
            if current_edit.operation == EditOp::Insert {
                tmp.write_all(current_edit.content.as_bytes())?;
            } else {
                panic!("the last one is a delete??");
            }
        } else if current_edit_index != edits.len() {
            panic!("edits left after??");
        }

        // drop the writer to the tmp file
        std::mem::drop(tmp);

        // overwrite the main file with the tmp file
        fs::rename(tmp_path, file_path)?;

        Ok(())
    }

    /// Apply a diff to a file
    pub fn apply(&self, file_path: &Path) -> io::Result<()> {
        Diff::apply_edits(&self.edits, file_path)
    }

    /// Rollback a diff on a file by applying the reverse diff
    pub fn rollback(&self, file_path: &Path) -> io::Result<()> {
        let mut rollback_edits = vec![];

        // the line offset, because inserting/deleting line numbers are based on
        // the unmodified file.
        let mut offset: isize = 0;

        for edit in &self.edits {
            let new_edit = match edit.operation {
                // change the insert to a delete, using the length of the content
                // for the line numbers while adding the offset as well.
                EditOp::Insert => {
                    let num_lines = edit.content.lines().count();

                    let line_start = if offset < 0 {
                        edit.line_start - offset.abs() as usize
                    } else {
                        edit.line_start + offset.abs() as usize
                    };

                    if num_lines != 1 {
                        offset += num_lines as isize;
                    }

                    Edit {
                        operation: EditOp::Delete,
                        line_start,
                        line_end: num_lines + line_start - 1,
                        content: edit.content.clone(),
                    }
                }
                // same as above for the delete
                EditOp::Delete => {
                    let num_lines = edit.content.lines().count();

                    let line_start = if offset < 0 {
                        edit.line_start - offset.abs() as usize
                    } else {
                        edit.line_start + offset.abs() as usize
                    };

                    if num_lines != 1 {
                        offset -= num_lines as isize;
                    }

                    Edit {
                        operation: EditOp::Insert,
                        line_start,
                        line_end: line_start + num_lines - 1,
                        content: edit.content.clone(),
                    }
                }
            };

            rollback_edits.push(new_edit);
        }

        Diff::apply_edits(&rollback_edits, file_path)
    }
}

/// Creates a vector of `Edit`s given a path through the edit graph
/// Final part of the Myers' Diff Algorithm
fn create_edits<S: AsRef<str>>(path: &[(usize, usize)], a: &[S], b: &[S]) -> Vec<Edit> {
    let mut diff: Vec<Edit> = Vec::new();

    let mut x = 0;
    let mut y = 0;

    // traverse the edit graph from start to finish
    // also means beginning of file to end
    for (prev_x, prev_y) in path {
        // a vertical move (no x change) means insert
        // horizontal move means delete
        // Diagonal move means same between files
        let edit_type = if x == *prev_x {
            Some(EditOp::Insert)
        } else if y == *prev_y {
            Some(EditOp::Delete)
        } else {
            None
        };

        if let Some(edit_type) = edit_type {
            // get what we are inserting/deleting and where
            // insert --> coming from the 2nd string, opposite for delete
            // the line number is always from the first file
            let (line_idx, lines) = match edit_type {
                EditOp::Insert => (x, &b[y]),
                EditOp::Delete => (x, &a[x]),
            };

            // If the last edit was of the same type, expand that edit
            // instead of creaing a new one. Otherwise add the new edit to the diff
            if let Some(edit) = diff.last_mut() {
                if edit.operation == edit_type && (edit.line_end + 1) >= line_idx {
                    edit.line_end += 1;
                    edit.content.push_str(lines.as_ref());
                    edit.content.push_str("\n");
                } else {
                    diff.push(Edit {
                        operation: edit_type,
                        line_start: line_idx,
                        line_end: line_idx,
                        content: String::from(lines.as_ref()) + "\n"
                    });
                }
            } else {
                diff.push(Edit {
                    operation: edit_type,
                    line_start: line_idx,
                    line_end: line_idx,
                    content: String::from(lines.as_ref()) + "\n"
                });
            }
        }

        x = *prev_x;
        y = *prev_y;
    }

    diff
}

/// Find the path traversed by a Shortest Edit Script
/// Second part of the Myers' Diff Algorithm
/// Single char names because it matches the paper
#[allow(clippy::many_single_char_names)]
fn find_path<V: AsRef<[usize]>>(trace: &[V], a_len: usize, b_len: usize) -> Vec<(usize, usize)> {
    let max = a_len + b_len;

    let mut x = a_len as isize;
    let mut y = b_len as isize;
    let mut path = vec![];

    // work our way from the end point to the start point
    for (d, v) in trace.iter().enumerate().rev() {
        let d = d as isize;
        let v = v.as_ref();

        let k = x - y;

        // This just ofsets max by k, because we can't index
        // a Vec with a negative number. Mapping from -max <-> max
        // to 0 <-> 2*max + 1
        let index_k = if k < 0 {
            max - k.abs() as usize
        } else {
            max + k.abs() as usize
        };

        // Which point in the edit graph do we backtrack to?
        // same logic as traversing it
        let prev_k = if (k == -d) || (k != d && v[index_k - 1] < v[index_k + 1]) {
            k + 1
        } else {
            k - 1
        };

        // Calculate all the same values for that backtracked point
        let prev_index_k = if k < 0 {
            max - prev_k.abs() as usize
        } else {
            max + prev_k.abs() as usize
        };
        let prev_x = v[prev_index_k] as isize;
        let prev_y = prev_x - prev_k;

        // Move along a diagonal if we can, adding points we traversed
        // to the path
        while x > prev_x && y > prev_y {
            path.push((x as usize, y as usize));
            x -= 1;
            y -= 1;
        }

        // push the final point
        if d > 0 {
            path.push((x as usize, y as usize));
        }

        x = prev_x;
        y = prev_y;
    }

    path.reverse();
    path
}

/// Explores possible paths in the edit graph, to find a possible SES,
/// Filling in the trace along the way
/// First part of the Myers' Diff Algorithm
/// Single char names because it matches the paper
#[allow(clippy::many_single_char_names)]
fn explore_paths<S: AsRef<str>>(a: &[S], b: &[S]) -> Vec<Vec<usize>> {
    let (n, m) = (a.len(), b.len());
    let max = n + m;
    let mut v = vec![0; 2 * max + 1];
    let mut t: Vec<Vec<usize>> = vec![];

    // for d = 0, we need a starting point at k = 1, (x, y) = (0, -1)
    v[max + 1] = 0;

    for d in 0..=max {
        // usually k would be iterating from -d <-> d. But isize is a pain here,
        // so it maps to 0 <-> 2d
        for k in (0..=(2 * d)).step_by(2) {
            // k is used as an index in the algorithm
            // but since k is mapped to 0 <-> 2d it's a problem, so this is a unique index
            let index_k = (max - d) + k;
            let mut x = if (k == 0) || (k != 2 * d && v[index_k - 1] < v[index_k + 1]) {
                v[index_k + 1]
            } else {
                v[index_k - 1] + 1
            };

            // this is `x - (k - d)` rewritten, since that requires
            // calculating a negative number
            let mut y = x + d - k;

            // going along a diagonal
            while x < n && y < m && a[x].as_ref() == b[y].as_ref() {
                x += 1;
                y += 1;
            }

            // add the farthest you could go on this depth
            v[index_k] = x;

            // we have reached the end point!
            if x >= n && y >= m {
                t.push(v);
                return t;
            }
        }

        t.push(v.clone());
    }

    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{SeekFrom, Seek, BufRead, BufReader, self};
    use tempfile::NamedTempFile;

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

    #[test]
    fn creating_diff_add_line() {
        const A: [&str; 2] = ["this is a line", "another line"];
        const B: [&str; 3] = ["this is a line", "new line!", "another line"];

        let diff = Diff::from(&A, &B);

        let mut expected_edits = Vec::new();
        
        expected_edits.push(Edit {
            content: "new line!\n".to_string(),
            line_start: 1,
            line_end: 1,
            operation: EditOp::Insert
        });

        assert_eq!(diff.edits, expected_edits);
    }

    #[test]
    fn creating_diff_add_multiple_lines() {
        const A: [&str; 2] = ["this is a line", "another line"];
        const B: [&str; 5] = ["this is a line", "new line!", "woah another", "another line", "one after"];

        let diff = Diff::from(&A, &B);

        let mut expected_edits = Vec::new();

        expected_edits.push(Edit {
            content: "new line!\nwoah another\n".to_string(),
            line_start: 1,
            line_end: 2,
            operation: EditOp::Insert
        });

        expected_edits.push(Edit {
            content: "one after\n".to_string(),
            line_start: 2,
            line_end: 2,
            operation: EditOp::Insert
        });

        assert_eq!(diff.edits, expected_edits);
    }

    #[test]
    fn creating_diff_delete_line() {
        const A: [&str; 3] = ["this is a line", "new line!", "another line"];
        const B: [&str; 2] = ["this is a line", "another line"];

        let diff = Diff::from(&A, &B);

        let mut expected_edits = Vec::new();
        
        expected_edits.push(Edit {
            content: "new line!\n".to_string(),
            line_start: 1,
            line_end: 1,
            operation: EditOp::Delete
        });

        assert_eq!(diff.edits, expected_edits);
    }

    #[test]
    fn creating_diff_delete_multiple_lines() {
        const A: [&str; 6] = ["this is a line", "new line!", "woah another", "another line", "one after", "and another!!"];
        const B: [&str; 2] = ["this is a line", "another line"];

        let diff = Diff::from(&A, &B);

        let mut expected_edits = Vec::new();

        expected_edits.push(Edit {
            content: "new line!\nwoah another\n".to_string(),
            line_start: 1,
            line_end: 2,
            operation: EditOp::Delete
        });

        expected_edits.push(Edit {
            content: "one after\nand another!!\n".to_string(),
            line_start: 4,
            line_end: 5,
            operation: EditOp::Delete
        });

        assert_eq!(diff.edits, expected_edits);
    }

    #[test]
    fn test_diff_apply() {
        let A = ["The small cactus sat in a",
                 "pot full of sand and dirt",
                 "",
                 "Next to it was a small basil",
                 "plant in a similar pot",
                 "",
                 "Everyday, the plants got plenty",
                 "of sunshine and water"];

        let B = ["The small green cactus sat in a",
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
        let A = ["The small cactus sat in a",
                 "pot full of sand and dirt",
                 "",
                 "Next to it was a small basil",
                 "plant in a similar pot",
                 "",
                 "Everyday, the plants got plenty",
                 "of sunshine and water"];

        let B = ["The small green cactus sat in a",
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
        let A = ["The small cactus sat in a",
                 "pot full of sand and dirt",
                 "",
                 "Next to it was a small basil",
                 "plant in a similar pot",
                 "",
                 "Everyday, the plants got plenty",
                 "of sunshine and water"];

        let B = ["The small green cactus sat in a",
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
    }

    #[test]
    fn from_edit_script() {
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
    }

    #[test]
    fn to_and_from_edit_script() {
        let A = ["The small cactus sat in a",
                 "pot full of sand and dirt",
                 "",
                 "Next to it was a small basil",
                 "plant in a similar pot",
                 "",
                 "Everyday, the plants got plenty",
                 "of sunshine and water"];

        let B = ["The small green cactus sat in a",
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
}