use std::fs::{File};
use std::io::{BufReader, BufRead};
use std::path::Path;

use ink::diff::Diff;

fn main() {
    let a = File::open("testing/a.txt").unwrap();
    let b = File::open("testing/b.txt").unwrap();

    let a = BufReader::new(a);
    let b = BufReader::new(b);

    let a: Vec<String> = a.lines().collect::<Result<_, _>>().unwrap();
    let b: Vec<String> = b.lines().collect::<Result<_, _>>().unwrap();

    let diff = Diff::from(a, b);
    let es: Vec<String> = diff.edit_script().lines().map(|s| s.to_owned()).collect();
    let s_diff = Diff::from_edit_script(es).unwrap();

    println!("{}", diff.edit_script());
    println!("\n\n");
    println!("{}", s_diff.edit_script());
}
