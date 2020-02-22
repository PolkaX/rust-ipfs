use std::fs;
use std::io;
use std::sync::Mutex;

use lazy_static::lazy_static;
// re-export it
pub use fs2::FileExt;

use log::{debug, error};
use std::path::PathBuf;

pub const LOG_TARGET: &str = "lock";

lazy_static! {
    static ref GLOBAL_LOCK: Mutex<()> = Mutex::new(());
}

pub fn lock(path: &PathBuf, lock_file: &str) -> io::Result<fs::File> {
    if !path.exists() {
        fs::create_dir_all(path.as_path())?;
    }

    let mut path = path.to_owned();
    path.push(lock_file);

    let _ = GLOBAL_LOCK.lock().unwrap();
    let f = fs::File::create(path.as_path())?;
    f.try_lock_exclusive()?;
    Ok(f)
}

pub fn unlock(file: fs::File) -> io::Result<()> {
    let _ = GLOBAL_LOCK.lock().unwrap();
    test_locked(&file).and_then(|r| if r { file.unlock() } else { Ok(()) })
}

fn test_locked(f: &fs::File) -> io::Result<bool> {
    let r = f.try_lock_exclusive();
    match r {
        Ok(_) => {
            debug!(target: LOG_TARGET, "No one has a lock");
            Ok(false)
        }
        Err(e) => {
            debug!(target: LOG_TARGET, "try lock failed, err: {:?}", e);
            if e.kind() == io::ErrorKind::WouldBlock {
                Ok(true)
            } else {
                error!(
                    target: LOG_TARGET,
                    "lock failed due to other reason, err: {:?}", e
                );
                Err(e)
            }
        }
    }
}

pub fn locked(path: &PathBuf, lock_file: &str) -> io::Result<bool> {
    debug!(target: LOG_TARGET, "Checking lock");
    let mut path = path.to_owned();
    path.push(lock_file);
    if !path.exists() {
        debug!(target: LOG_TARGET, "File doesn't exist: {:?}", path);
        return Ok(false);
    }

    let _ = GLOBAL_LOCK.lock().unwrap();
    let f = fs::OpenOptions::new().write(true).open(path.as_path())?;
    test_locked(&f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    fn assert_lock(path: &PathBuf, file: &str, expected: bool) {
        let is_locked = locked(path, file).unwrap();
        assert_eq!(is_locked, expected)
    }

    #[test]
    fn test_lock_simple() {
        let lock_file = "my-test.lock";
        let dir = TempDir::new("lock").unwrap();

        let mut p = dir.path().to_path_buf();
        p.push(lock_file);

        // make sure we start clean
        let _ = fs::remove_file(p.as_path());

        let path = dir.path().to_path_buf();
        assert_lock(&path, lock_file, false);

        let file = lock(&path, lock_file).unwrap();

        assert_lock(&path, lock_file, true);

        unlock(file).unwrap();

        assert_lock(&path, lock_file, false);

        // second round of locking
        let file = lock(&path, lock_file).unwrap();
        assert_lock(&path, lock_file, true);
        unlock(file).unwrap();
        assert_lock(&path, lock_file, false);
    }

    #[test]
    fn test_lock_multiple() {
        let lock_file1 = "test-1.lock";
        let lock_file2 = "test-2.lock";
        let dir = TempDir::new("lock").unwrap();

        // make sure we start clean
        let mut p = dir.path().to_path_buf();
        p.push(lock_file1);
        let _ = fs::remove_file(p.as_path());
        p.pop();
        p.push(lock_file2);
        let _ = fs::remove_file(p.as_path());

        let path = dir.path().to_path_buf();
        let file1 = lock(&path, lock_file1).unwrap();
        let file2 = lock(&path, lock_file2).unwrap();

        assert_lock(&path, lock_file1, true);
        assert_lock(&path, lock_file2, true);

        unlock(file1).unwrap();

        assert_lock(&path, lock_file1, false);
        assert_lock(&path, lock_file2, true);

        unlock(file2).unwrap();

        assert_lock(&path, lock_file1, false);
        assert_lock(&path, lock_file2, false);
    }
}
