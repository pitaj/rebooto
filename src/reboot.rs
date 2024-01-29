#[cfg(target_os = "linux")]
mod manjaro {
    use anyhow::Context;
    use zbus::blocking::Connection;

    pub fn reboot(dry_run: bool) -> anyhow::Result<()> {
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
}

#[cfg(target_os = "linux")]
pub use manjaro::*;

#[cfg(target_os = "windows")]
mod windows {}

#[cfg(target_os = "windows")]
pub use windows::*;
