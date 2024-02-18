use windows::core::Error;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::SE_PRIVILEGE_ENABLED;
use windows::Win32::Security::SE_SHUTDOWN_NAME;
use windows::Win32::Security::{
    AdjustTokenPrivileges, LookupPrivilegeValueW, TOKEN_ACCESS_MASK, TOKEN_ADJUST_PRIVILEGES,
    TOKEN_PRIVILEGES,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

/// Represents a process token. The associated `HANDLE` is closed when
/// this object is dropped.
struct ProcessToken(HANDLE);

impl ProcessToken {
    /// Obtains the process token with the given access for the current process
    ///
    /// # Arguments
    ///
    /// * `desired_access`: Token access level
    pub fn open_current(desired_access: TOKEN_ACCESS_MASK) -> Result<Self, Error> {
        Self::open(unsafe { GetCurrentProcess() }, desired_access)
    }

    /// Obtains the process token for the given `process`
    ///
    /// # Arguments
    ///
    /// * `process`: Process to get the token for
    /// * `desired_access`: Token access level
    pub fn open(process: HANDLE, desired_access: TOKEN_ACCESS_MASK) -> Result<Self, Error> {
        let mut process_token: HANDLE = HANDLE::default();
        let result = unsafe { OpenProcessToken(process, desired_access, &mut process_token) };

        result.map(|_| Self(process_token))
    }
}

impl Drop for ProcessToken {
    fn drop(&mut self) {
        unsafe {
            // Ignore errors when closing the handle.
            let _ = CloseHandle(self.0);
        }
    }
}

/// Updates the privileges of the current thread to include SeShutdownPrivilege, which is
/// required to reboot the system.
pub fn add_privilege() -> Result<(), windows::core::Error> {
    // We need SeShutdownPrivilege to do anything NVRAM-related
    // So we configure it for the current thread here
    // This means SystemManager is not Send
    let mut tkp = TOKEN_PRIVILEGES::default();

    // Get a token for this process.
    let process_token = ProcessToken::open_current(TOKEN_ADJUST_PRIVILEGES)?;

    // Get the LUID for the shutdown privilege.
    unsafe {
        LookupPrivilegeValueW(
            PCWSTR::null(),
            SE_SHUTDOWN_NAME,
            &mut tkp.Privileges[0].Luid,
        )?;
    }

    tkp.PrivilegeCount = 1;
    tkp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

    // Get the shutdown privilege for this process.
    unsafe {
        AdjustTokenPrivileges(process_token.0, false, Some(&tkp), 0, None, None)?;
    }

    Ok(())
}
