use crate::diff::edit::{Edit, Operation};

pub mod myers {
    use super::create_edits;
    use crate::diff::edit::Edit;

    /// Find the path traversed by a Shortest Edit Script
    /// Second part of the Myers' Diff Algorithm
    /// Single char names because it matches the paper
    #[allow(clippy::many_single_char_names)]
    fn find_path<V: AsRef<[usize]>>(
        trace: &[V],
        a_len: usize,
        b_len: usize,
    ) -> Vec<(usize, usize)> {
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
            let prev_index_k = if prev_k < 0 {
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

    /// A function to be used by the diff module to create a diff with the Myers
    /// Diff Algorithm
    pub fn from<S: AsRef<str>>(a: &[S], b: &[S]) -> Vec<Edit> {
        let trace = explore_paths(a, b);
        let path = find_path(&trace, a.len(), b.len());
        create_edits(&path, a, b)
    }

    #[cfg(test)]
    mod tests {
        use crate::diff::algo::myers;
        use crate::diff::edit::{Edit, HalfEdit, Operation};
        #[test]
        fn myers_creating_edits_replace_lines() {
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

            let edits = myers::from(&A, &B);

            assert_eq!(
                edits,
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
            )
        }

        #[test]
        fn myers_creating_edit_add_line() {
            const A: [&str; 2] = ["this is a line", "another line"];
            const B: [&str; 3] = ["this is a line", "new line!", "another line"];

            assert_eq!(
                myers::from(&A, &B),
                vec![Edit {
                    op: Operation::Insert,
                    original: HalfEdit {
                        line: 1,
                        content: vec![]
                    },
                    modified: HalfEdit {
                        line: 1,
                        content: vec!["new line!".to_string()]
                    }
                }]
            );
        }

        #[test]
        fn myers_creating_edits_add_multiple_lines() {
            const A: [&str; 2] = ["this is a line", "another line"];
            const B: [&str; 5] = [
                "this is a line",
                "new line!",
                "woah another",
                "another line",
                "one after",
            ];

            assert_eq!(
                myers::from(&A, &B),
                vec![
                    Edit {
                        op: Operation::Insert,
                        original: HalfEdit {
                            line: 1,
                            content: vec![]
                        },
                        modified: HalfEdit {
                            line: 1,
                            content: vec!["new line!".to_string(), "woah another".to_string()]
                        }
                    },
                    Edit {
                        op: Operation::Insert,
                        original: HalfEdit {
                            line: 2,
                            content: vec![]
                        },
                        modified: HalfEdit {
                            line: 4,
                            content: vec!["one after".to_string()]
                        }
                    }
                ]
            );
        }

        #[test]
        fn myers_creating_edit_delete_line() {
            const A: [&str; 3] = ["this is a line", "new line!", "another line"];
            const B: [&str; 2] = ["this is a line", "another line"];

            assert_eq!(
                myers::from(&A, &B),
                vec![Edit {
                    op: Operation::Delete,
                    original: HalfEdit {
                        line: 1,
                        content: vec!["new line!".to_string()]
                    },
                    modified: HalfEdit {
                        line: 1,
                        content: vec![]
                    }
                }]
            );
        }

        #[test]
        fn myers_creating_edit_delete_multiple_lines() {
            const A: [&str; 6] = [
                "this is a line",
                "new line!",
                "woah another",
                "another line",
                "one after",
                "and another!!",
            ];
            const B: [&str; 2] = ["this is a line", "another line"];

            assert_eq!(
                myers::from(&A, &B),
                vec![
                    Edit {
                        op: Operation::Delete,
                        original: HalfEdit {
                            line: 1,
                            content: vec!["new line!".to_string(), "woah another".to_string()]
                        },
                        modified: HalfEdit {
                            line: 1,
                            content: vec![]
                        }
                    },
                    Edit {
                        op: Operation::Delete,
                        original: HalfEdit {
                            line: 4,
                            content: vec!["one after".to_string(), "and another!!".to_string()]
                        },
                        modified: HalfEdit {
                            line: 2,
                            content: vec![]
                        }
                    }
                ]
            );
        }
    }
}

/// Creates a vector of `Edit`s given a path through the edit graph
/// Final part of the Myers' Diff Algorithm
#[allow(clippy::collapsible_if)]
fn create_edits<S: AsRef<str>>(path: &[(usize, usize)], a: &[S], b: &[S]) -> Vec<Edit> {
    let mut diff: Vec<Edit> = Vec::new();
    let mut chunk: Option<Edit> = None;
    let mut x = 0;
    let mut y = 0;

    // traverse the edit graph from start to finish
    // also means beginning of file to end
    for (prev_x, prev_y) in path {
        // a vertical move (no x change) means insert
        // horizontal move means delete
        // Diagonal move means same between files
        let edit_type = if x == *prev_x {
            Some(Operation::Insert)
        } else if y == *prev_y {
            Some(Operation::Delete)
        } else {
            None
        };

        match &edit_type {
            Some(edit_type) => {
                // constuct edit
                let original_content = if x != a.len() {
                    vec![a[x].as_ref().to_string()]
                } else {
                    vec![]
                };
                let modified_content = if y != b.len() {
                    vec![b[y].as_ref().to_string()]
                } else {
                    vec![]
                };

                let edit = Edit::new(edit_type.clone(), x, y, original_content, modified_content);

                match &mut chunk {
                    // add edit to chunk. The way the path is created, this should never fail.
                    Some(chunk) => chunk.join(edit).unwrap(),
                    // first edit of chunk, so set chunk
                    None => chunk = Some(edit),
                }
            }
            None => {
                if let Some(inner_chunk) = &chunk {
                    // add chunk to diff and reset chunk
                    diff.push(inner_chunk.clone());
                    chunk = None;
                }
            }
        }

        x = *prev_x;
        y = *prev_y;
    }

    if let Some(inner_chunk) = &chunk {
        // add chunk to diff and reset chunk
        diff.push(inner_chunk.clone());
    }

    diff
}
