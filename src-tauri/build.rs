use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    build_popper_sidecar();
    tauri_build::build();
}

fn build_popper_sidecar() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let popper_path_env = env::var("POPPER_PATH").ok();
    let popper_manifest = popper_path_env
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.join("..").join("..").join("popper"))
        .join("Cargo.toml");

    if !popper_manifest.exists() {
        println!(
            "cargo:warning=Popper manifest not found at {}",
            popper_manifest.display()
        );
        return;
    }

    let profile = env::var("PROFILE").unwrap_or_else(|_| "release".into());
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--manifest-path")
        .arg(&popper_manifest);

    if profile == "release" {
        cmd.arg("--release");
    }

    match cmd.status() {
        Ok(status) if status.success() => {}
        Ok(status) => {
            println!(
                "cargo:warning=Failed to build popper sidecar (status {}), skipping copy",
                status
            );
            return;
        }
        Err(err) => {
            println!(
                "cargo:warning=Error invoking cargo to build popper sidecar: {}",
                err
            );
            return;
        }
    }

    let profile_dir = if profile == "release" { "release" } else { "debug" };
    let popper_bin = popper_manifest
        .parent()
        .unwrap_or(Path::new("."))
        .join("target")
        .join(profile_dir)
        .join("popper");

    if !popper_bin.exists() {
        println!(
            "cargo:warning=Popper binary not found at {}, skipping copy",
            popper_bin.display()
        );
        return;
    }

    let dest_dir = manifest_dir.join("bin");
    if let Err(err) = fs::create_dir_all(&dest_dir) {
        println!(
            "cargo:warning=Failed to create bin directory {}: {}",
            dest_dir.display(),
            err
        );
        return;
    }

    let target_triple = env::var("TAURI_ENV_TARGET_TRIPLE")
        .ok()
        .or_else(|| env::var("TARGET").ok())
        .unwrap_or_else(|| "unknown-target".to_string());

    let copies = [
        dest_dir.join("popper"),
        dest_dir.join(format!("popper-{target_triple}")),
    ];

    for dest_path in copies {
        if let Err(err) = fs::copy(&popper_bin, &dest_path) {
            println!(
                "cargo:warning=Failed to copy popper sidecar to {}: {}",
                dest_path.display(),
                err
            );
            continue;
        }

        #[cfg(target_family = "unix")]
        if let Ok(metadata) = fs::metadata(&dest_path) {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            let _ = fs::set_permissions(&dest_path, permissions);
        }
    }

    println!("cargo:rerun-if-changed={}", popper_manifest.display());
    println!("cargo:rerun-if-env-changed=POPPER_PATH");
}
