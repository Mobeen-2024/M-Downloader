use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, OPEN_EXISTING,
};
use windows_sys::Win32::System::IO::DeviceIoControl;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SniffedUrl {
    pub url: String,
    pub process_id: u32,
    pub process_name: String,
    pub content_type: String,
    pub timestamp: u64,
}

/// Mdownloader WFP Driver Handle
pub struct SniffingHandle {
    pub device_handle: HANDLE,
}

impl SniffingHandle {
    /// Opens a handle to the mdownloader_sniff device.
    /// This requires the .sys driver to be installed and started.
    pub fn open() -> Result<Self, String> {
        let device_name: Vec<u16> = OsStr::new("\\\\.\\mdownloader_sniff")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let handle = CreateFileW(
                device_name.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                0,
                null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                0,
            );

            if handle == INVALID_HANDLE_VALUE {
                // Error 2 (FileNotFound) usually means the driver isn't loaded.
                let err = GetLastError();
                return Err(format!("Failed to open WFP driver handle: 0x{:X}", err));
            }

            Ok(Self { device_handle: handle })
        }
    }

    /// Checks if the driver is currently active.
    pub fn is_active(&self) -> bool {
        self.device_handle != INVALID_HANDLE_VALUE
    }

    /// Placeholder for IOCTL-based manifest polling or event notification.
    pub fn poll_driver_events(&self) -> Result<Vec<SniffedUrl>, String> {
        // In a real implementation, we would call DeviceIoControl here
        // with IOCTL_MDOWNLOADER_GET_URLS.
        
        // For demonstration/simulation, we return an empty vec.
        // The orchestrator will handle the polling loop.
        Ok(vec![])
    }
}

impl Drop for SniffingHandle {
    fn drop(&mut self) {
        if self.device_handle != INVALID_HANDLE_VALUE {
            unsafe {
                CloseHandle(self.device_handle);
            }
        }
    }
}
