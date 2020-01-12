#![allow(deprecated)]

use std::fs;

use roblox_install::RobloxStudio;

#[cfg(target_os = "windows")]
#[test]
fn test_windows() {
    let studio = RobloxStudio::locate().unwrap();

    assert!(studio.root_path().to_string_lossy().contains("Roblox"));

    assert!(studio
        .built_in_plugins_path()
        .to_string_lossy()
        .contains("BuiltInPlugins"));

    assert!(studio.plugins_path().to_string_lossy().contains("Plugins"));

    assert!(studio
        .application_path()
        .to_string_lossy()
        .contains("RobloxStudioBeta.exe"));

    let content = studio.content_path();
    let meta = fs::metadata(&content).unwrap();
    assert!(meta.is_dir());
    assert!(content.to_string_lossy().contains("content"));
}

#[cfg(target_os = "macos")]
#[test]
fn test_macos() {
    let studio = RobloxStudio::locate().unwrap();

    assert!(studio
        .root_path()
        .to_string_lossy()
        .contains("RobloxStudio.app"));

    assert!(studio
        .built_in_plugins_path()
        .to_string_lossy()
        .contains("BuiltInPlugins"));

    assert!(studio.plugins_path().to_string_lossy().contains("Plugins"));

    assert!(studio
        .application_path()
        .to_string_lossy()
        .contains("RobloxStudio"));

    let content = studio.content_path();
    let meta = fs::metadata(&content).unwrap();
    assert!(meta.is_dir());
    assert!(content.to_string_lossy().contains("content"));
}
