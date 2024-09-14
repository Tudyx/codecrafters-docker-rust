use std::{env, fs, os::unix::fs::chroot, path::Path};

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use libc::{unshare, CLONE_NEWPID};
use serde::Deserialize;

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let image = &args[2];
    let command = &args[3];
    let command_args = &args[4..];

    let sandbox = tempfile::tempdir().context("failed to create tmpdir")?;
    fetch_image_from_registry(image, sandbox.path())?;

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

    // Create a pid namespace for the process.
    unsafe { unshare(CLONE_NEWPID) };
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

fn fetch_image_from_registry(image: &str, sandbox: &Path) -> Result<()> {
    let (image, tag) = match image.split_once(':') {
        Some((image, tag)) => (image, tag),
        None => (image, "latest"),
    };
    println!("Will fetch the '{image}' image with this tag '{tag}' from the registry");
    let response : AuthResponse = reqwest::blocking::get(format!("https://auth.docker.io/token?service=registry.docker.io&scope=repository:library/{image}:pull"))?.json()?;
    let token = response.token;
    let client = reqwest::blocking::ClientBuilder::new().build()?;
    // For some image, like busybox, the returned response won't contains the layer
    // but the list of manifest. The accept header get ignore in that case.
    let manifest: serde_json::Value = client
        .get(format!(
            "https://registry.hub.docker.com/v2/library/{image}/manifests/{tag}"
        ))
        .bearer_auth(&token)
        .header(
            reqwest::header::ACCEPT,
            "application/vnd.docker.distribution.manifest.v2+json",
        )
        .send()?
        .json()?;
    println!("{manifest:#?}");

    for layer in manifest["layers"].as_array().unwrap() {
        let digest = layer["digest"].as_str().unwrap();
        let tar_gz = client
            .get(format!(
                "https://registry.hub.docker.com/v2/library/{image}/blobs/{digest}"
            ))
            .bearer_auth(&token)
            .send()?
            .bytes()?;
        println!("Size of the layer is {} bytes", tar_gz.len());

        let tar = GzDecoder::new(tar_gz.as_ref());
        let mut archive = tar::Archive::new(tar);

        // archive.set_preserve_permissions(true);

        // archive.set_unpack_xattrs(true);
        archive.unpack(sandbox)?;
    }
    Ok(())
}

// #[derive(Deserialize, Debug)]
// struct ManifestResponse {
//     manifests: Vec,
// }

#[derive(Deserialize, Debug)]
struct AuthResponse {
    token: String,
}
