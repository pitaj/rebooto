use std::str::FromStr;

use anyhow::Context;
use clap::Parser;
use efivar::efi::Variable;
use once_cell::sync::Lazy;

mod efi;
mod reboot;
mod string;

#[derive(clap::Parser)]
struct Args {
    #[clap(short, long)]
    dry_run: bool,

    #[clap(long)]
    privileged: bool,

    target_os: String,
}

pub const REFIND_PREVIOUSBOOT_NAME: &str = "PreviousBoot-36d08fa7-cf0b-42f5-8f14-68df73ed3740";
pub const BACKUP_PREVIOUSBOOT_NAME: &str =
    "PreviousBootBackup-2bd809f0-cba2-4074-bbe8-56a655f07262";

pub static REFIND_PREVIOUSBOOT: Lazy<Variable> =
    Lazy::new(|| Variable::from_str(REFIND_PREVIOUSBOOT_NAME).unwrap());
pub static BACKUP_PREVIOUSBOOT: Lazy<Variable> =
    Lazy::new(|| Variable::from_str(BACKUP_PREVIOUSBOOT_NAME).unwrap());

fn main() -> anyhow::Result<()> {
    let Args {
        dry_run,
        privileged,
        target_os,
    } = Args::parse();

    let refind_previousboot = &*REFIND_PREVIOUSBOOT;

    let system = efivar::system();

    let (prev_utf16le, flags) = system
        .read(refind_previousboot)
        .context("failed to read previous EFI variable value")?;
    let prev = string::from_utf16le(&prev_utf16le).context("failed to parse value as utf16le")?;

    assert_eq!(&prev_utf16le, &string::encode_utf16le(&prev));

    let next = format!("{} \0", target_os);

    if next == prev {
        println!("EFI variable `{refind_previousboot}` already set to '{next}'");

        reboot::reboot(dry_run)?;
        return Ok(());
    }

    if privileged {
        efi::set_privileged(
            dry_run,
            &target_os,
            system,
            flags,
            &prev_utf16le,
            &prev,
            &next,
        )?;

        return Ok(());
    }

    efi::set(
        dry_run,
        &target_os,
        system,
        flags,
        &prev_utf16le,
        &prev,
        &next,
    )?;
    reboot::reboot(dry_run)?;

    Ok(())
}
