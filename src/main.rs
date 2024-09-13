use std::io::Write;

use anyhow::{Context, Result};

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // Uncomment this block to pass the first stage!
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];
    let output = std::process::Command::new(command)
        .args(command_args)
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
