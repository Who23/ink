/// The type of edit - Insertion, Deletion, or Replacement
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Operation {
    Insert,
    Delete,
    Replace,
}

/// Half of an edit, that can refer to the original file
/// or the modified file. Should only be constructed with an Edit.
/// The first line is line 0, and the last line is line len - 1
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HalfEdit {
    pub line: usize,
    pub content: Vec<String>,
}

impl HalfEdit {
    fn joinable(&self, edit: &HalfEdit) -> bool {
        self.line + self.content.len() == edit.line || edit.line + edit.content.len() == self.line
    }
}

/// One section of a diff which involves adding or removing, or replacing
/// or more lines.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Edit {
    pub op: Operation,
    pub original: HalfEdit,
    pub modified: HalfEdit,
}

impl Edit {
    /// create an edit given an op, line numbers, and content
    // TODO: error check that for insert and delete edit, modified/original is exclusive, just for
    // error keeping sake.
    pub fn new(
        op: Operation,
        x: usize,
        y: usize,
        original_content: Vec<String>,
        modified_content: Vec<String>,
    ) -> Edit {
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

    /// 
    fn joinable(&self, edit: &Edit) -> bool {
        self.original.joinable(&edit.original) && self.modified.joinable(&edit.modified)
    }

    pub fn join(&mut self, edit: Edit) -> Result<(), &'static str> {
        if !self.joinable(&edit) {
            return Err("edits are not joinable")
        }

        self.original.content.extend(edit.original.content);
        self.original.line = std::cmp::min(self.original.line, edit.original.line);

        self.modified.content.extend(edit.modified.content);
        self.modified.line = std::cmp::min(self.modified.line, edit.modified.line);

        if !self.original.content.is_empty() && !self.modified.content.is_empty() {
            self.op = Operation::Replace
        }

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edit_join_insert_and_delete() {
        let mut insert = Edit {
            op: Operation::Insert,
            original: HalfEdit { line: 0, content: vec![] },
            modified: HalfEdit { line: 0, content: vec!["boop".to_string()] },
        };

        let delete = Edit {
            op: Operation::Delete,
            original: HalfEdit { line: 0, content: vec!["bap".to_string()] },
            modified: HalfEdit { line: 0, content: vec![] },
        };

        insert.join(delete).unwrap();

        assert_eq!(
            insert,
            Edit {
                op: Operation::Replace,
                original: HalfEdit { line: 0, content: vec!["bap".to_string()] },
                modified: HalfEdit { line: 0, content: vec!["boop".to_string()] },
            }
        )
    }

    #[test]
    fn edit_join_insert_and_insert() {
        let mut insert = Edit {
            op: Operation::Insert,
            original: HalfEdit { line: 0, content: vec![] },
            modified: HalfEdit { line: 0, content: vec!["boop".to_string()] },
        };

        let second_insert = Edit {
            op: Operation::Delete,
            original: HalfEdit { line: 0, content: vec![] },
            modified: HalfEdit { line: 1, content: vec!["bap".to_string()] },
        };

        insert.join(second_insert).unwrap();

        assert_eq!(
            insert,
            Edit {
                op: Operation::Insert,
                original: HalfEdit { line: 0, content: vec![] },
                modified: HalfEdit { line: 0, content: vec!["boop".to_string(), "bap".to_string()] },
            }
        )
    }
}
