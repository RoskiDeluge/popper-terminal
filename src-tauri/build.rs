use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

fn main() {
    build_popper_sidecar();
    tauri_build::build();
}

fn watch_dir(path: &Path) {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();

        if entry_path.is_dir() {
            watch_dir(&entry_path);
            continue;
        }

        println!("cargo:rerun-if-changed={}", entry_path.display());
    }
}

fn build_popper_sidecar() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let popper_path_env = env::var("POPPER_PATH").ok();
    let popper_root = popper_path_env
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.join("..").join("..").join("popper"));
    let popper_manifest = popper_root.join("Cargo.toml");

    if !popper_manifest.exists() {
        println!(
            "cargo:warning=Popper manifest not found at {}",
            popper_manifest.display()
        );
        return;
    }

    let profile = env::var("PROFILE").unwrap_or_else(|_| "release".into());
    let mut cmd = Command::new("cargo");
    let popper_dir = popper_manifest.parent().unwrap_or(Path::new("."));
    let sidecar_target_dir = manifest_dir.join("target").join("popper-sidecar");
    cmd.arg("build")
        .arg("--manifest-path")
        .arg(&popper_manifest)
        .current_dir(popper_dir)
        .env("CARGO_TARGET_DIR", &sidecar_target_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped());

    if profile == "release" {
        cmd.arg("--release");
    }

    let profile_dir = if profile == "release" { "release" } else { "debug" };
    let popper_bin = sidecar_target_dir
        .join(profile_dir)
        .join("popper");

    match cmd.output() {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!(
                "cargo:warning=Failed to build popper sidecar (status {}); attempting to reuse existing binary at {}{}{}",
                output.status,
                popper_bin.display(),
                if stderr.trim().is_empty() { "" } else { ": " },
                stderr.trim()
            );
        }
        Err(err) => {
            println!(
                "cargo:warning=Error invoking cargo to build popper sidecar; attempting to reuse existing binary at {}: {}",
                popper_bin.display(),
                err
            );
        }
    }

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
    println!(
        "cargo:rerun-if-changed={}",
        popper_root.join("Cargo.lock").display()
    );
    watch_dir(&popper_root.join("src"));
    println!("cargo:rerun-if-env-changed=POPPER_PATH");
}
