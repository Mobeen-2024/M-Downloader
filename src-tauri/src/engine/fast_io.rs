use std::fs::File;
use std::os::windows::io::AsRawHandle;
use std::path::Path;

#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::{
    SetFileValidData, FSCTL_SET_SPARSE,
};
#[cfg(windows)]
use windows_sys::Win32::System::IO::DeviceIoControl;

#[cfg(windows)]
pub fn prepare_file_allocation(path: &Path, size: u64) -> Result<File, String> {
    // 1. Create the file normally.
    let file = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;
    
    // 2. Enable Sparse File attribute.
    // This tells NTFS not to zero-fill the gaps between segments.
    if let Err(e) = enable_sparse(&file) {
        log::warn!("Could not enable sparse attribute: {}. Falling back to default I/O.", e);
    }

    // 3. Set Logical Size. 
    // This establishes the final file size in the directory entry.
    file.set_len(size).map_err(|e| format!("Failed to set logical size: {}", e))?;

    // 4. (Optional) Advanced VDL bypass for Admin users.
    // SetFileValidData prevents NTFS from zeroing up to the end of the file.
    // This is high-risk/high-performance. If it fails, we still have Sparse.
    if let Err(e) = try_set_valid_data(&file, size) {
        log::debug!("SetFileValidData (VDL bypass) skipped or failed: {}. This is normal for non-admin users.", e);
    }

    Ok(file)
}

#[cfg(windows)]
fn enable_sparse(file: &File) -> Result<(), String> {
    let handle = file.as_raw_handle() as _;
    let mut bytes_returned = 0u32;
    
    unsafe {
        let result = DeviceIoControl(
            handle,
            FSCTL_SET_SPARSE,
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            0,
            &mut bytes_returned,
            std::ptr::null_mut(),
        );

        if result == 0 {
            return Err("Win32 DeviceIoControl(FSCTL_SET_SPARSE) failed".to_string());
        }
    }
    Ok(())
}

#[cfg(windows)]
fn try_set_valid_data(file: &File, size: u64) -> Result<(), String> {
    let handle = file.as_raw_handle() as _;
    
    unsafe {
        // SetFileValidData advances the VDL (Valid Data Length) to the end of the file.
        // This effectively bypasses the background zeroing operation on NTFS.
        let result = SetFileValidData(handle, size as i64);
        
        if result == 0 {
            return Err("Win32 SetFileValidData failed (likely missing SeManageVolumePrivilege)".to_string());
        }
    }
    Ok(())
}

// Fallback for non-windows platforms (e.g., during cross-compilation/tests).
#[cfg(not(windows))]
pub fn prepare_file_allocation(path: &Path, size: u64) -> Result<File, String> {
    let file = File::create(path).map_err(|e| e.to_string())?;
    file.set_len(size).map_err(|e| e.to_string())?;
    Ok(file)
}
