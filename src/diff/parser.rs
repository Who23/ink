/// Holds functions for parsing simple types from a string
// TODO: use InkErrors!
use std::error::Error;

/// Parse a usize from a string, returning the reamaining string and the usize
pub fn read_usize<'a>(input: &'a str) -> Result<(&'a str, usize), Box<dyn Error>> {
    let mut boundary = 0;

    for (index, c) in input.char_indices() {
        if !c.is_ascii_digit() {
            boundary = index;
            break;
        }
    }

    let number = input[..boundary].parse::<usize>()?;
    let remainder = &input[boundary..];

    Ok((remainder, number))
}

/// Parse lines from a string, returning the remaining string and a vec of lines
/// If there are no newlines in the string, it will assume the entire string is one line
/// as a workaround for how lines are dealt with in the diff module
pub fn read_lines<'a>(
    input: &'a str,
    num_lines: usize,
) -> Result<(&'a str, Vec<&'a str>), Box<dyn Error>> {
    let mut counted_newlines = 0;
    let mut lines = Vec::with_capacity(num_lines);
    let mut prev_newline_index = 0;

    // count newline characters until we reach the amount of lines specified or we're through the whole string.
    for (index, c) in input.char_indices() {
        if c == '\n' {
            counted_newlines += 1;
            lines.push(&input[prev_newline_index..index]);
            prev_newline_index = index + 1;

            if counted_newlines == num_lines {
                break;
            }
        }
    }

    // if we had no newlines, treat the entire string as one big line. Otherwise, return the lines found.
    if prev_newline_index != 0 {
        let remainder = &input[prev_newline_index..];
        Ok((remainder, lines))
    } else {
        lines.push(input);
        Ok(("", lines))
    }
}

/// Skips the given string in the input, returning the remaining string.
pub fn skip_sequence<'a>(input: &'a str, sequence: &str) -> Result<&'a str, Box<dyn Error>> {
    let val = input
        .strip_prefix(sequence)
        .ok_or("Sequence not found at beginning of input string")?;

    Ok(val)
}

#[cfg(test)]
mod tests {
    #[test]
    fn read_usize() {
        let hello = "123456hi";
        assert_eq!(super::read_usize(hello).unwrap(), ("hi", 123456))
    }

    #[test]
    fn read_lines() {
        let ex = "hello\nI am \n\nso cool";
        assert_eq!(
            super::read_lines(ex, 3).unwrap(),
            ("so cool", vec!["hello", "I am ", ""])
        )
    }

    #[test]
    fn skip_sequence() {
        let ex = ",.123";
        assert_eq!(super::skip_sequence(ex, ",.").unwrap(), "123")
    }
}
