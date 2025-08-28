use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

#[cfg(windows)]
use std::fs::File;
#[cfg(windows)]
use std::mem::MaybeUninit;
#[cfg(windows)]
use std::os::windows::io::AsRawHandle;
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::{
    BY_HANDLE_FILE_INFORMATION, GetFileInformationByHandle,
};

pub fn file_unique_id(path: &Path) -> Option<String> {
    #[cfg(unix)]
    {
        if let Ok(meta) = path.metadata() {
            let dev = meta.dev();
            let ino = meta.ino();
            return Some(format!("{}:{}", dev, ino));
        }
        return None;
    }

    #[cfg(windows)]
    {
        let file = File::open(path).ok()?;
        let handle = file.as_raw_handle();

        unsafe {
            let mut info = MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::uninit();
            if GetFileInformationByHandle(handle as _, info.as_mut_ptr()) != 0 {
                let info = info.assume_init();
                let volume_serial = info.dwVolumeSerialNumber;
                let file_index = ((info.nFileIndexHigh as u64) << 32) | info.nFileIndexLow as u64;
                return Some(format!("{}:{}", volume_serial, file_index));
            }
        }
        None
    }

    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_file_unique_id_does_not_crash() {
        // Create a temporary file
        let tmp_dir = env::temp_dir();
        let tmp_file_path = tmp_dir.join("test_file_unique_id.txt");

        {
            let mut file = File::create(&tmp_file_path).expect("Failed to create temp file");
            writeln!(file, "hello world").expect("Failed to write temp file");
        }

        // Ensure calling file_unique_id does not panic
        let _ = file_unique_id(&tmp_file_path);

        // Cleanup
        let _ = std::fs::remove_file(&tmp_file_path);
    }
}
