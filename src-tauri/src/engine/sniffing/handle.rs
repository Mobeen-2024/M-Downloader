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

const FILE_DEVICE_UNKNOWN: u32 = 0x00000022;
const METHOD_BUFFERED: u32 = 0;
const FILE_ANY_ACCESS: u32 = 0;

#[inline(always)]
const fn ctl_code(device_type: u32, function: u32, method: u32, access: u32) -> u32 {
    (device_type << 16) | (access << 14) | (function << 2) | method
}

const IOCTL_MDOWNLOADER_GET_URLS: u32 = ctl_code(FILE_DEVICE_UNKNOWN, 0x801, METHOD_BUFFERED, FILE_ANY_ACCESS);

#[repr(C)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SniffedUrl {
    pub url: String,
    pub process_id: u32,
    pub process_name: String,
    pub content_type: String,
    pub timestamp: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SniffedUrlRaw {
    pub url: [u8; 512],
    pub process_id: u32,
    pub process_name: [u8; 128],
    pub content_type: [u8; 64],
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

    /// Fetches captured URL events from the kernel driver via IOCTL.
    pub fn poll_driver_events(&self) -> Result<Vec<SniffedUrl>, String> {
        let mut raw_buffer = [SniffedUrlRaw {
            url: [0; 512],
            process_id: 0,
            process_name: [0; 128],
            content_type: [0; 64],
            timestamp: 0,
        }; 16]; // Max 16 events per poll

        let mut bytes_returned: u32 = 0;

        unsafe {
            let success = DeviceIoControl(
                self.device_handle,
                IOCTL_MDOWNLOADER_GET_URLS,
                null_mut(),
                0,
                raw_buffer.as_mut_ptr() as _,
                std::mem::size_of_val(&raw_buffer) as u32,
                &mut bytes_returned,
                null_mut(),
            );

            if success == 0 {
                let err = GetLastError();
                if err == 0x103 { // ERROR_NO_MORE_ITEMS or empty
                    return Ok(vec![]);
                }
                return Err(format!("IOCTL_MDOWNLOADER_GET_URLS failed: 0x{:X}", err));
            }

            let num_events = bytes_returned / (std::mem::size_of::<SniffedUrlRaw>() as u32);
            let mut results = Vec::with_capacity(num_events as usize);

            for i in 0..num_events as usize {
                let raw = &raw_buffer[i];
                
                // Helper to convert null-terminated or zero-padded bytes to string
                let to_string = |bytes: &[u8]| {
                    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
                    String::from_utf8_lossy(&bytes[..len]).into_owned()
                };

                results.push(SniffedUrl {
                    url: to_string(&raw.url),
                    process_id: raw.process_id,
                    process_name: to_string(&raw.process_name),
                    content_type: to_string(&raw.content_type),
                    timestamp: raw.timestamp,
                });
            }

            Ok(results)
        }
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
