use std::{env, fs, io::Write, os::unix::fs::chroot};

use anyhow::{Context, Result};

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // Uncomment this block to pass the first stage!
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    let sandbox = tempfile::tempdir().context("failed to create tmpdir")?;

    // copy the binary into the sandbox
    let path_command = sandbox
        .path()
        .join(command.strip_prefix('/').unwrap_or(command));
    // println!("create_dir_all '{:?}'", path_command.parent().unwrap());

    std::fs::create_dir_all(path_command.parent().unwrap())
        .context("failed create dir for the command")?;
    std::fs::copy(command, path_command).context("failed to copy the commande")?;
    // println!("copy command {command}");

    // std::process::Command expect /dev/null to work
    let dev_null = sandbox.path().join("dev/null");
    fs::create_dir_all(sandbox.path().join(dev_null.parent().unwrap()))
        .context("failed to create /dev/null")?;
    fs::File::create(&dev_null)?;

    // let folder = sandbox.path().join("usr/local/bin/docker-explorer");
    // let folder = folder.to_string_lossy();
    // println!("ls {folder}");

    // let stdout = std::process::Command::new("ls")
    //     .arg(folder.as_ref())
    //     .output()
    //     .unwrap()
    //     .stdout;
    // println!("{}", std::str::from_utf8(&stdout).unwrap());

    // Create the jail.
    chroot(sandbox).context("failed to chroot")?;
    env::set_current_dir("/").context("failed to set current dir")?;
    // println!("the command excuted\n{}", command);
    // println!("the command args exexcuted\n{:?}", command_args);

    let output = std::process::Command::new(command)
        .args(command_args)
        // .current_dir("/")
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
