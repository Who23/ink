fn main() {
    let a = "ABBA";
    let b = "ABCA";
    println!("{}\n{}", a, b);
    println!("{}", diff(a, b));
}

fn diff(a: &str, b: &str) -> String {
    let trace = explore_paths(a.as_bytes(), b.as_bytes());
    let path = find_path(trace, a.len(), b.len());
    let script = create_edit_script(path, a.as_bytes(), b.as_bytes());
    script
}

fn create_edit_script(path: Vec<(usize, usize)>, a: &[u8], b: &[u8]) -> String {
    let mut diff = String::new();
    
    let mut prev_x = a.len();
    let mut prev_y = b.len();

    for (x, y) in path {
        if x == prev_x {
            let deletion = format!("\u{001b}[32m+ {}\u{001b}[0m\n", b[y] as char);
            diff.push_str(&deletion);

        } else if y == prev_y {
            let addition = format!("\u{001b}[31m- {}\u{001b}[0m\n", a[x] as char);
            diff.push_str(&addition);

        } else {
            let same = format!("  {}\n", a[x] as char);
            diff.push_str(&same);
        }
        prev_x = x;
        prev_y = y;
    }
    diff = diff.lines().rev().map(|s| format!("{}\n", s)).collect();
    diff
}

fn find_path(trace: Vec<Vec<usize>>, a_len: usize, b_len: usize) -> Vec<(usize, usize)>{
    let max = a_len + b_len;

    let mut x = a_len as isize;
    let mut y = b_len as isize;
    let mut path = vec![];

    for (d, v) in trace.iter().enumerate().rev() {
        let d = d as isize;

        let k = x - y;

        let index_k = if k < 0 {
            max - k.abs() as usize
        } else {
            max + k.abs() as usize
        };


        let prev_k = if (k == -d) || (k != d && v[index_k - 1] < v[index_k + 1]) {
            k + 1
        }
        else {
            k - 1
        };

        let prev_index_k = if k < 0 {
            max - prev_k.abs() as usize
        } else {
            max + prev_k.abs() as usize
        };
        let prev_x = v[prev_index_k] as isize;
        let prev_y = prev_x - prev_k;

        while x > prev_x && y > prev_y {
            path.push((x as usize, y as usize));
            x -= 1;
            y -= 1;
        }

        if d > 0 {
            path.push((x as usize, y as usize));
        }

        x = prev_x;
        y = prev_y;
    }

    path.push((0, 0));
    path.remove(0);
    path
}

// finds the trace of an SES
fn explore_paths(a: &[u8], b: &[u8]) -> Vec<Vec<usize>> {
    let (n, m) = (a.len(), b.len());
    let max = n + m;
    let mut v = vec![0; 2 * max + 1];
    let mut t: Vec<Vec<usize>> = vec![];

    // for d = 0, we need a starting point at k = 1, (x, y) = (0, -1)
    v[max + 1] = 0;

    for d in 0..=max {
        // negatives are a pain so 0 to 2d maps to -d to d. using this,
        // the array can be mapped from index k (-max=..=max) (when k -d..d as normal)
        // to (max - d) + k (0..2*max) (when k 0..=2*d)
        for k in (0..=(2*d)).step_by(2) {
            let index_k = (max - d) + k;
            let mut x = if (k == 0) || (k != 2*d && v[index_k - 1] < v[index_k + 1]) {
                v[index_k + 1]
            }
            else {
                v[index_k - 1] + 1
            };

            // this is an archaic form of x - (k - d)
            // but that would require computing a negative number
            // even though the result is always positive
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

    // not possible to get here, but needed
    t
}

