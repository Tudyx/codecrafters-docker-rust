use std::{env, fs, os::unix::fs::chroot};

use anyhow::{Context, Result};

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    let sandbox = tempfile::tempdir().context("failed to create tmpdir")?;

    // Copy the docker-explorer binary into the sandbox
    let path_command = sandbox
        .path()
        .join(command.strip_prefix('/').unwrap_or(command));
    std::fs::create_dir_all(path_command.parent().unwrap())
        .context("failed create dir for the command")?;
    std::fs::copy(command, path_command).context("failed to copy the commande")?;

    // Create /dev/null required by std::process::Command
    let dev_null = sandbox.path().join("dev/null");
    fs::create_dir_all(sandbox.path().join(dev_null.parent().unwrap()))
        .context("failed to create /dev/null")?;
    fs::File::create(&dev_null)?;

    // Create the jail.
    chroot(sandbox).context("failed to chroot")?;
    env::set_current_dir("/").context("failed to set current dir")?;

    let status = std::process::Command::new(command)
        .args(command_args)
        .status()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}
