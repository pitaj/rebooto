#[cfg(target_os = "linux")]
mod manjaro {
    use anyhow::Context;
    use efivar::{efi::VariableFlags, VarManager};

    pub fn set(
        dry_run: bool,
        target_os: &str,
        _system: Box<dyn VarManager>,
        _flags: VariableFlags,
        _prev_utf16le: &[u8],
        _prev: &str,
        _next: &str,
    ) -> anyhow::Result<()> {
        let executable = std::env::current_exe()?.canonicalize()?;

        let mut command = std::process::Command::new("/usr/bin/sudo");
        command
            .arg(&executable)
            .arg(target_os)
            .arg("--privileged");

        if dry_run {
            command.arg("--dry-run");
        }

        let stat = command.status()?;
        if !stat.success() {
            anyhow::bail!("failed to run as privileged: {stat}");
        }

        Ok(())
    }

    pub fn set_privileged(
        dry_run: bool,
        _target_os: &str,
        mut system: Box<dyn VarManager>,
        flags: VariableFlags,
        prev_utf16le: &[u8],
        prev: &str,
        next: &str,
    ) -> anyhow::Result<()> {
        let refind_previousboot = &*crate::REFIND_PREVIOUSBOOT;
        let backup_previousboot = &*crate::BACKUP_PREVIOUSBOOT;

        println!("Setting EFI vars...");

        if dry_run {
            match system.read(backup_previousboot) {
                Ok((backup_utf16le, _)) => {
                    let backup = crate::string::from_utf16le(&backup_utf16le)
                        .context("failed to parse backup value as utf16le")?;

                    println!("[dry run] set EFI variable `{backup_previousboot}` to '{prev}' (was '{backup}')");
                }
                Err(efivar::Error::VarNotFound { .. }) => {
                    println!(
                        "[dry run] create EFI variable `{backup_previousboot}` with value '{prev}'"
                    );
                }
                Err(e) => return Err(e)?,
            }
            println!(
                "[dry run] set EFI variable `{refind_previousboot}` to '{next}' (was '{prev}')"
            );
        } else {
            system
                .write(backup_previousboot, flags, prev_utf16le)
                .context("failed to write backup EFI variable")?;
            system
                .write(
                    refind_previousboot,
                    flags,
                    &crate::string::encode_utf16le(next),
                )
                .context("failed to write EFI variable")?;
        }

        println!("Success: EFI variable `{refind_previousboot}` set to '{next}' (was '{prev}')");

        Ok(())
    }
}

#[cfg(target_os = "linux")]
pub use manjaro::*;

#[cfg(target_os = "windows")]
mod windows {}

#[cfg(target_os = "windows")]
pub use windows::*;
