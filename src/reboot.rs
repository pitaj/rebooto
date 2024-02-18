use anyhow::Context;

#[cfg(target_os = "linux")]
pub fn reboot(dry_run: bool) -> anyhow::Result<()> {
    use zbus::blocking::Connection;

    println!("Rebooting...");

    let connection =
        Connection::session().context("reboot failed: error connecting to dbus session")?;

    if dry_run {
        println!("[dry run] DBUS org.kde.LogoutPrompt /LogoutPrompt promptReboot");
    } else {
        connection
            .call_method(
                Some("org.kde.LogoutPrompt"),
                "/LogoutPrompt",
                Some("org.kde.LogoutPrompt"),
                "promptReboot",
                &(),
            )
            .context("reboot failed: error sending dbus message")?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
mod win_security;

#[cfg(target_os = "windows")]
pub fn reboot(dry_run: bool) -> anyhow::Result<()> {
    use windows::Win32::System::Shutdown::{
        ExitWindowsEx, EWX_REBOOT, SHTDN_REASON_MAJOR_OTHER, SHTDN_REASON_MINOR_OTHER,
    };

    win_security::add_privilege().context("reboot failed: error adding shutdown privilege")?;

    println!("Rebooting...");

    if dry_run {
        println!("[dry run] ExitWindowsEx(RESTARTAPPS)");
    } else {
        unsafe {
            ExitWindowsEx(
                // Shuts down the system and then restarts the system.
                EWX_REBOOT,
                SHTDN_REASON_MAJOR_OTHER | SHTDN_REASON_MINOR_OTHER,
            )
            .context("reboot failed: error calling ExitWindowsEx")?
        }
    }

    Ok(())
}
