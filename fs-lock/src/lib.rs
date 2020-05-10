use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

// re-export it
pub use fs2::FileExt;
use lazy_static::lazy_static;
use log::{debug, error};

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

    let _lock = GLOBAL_LOCK.lock().unwrap();
    let f = fs::File::create(path.as_path())?;
    f.try_lock_exclusive()?;
    Ok(f)
}

/// Unlock the file. if file is not locked, just return Ok(()).
/// Thus the file could be unlocked more than once.
pub fn unlock(file: &fs::File) -> io::Result<()> {
    let _lock = GLOBAL_LOCK.lock().unwrap();
    file.unlock()
}

pub fn locked(path: &PathBuf, lock_file: &str) -> io::Result<bool> {
    debug!(target: LOG_TARGET, "Checking lock");
    let mut path = path.to_owned();
    path.push(lock_file);
    if !path.exists() {
        debug!(target: LOG_TARGET, "File doesn't exist: {:?}", path);
        return Ok(false);
    }

    let _lock = GLOBAL_LOCK.lock().unwrap();
    let f = fs::OpenOptions::new().write(true).open(path.as_path())?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_lock(path: &PathBuf, file: &str, expected: bool) {
        let is_locked = locked(path, file).unwrap();
        assert_eq!(is_locked, expected)
    }

    #[test]
    fn test_lock_simple() {
        let lock_file = "my-test.lock";
        let dir = tempfile::Builder::new().prefix("lock").tempdir().unwrap();

        let mut p = dir.path().to_path_buf();
        p.push(lock_file);

        // make sure we start clean
        let _ = fs::remove_file(p.as_path());

        let path = dir.path().to_path_buf();
        assert_lock(&path, lock_file, false);

        let file = lock(&path, lock_file).unwrap();

        assert_eq!(locked(&path, lock_file).unwrap(), true);

        unlock(&file).unwrap();

        assert_eq!(locked(&path, lock_file).unwrap(), false);

        // second round of locking
        let file2 = lock(&path, lock_file).unwrap();
        assert_lock(&path, lock_file, true);
        unlock(&file2).unwrap();
        assert_lock(&path, lock_file, false);
    }

    #[test]
    fn test_lock_multiple() {
        let lock_file1 = "test-1.lock";
        let lock_file2 = "test-2.lock";
        let dir = tempfile::Builder::new().prefix("lock").tempdir().unwrap();

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

        unlock(&file1).unwrap();

        assert_lock(&path, lock_file1, false);
        assert_lock(&path, lock_file2, true);

        unlock(&file2).unwrap();

        assert_lock(&path, lock_file1, false);
        assert_lock(&path, lock_file2, false);
    }

    #[test]
    fn test_thread_safe() {
        use std::thread;

        let lock_file = "my-test.lock";
        let dir = tempfile::Builder::new().prefix("lock").tempdir().unwrap();
        let mut p = dir.path().to_path_buf();
        p.push(lock_file);

        // make sure we start clean
        let _ = fs::remove_file(p.as_path());

        let path = dir.path().to_path_buf();
        for _ in 0..100 {
            let path_d = path.clone();
            let file_name = lock_file.to_string();
            let child = thread::spawn(move || lock(&path_d, &file_name));

            let current_ret = lock(&path, lock_file);

            let res = child.join().unwrap();

            assert!((current_ret.is_ok() && res.is_err()) || (current_ret.is_err() && res.is_ok()));

            if let Ok(r) = current_ret {
                unlock(&r).unwrap();
            }

            if let Ok(r) = res {
                unlock(&r).unwrap();
            }
        }
    }
}
