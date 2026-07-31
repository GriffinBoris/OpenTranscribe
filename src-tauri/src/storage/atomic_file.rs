use std::io::Write;
use std::path::Path;

use atomic_write_file::AtomicWriteFile;

use crate::error::AppResult;

pub fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> AppResult<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    write(path, &bytes)
}

pub fn write(path: &Path, bytes: &[u8]) -> AppResult<()> {
    let mut file = AtomicWriteFile::options().open(path)?;
    file.write_all(bytes)?;
    file.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::write;

    #[test]
    fn atomically_replaces_existing_content() {
        let directory = tempdir().expect("temporary directory should exist");
        let path = directory.path().join("notes.md");
        write(&path, b"first").expect("first write should succeed");
        write(&path, b"second").expect("replacement should succeed");
        assert_eq!(std::fs::read(path).unwrap(), b"second");
    }
}
