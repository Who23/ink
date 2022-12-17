use crate::commit::Commit;
use crate::{InkError, CURSOR_FILE};
use std::convert::TryInto;
use std::fs::{self, File};
use std::path::Path;

pub fn init(ink_root: &Path) -> Result<(), InkError> {
    File::create(ink_root.join(CURSOR_FILE))?;
    Ok(())
}

pub fn set(ink_root: &Path, commit: &Commit) -> Result<(), InkError> {
    fs::write(ink_root.join(CURSOR_FILE), commit.hash())?;
    Ok(())
}

pub fn get(ink_root: &Path) -> Result<Commit, InkError> {
    let hash: [u8; 32] = match fs::read(ink_root.join(CURSOR_FILE))?.try_into() {
        Ok(hash) => Ok(hash),
        Err(_) => Err(InkError::Err("Cursor hash is wrong length")),
    }?;

    Commit::from(&hash, ink_root)
}
