#![allow(deprecated)]

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
}
