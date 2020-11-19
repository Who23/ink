use crate::diff::parser;
use std::error::Error;

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
    /// Checks if two HalfEdits are joinable.
    fn joinable(&self, edit: &HalfEdit) -> bool {
        // checks whether the end point of one is the start point of the other
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
            Operation::Insert => Edit {
                op,
                original: HalfEdit {
                    line: x,
                    content: vec![],
                },
                modified: HalfEdit {
                    line: y,
                    content: modified_content,
                },
            },
            Operation::Delete => Edit {
                op,
                original: HalfEdit {
                    line: x,
                    content: original_content,
                },
                modified: HalfEdit {
                    line: y,
                    content: vec![],
                },
            },
            Operation::Replace => Edit {
                op,
                original: HalfEdit {
                    line: x,
                    content: original_content,
                },
                modified: HalfEdit {
                    line: y,
                    content: modified_content,
                },
            },
        }
    }

    /// Checks if two edits can be joined together
    fn joinable(&self, edit: &Edit) -> bool {
        self.original.joinable(&edit.original) && self.modified.joinable(&edit.modified)
    }

    /// Joins two edits, consuming the given edit
    // TODO: any way to consume the edit only if it's joinable, but leave it as a
    // ref otherwise so it's still usable?
    pub fn join(&mut self, edit: Edit) -> Result<(), &'static str> {
        if !self.joinable(&edit) {
            return Err("edits are not joinable");
        }

        // need to find the smaller line to add which should go first
        self.original.content.extend(edit.original.content);
        self.original.line = std::cmp::min(self.original.line, edit.original.line);

        self.modified.content.extend(edit.modified.content);
        self.modified.line = std::cmp::min(self.modified.line, edit.modified.line);

        // the only way an edit's op can change is from insert -> replace or delete -> replace
        // so just check for that.
        if !self.original.content.is_empty() && !self.modified.content.is_empty() {
            self.op = Operation::Replace
        }

        Ok(())
    }

    /// Creating an 'edit script' from a single edit,
    /// based on the UNIX diff utility's edit script,
    /// though this is not an 'ed' compatible edit script
    pub fn to_edit_script(&self) -> String {
        let op = match self.op {
            Operation::Insert => "a",
            Operation::Delete => "d",
            Operation::Replace => "r",
        };

        format!(
            "{},{}{}{},{}\n{}\n---\n{}",
            self.original.line,
            self.original.line + self.original.content.len() - 1,
            op,
            self.modified.line,
            self.modified.line + self.modified.content.len() - 1,
            // this just prepends a > or a < to every line
            self.original
                .content
                .iter()
                .map(|line| "< ".to_string() + line)
                .collect::<Vec<String>>()
                .join("\n"),
            self.modified
                .content
                .iter()
                .map(|line| "> ".to_string() + line)
                .collect::<Vec<String>>()
                .join("\n"),
        )
    }

    /// Parse an edit script into an Edit
    /// Takes an entire edit script as input, and if sucessful returns the remaining portion of
    /// the edit script along with the serialized edit
    pub fn parse_edit_script(script: &str) -> Result<(&str, Edit), Box<dyn Error>> {
        // parse out the line numbers from the original file
        let (r, og_line_start) = parser::read_usize(script)?;
        let r = parser::skip_sequence(r, ",")?;
        let (r, og_line_end) = parser::read_usize(r)?;

        // parse out the edit's operation
        let (r, op) = match r.chars().next().ok_or("No operation")? {
            'r' => Ok((&r[1..], Operation::Replace)),
            'a' => Ok((&r[1..], Operation::Insert)),
            'd' => Ok((&r[1..], Operation::Delete)),
            _ => Err("Invalid Operation"),
        }?;

        // parse out the line number from the modified file
        let (r, mod_line_start) = parser::read_usize(r)?;
        let r = parser::skip_sequence(r, ",")?;
        let (r, mod_line_end) = parser::read_usize(r)?;

        // parse out the content for each half of the edit
        let r = parser::skip_sequence(r, "\n")?;
        let (r, og_content_ref) = parser::read_lines(r, og_line_end - og_line_start + 1)?;
        let r = parser::skip_sequence(r, "---\n")?;
        let (r, mod_content_ref) = parser::read_lines(r, mod_line_end - mod_line_start + 1)?;

        // I couldn't find a better way to make a Vec<&str> -> Vec<String> while also stripping the '> '/'< ',
        // so here we are
        let mut og_content = Vec::with_capacity(og_content_ref.len());
        for line in og_content_ref {
            og_content.push(
                line.strip_prefix("< ")
                    .ok_or("Content formatted incorrectly")?
                    .to_string(),
            );
        }

        let mut mod_content = Vec::with_capacity(mod_content_ref.len());
        for line in mod_content_ref {
            mod_content.push(
                line.strip_prefix("> ")
                    .ok_or("Content formatted incorrectly")?
                    .to_string(),
            );
        }

        let edit = Edit {
            op,
            original: HalfEdit {
                line: og_line_start,
                content: og_content,
            },
            modified: HalfEdit {
                line: mod_line_start,
                content: mod_content,
            },
        };

        Ok((r, edit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edit_join_insert_and_delete() {
        let mut insert = Edit {
            op: Operation::Insert,
            original: HalfEdit {
                line: 0,
                content: vec![],
            },
            modified: HalfEdit {
                line: 0,
                content: vec!["boop".to_string()],
            },
        };

        let delete = Edit {
            op: Operation::Delete,
            original: HalfEdit {
                line: 0,
                content: vec!["bap".to_string()],
            },
            modified: HalfEdit {
                line: 0,
                content: vec![],
            },
        };

        insert.join(delete).unwrap();

        assert_eq!(
            insert,
            Edit {
                op: Operation::Replace,
                original: HalfEdit {
                    line: 0,
                    content: vec!["bap".to_string()]
                },
                modified: HalfEdit {
                    line: 0,
                    content: vec!["boop".to_string()]
                },
            }
        )
    }

    #[test]
    fn edit_join_insert_and_insert() {
        let mut insert = Edit {
            op: Operation::Insert,
            original: HalfEdit {
                line: 0,
                content: vec![],
            },
            modified: HalfEdit {
                line: 0,
                content: vec!["boop".to_string()],
            },
        };

        let second_insert = Edit {
            op: Operation::Delete,
            original: HalfEdit {
                line: 0,
                content: vec![],
            },
            modified: HalfEdit {
                line: 1,
                content: vec!["bap".to_string()],
            },
        };

        insert.join(second_insert).unwrap();

        assert_eq!(
            insert,
            Edit {
                op: Operation::Insert,
                original: HalfEdit {
                    line: 0,
                    content: vec![]
                },
                modified: HalfEdit {
                    line: 0,
                    content: vec!["boop".to_string(), "bap".to_string()]
                },
            }
        )
    }
}
