//! Tools for creating diffs, done through the `Diff` struct

/// The type of edit - Insertion or Deletion
#[derive(PartialEq)]
enum EditOp {
    Insert,
    Delete
}

/// One section of a diff which involves adding or removing one
/// or more lines. 
///
/// This should only be created via the `Diff` struct
pub struct Edit {
    operation: EditOp,
    line_start: usize,
    line_end: usize,
    content: String
}

impl Edit {
    /// Creating an 'edit script' from a single edit,
    /// based on the UNIX diff utility's edit script,
    /// though this is not an 'ed' compatible edit script
    fn to_edit_script(&self) -> String {
        let op = match self.operation {
            EditOp::Insert => "a",
            EditOp::Delete => "d"
        };

        format!("{},{}{}\n{}.\n", self.line_start, self.line_end, op, self.content)
    }
}

/// Struct that holds the diff of two files. 
///
/// Constructs and holds a sequence of `Edit`.
pub struct Diff {
    pub edits: Vec<Edit>
}

impl Diff {
    /// Create a diff from two files using the Myers' Diff Algorithm.
    /// Currently this not efficient in terms of space.
    /// This will probably be replaced by it's linear space version later.
    pub fn from(a: Vec<String>, b: Vec<String>) -> Diff {

        let trace = explore_paths(&a, &b);
        let path = find_path(trace, a.len(), b.len());
        let edits = create_edits(path, a, b);
        
        Diff {
            edits
        }
    }

    /// Create an 'edit script' for the diff.
    /// The changes in the edit script are thought to happen simultaneously.
    pub fn edit_script(&self) -> String {
        self.edits.iter()
                  .map(|e| e.to_edit_script())
                  .collect::<Vec<String>>()
                  .join("")
    }
}

/// Creates a vector of `Edit`s given a path through the edit graph
/// Final part of the Myers' Diff Algorithm
fn create_edits(path: Vec<(usize, usize)>, a: Vec<String>, b: Vec<String>) -> Vec<Edit> {
    let mut diff: Vec<Edit> = Vec::new();

    let mut x = 0;
    let mut y = 0;

    // traverse the edit graph from start to finish
    // also means beginning of file to end
    for (prev_x, prev_y) in path {
        // a vertical move (no x change) means insert
        // horizontal move means delete
        // Diagonal move means same between files
        let edit_type = if x == prev_x {
            Some(EditOp::Insert)

        } else if y == prev_y {
            Some(EditOp::Delete)

        } else { None };

        if let Some(edit_type) = edit_type {
            // get what we are inserting/deleting and where
            // insert --> coming from the 2nd string, opposite for delete
            let (line_idx, lines) = match edit_type {
                EditOp::Insert => { (y, &b[y]) },
                EditOp::Delete => { (x, &a[x]) }
            };

            // If the last edit was of the same type, expand that edit
            // instead of creaing a new one. Otherwise add the new edit to the diff
            if let Some(edit) = diff.last_mut() {
                if edit.operation == edit_type {
                    edit.line_end = line_idx;
                    edit.content.push_str(lines);
                    edit.content.push_str("\n");
                } else {
                    diff.push(Edit {
                        operation: edit_type,
                        line_start: line_idx,
                        line_end: line_idx,
                        content: (*lines).clone() + "\n"
                    });
                }
            } else {
                diff.push(Edit {
                    operation: edit_type,
                    line_start: line_idx,
                    line_end: line_idx,
                    content: (*lines).clone() + "\n"
                });
            }
        }

        x = prev_x;
        y = prev_y;
    }

    diff
}

/// Find the path traversed by a Shortest Edit Script
/// Second part of the Myers' Diff Algorithm
fn find_path(trace: Vec<Vec<usize>>, a_len: usize, b_len: usize) -> Vec<(usize, usize)> {
    let max = a_len + b_len;

    let mut x = a_len as isize;
    let mut y = b_len as isize;
    let mut path = vec![];

    // work our way from the end point to the start point
    for (d, v) in trace.iter().enumerate().rev() {
        let d = d as isize;

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
        }
        else {
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
fn explore_paths(a: &Vec<String>, b: &Vec<String>) -> Vec<Vec<usize>> {
    let (n, m) = (a.len(), b.len());
    let max = n + m;
    let mut v = vec![0; 2 * max + 1];
    let mut t: Vec<Vec<usize>> = vec![];

    // for d = 0, we need a starting point at k = 1, (x, y) = (0, -1)
    v[max + 1] = 0;

    for d in 0..=max {
        // usually k would be iterating from -d <-> d. But isize is a pain here,
        // so it maps to 0 <-> 2d
        for k in (0..=(2*d)).step_by(2) {
            // k is used as an index in the algorithm
            // but since k is mapped to 0 <-> 2d it's a problem, so this is a unique index
            let index_k = (max - d) + k;
            let mut x = if (k == 0) || (k != 2*d && v[index_k - 1] < v[index_k + 1]) {
                v[index_k + 1]
            }
            else {
                v[index_k - 1] + 1
            };

            // this is `x - (k - d)` rewritten, since that requires
            // calculating a negative number
            let mut y = x + d - k;

            // going along a diagonal
            while x < n && y < m && a[x] == b[y] {
                x += 1;
                y += 1;
            }

            // add the farthest you could go on this depth
            v[index_k] = x;

            // we have reached the end point!
            if x >= n && y >= m {
                t.push(v.clone());
                return t;
            }
        }
        t.push(v.clone());
    }

    t
}
