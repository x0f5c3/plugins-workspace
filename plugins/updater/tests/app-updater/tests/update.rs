// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(dead_code, unused_imports)]

use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
    process::Command,
};

use serde::Serialize;

const UPDATER_PRIVATE_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5ClJXUlRZMEl5TlFOMFpXYzJFOUdjeHJEVXY4WE1TMUxGNDJVUjNrMmk1WlR3UVJVUWwva0FBQkFBQUFBQUFBQUFBQUlBQUFBQUpVK3ZkM3R3eWhyN3hiUXhQb2hvWFVzUW9FbEs3NlNWYjVkK1F2VGFRU1FEaGxuRUtlell5U0gxYS9DbVRrS0YyZVJGblhjeXJibmpZeGJjS0ZKSUYwYndYc2FCNXpHalM3MHcrODMwN3kwUG9SOWpFNVhCSUd6L0E4TGRUT096TEtLR1JwT1JEVFU9Cg==";

#[derive(Serialize)]
struct Config {
    version: &'static str,
}

#[derive(Serialize)]
struct PlatformUpdate {
    signature: String,
    url: &'static str,
    with_elevated_task: bool,
}

#[derive(Serialize)]
struct Update {
    version: &'static str,
    date: String,
    platforms: HashMap<String, PlatformUpdate>,
}

fn build_app(cwd: &Path, config: &Config, bundle_updater: bool, target: BundleTarget) {
    let mut command = Command::new("cargo");
    command
        .args(["tauri", "build", "--debug", "--verbose"])
        .arg("--config")
        .arg(serde_json::to_string(config).unwrap())
        .current_dir(cwd);

    #[cfg(target_os = "linux")]
    command.args(["--bundles", target.name()]);
    #[cfg(target_os = "macos")]
    command.args(["--bundles", target.name()]);

    if bundle_updater {
        #[cfg(windows)]
        command.args(["--bundles", "msi", "nsis"]);

        command
            .env("TAURI_SIGNING_PRIVATE_KEY", UPDATER_PRIVATE_KEY)
            .env("TAURI_SIGNING_PRIVATE_KEY_PASSWORD", "")
            .args(["--bundles", "updater"]);
    } else {
        #[cfg(windows)]
        command.args(["--bundles", target.name()]);
    }

    let status = command
        .status()
        .expect("failed to run Tauri CLI to bundle app");

    if !status.code().map(|c| c == 0).unwrap_or(true) {
        panic!("failed to bundle app {:?}", status.code());
    }
}

#[derive(Copy, Clone)]
enum BundleTarget {
    AppImage,

    App,

    Msi,
    Nsis,
}

impl BundleTarget {
    fn name(self) -> &'static str {
        match self {
            Self::AppImage => "appimage",
            Self::App => "app",
            Self::Msi => "msi",
            Self::Nsis => "nsis",
        }
    }
}

impl Default for BundleTarget {
    fn default() -> Self {
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        return Self::App;
        #[cfg(target_os = "linux")]
        return Self::AppImage;
        #[cfg(windows)]
        return Self::Nsis;
    }
}

#[cfg(target_os = "linux")]
fn bundle_paths(root_dir: &Path, version: &str) -> Vec<(BundleTarget, PathBuf)> {
    vec![(
        BundleTarget::AppImage,
        root_dir.join(format!(
            "target/debug/bundle/appimage/app-updater_{version}_amd64.AppImage"
        )),
    )]
}

#[cfg(target_os = "macos")]
fn bundle_paths(root_dir: &Path, _version: &str) -> Vec<(BundleTarget, PathBuf)> {
    vec![(
        BundleTarget::App,
        root_dir.join("target/debug/bundle/macos/app-updater.app"),
    )]
}

#[cfg(target_os = "ios")]
fn bundle_paths(root_dir: &Path, _version: &str) -> Vec<(BundleTarget, PathBuf)> {
    vec![(
        BundleTarget::App,
        root_dir.join("target/debug/bundle/ios/app-updater.ipa"),
    )]
}

#[cfg(target_os = "android")]
fn bundle_path(root_dir: &Path, _version: &str) -> PathBuf {
    root_dir.join("target/debug/bundle/android/app-updater.apk")
}

#[cfg(windows)]
fn bundle_paths(root_dir: &Path, version: &str) -> Vec<(BundleTarget, PathBuf)> {
    vec![
        (
            BundleTarget::Nsis,
            root_dir.join(format!(
                "target/debug/bundle/nsis/app-updater_{version}_x64-setup.exe"
            )),
        ),
        (
            BundleTarget::Msi,
            root_dir.join(format!(
                "target/debug/bundle/msi/app-updater_{version}_x64_en-US.msi"
            )),
        ),
    ]
}

#[test]
#[ignore]
fn update_app() {
    let target =
        tauri_plugin_updater::target().expect("running updater test in an unsupported platform");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root_dir = manifest_dir.join("../../../..");

    let mut config = Config { version: "1.0.0" };

    // bundle app update
    build_app(&manifest_dir, &config, true, Default::default());

    let updater_zip_ext = if cfg!(windows) { "zip" } else { "tar.gz" };

    for (bundle_target, out_bundle_path) in bundle_paths(&root_dir, "1.0.0") {
        let bundle_updater_ext = out_bundle_path
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .replace("exe", "nsis");
        let signature_path =
            out_bundle_path.with_extension(format!("{bundle_updater_ext}.{updater_zip_ext}.sig"));
        let signature = std::fs::read_to_string(&signature_path).unwrap_or_else(|_| {
            panic!("failed to read signature file {}", signature_path.display())
        });
        let out_updater_path =
            out_bundle_path.with_extension(format!("{}.{}", bundle_updater_ext, updater_zip_ext));
        let updater_path = root_dir.join(format!(
            "target/debug/{}",
            out_updater_path.file_name().unwrap().to_str().unwrap()
        ));
        std::fs::rename(&out_updater_path, &updater_path).expect("failed to rename bundle");

        let target = target.clone();
        std::thread::spawn(move || {
            // start the updater server
            let server =
                tiny_http::Server::http("localhost:3007").expect("failed to start updater server");

            loop {
                if let Ok(request) = server.recv() {
                    match request.url() {
                        "/" => {
                            let mut platforms = HashMap::new();

                            platforms.insert(
                                target.clone(),
                                PlatformUpdate {
                                    signature: signature.clone(),
                                    url: "http://localhost:3007/download",
                                    with_elevated_task: false,
                                },
                            );
                            let body = serde_json::to_vec(&Update {
                                version: "1.0.0",
                                date: time::OffsetDateTime::now_utc()
                                    .format(&time::format_description::well_known::Rfc3339)
                                    .unwrap(),
                                platforms,
                            })
                            .unwrap();
                            let len = body.len();
                            let response = tiny_http::Response::new(
                                tiny_http::StatusCode(200),
                                Vec::new(),
                                std::io::Cursor::new(body),
                                Some(len),
                                None,
                            );
                            let _ = request.respond(response);
                        }
                        "/download" => {
                            let _ = request.respond(tiny_http::Response::from_file(
                                File::open(&updater_path).unwrap_or_else(|_| {
                                    panic!(
                                        "failed to open updater bundle {}",
                                        updater_path.display()
                                    )
                                }),
                            ));
                            // close server
                            return;
                        }
                        _ => (),
                    }
                }
            }
        });

        config.version = "0.1.0";

        // bundle initial app version
        build_app(&manifest_dir, &config, false, bundle_target);

        let mut binary_cmd = if cfg!(windows) {
            Command::new(root_dir.join("target/debug/app-updater.exe"))
        } else if cfg!(target_os = "macos") {
            Command::new(
                bundle_paths(&root_dir, "0.1.0")
                    .first()
                    .unwrap()
                    .1
                    .join("Contents/MacOS/app-updater"),
            )
        } else if std::env::var("CI").map(|v| v == "true").unwrap_or_default() {
            let mut c = Command::new("xvfb-run");
            c.arg("--auto-servernum")
                .arg(&bundle_paths(&root_dir, "0.1.0").first().unwrap().1);
            c
        } else {
            Command::new(&bundle_paths(&root_dir, "0.1.0").first().unwrap().1)
        };

        binary_cmd.env("TARGET", bundle_target.name());

        let status = binary_cmd.status().expect("failed to run app");

        if !status.success() {
            panic!("failed to run app");
        }
    }
}
