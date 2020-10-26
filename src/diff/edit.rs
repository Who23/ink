/// The type of edit - Insertion, Deletion, or Replacement
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Operation {
    Insert,
    Delete,
    Replace
}

/// Half of an edit, that can refer to the original file
/// or the modified file. Should only be constructed with an Edit.
/// The first line is line 0, and the last line is line len - 1
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HalfEdit {
    pub line: usize,
    pub content: Vec<String>
}

/// One section of a diff which involves adding or removing, or replacing
/// or more lines.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Edit {
    pub op: Operation,
    pub original: HalfEdit,
    pub modified: HalfEdit
}

impl Edit {
    /// create an edit given an op, line numbers, and content
    // TODO: error check that for insert and delete edit, modified/original is exclusive, just for
    // error keeping sake.
    pub fn new(op: Operation, x: usize, y: usize, original_content: Vec<String>, modified_content: Vec<String>) -> Edit {
        match op {
            Operation::Insert => {
                Edit {
                    op,
                    original: HalfEdit { line: x, content: vec![] },
                    modified: HalfEdit { line: y, content: modified_content }
                }
            },
            Operation::Delete => {
                Edit {
                    op,
                    original: HalfEdit { line: x, content: original_content },
                    modified: HalfEdit { line: y, content: vec![] }
                }
            }
            Operation::Replace => {
                Edit {
                    op,
                    original: HalfEdit { line: x, content: original_content },
                    modified: HalfEdit { line: y, content: modified_content }
                }
            }
        }
    }

    pub fn join(&mut self, edit: Edit) {
        self.original.content.extend(edit.original.content);
        self.modified.content.extend(edit.modified.content);

        if self.original.content.len() > 0 && self.modified.content.len() > 0 {
            self.op = Operation::Replace
        }
    }


    /// Creating an 'edit script' from a single edit,
    /// based on the UNIX diff utility's edit script,
    /// though this is not an 'ed' compatible edit script
    pub fn to_edit_script(&self) -> String {
        unimplemented!();
        /*
        let op = match self.operation {
            EditOp::Insert => "a",
            EditOp::Delete => "d",
        };

        format!(
            "{},{},{}\n{}",
            self.line_start, self.line_end, op, self.content
        )
        */
    }
}