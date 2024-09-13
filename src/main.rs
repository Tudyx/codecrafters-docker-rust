use std::{env, fs, io::Write, os::unix::fs::chroot};

use anyhow::{Context, Result};

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // Uncomment this block to pass the first stage!
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];
    fs::create_dir("/tmp/some_dir")?;
    fs::copy("/usr/bin/ls", "/tmp/some_dir")?;
    chroot("/tmp/some_dir")?;
    env::set_current_dir("/")?;
    fs::create_dir("/dev/null")?;

    let output = std::process::Command::new(command)
        .args(command_args)
        .current_dir("/")
        .output()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    if output.status.success() {
        std::io::stdout().write_all(&output.stdout)?;
        std::io::stderr().write_all(&output.stderr)?;
    } else {
        std::process::exit(output.status.code().unwrap_or(1));
    }
    Ok(())
}
