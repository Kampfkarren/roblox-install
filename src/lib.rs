use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

#[cfg(target_os = "windows")]
use winreg::RegKey;

use thiserror::Error;

/// A wrapper for [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html) that
/// contains [`Error`] in the `Err` type.
pub type Result<T> = std::result::Result<T, Error>;

const ROBLOX_STUDIO_PATH_VARIABLE: &str = "ROBLOX_STUDIO_PATH";

#[derive(Debug, Error)]
#[non_exhaustive]
/// Everything that can go wrong while using roblox-install.
pub enum Error {
    #[error("Couldn't find Documents directory")]
    DocumentsDirectoryNotFound,

    #[error("The values of the registry keys used to find Roblox are malformed, maybe your Roblox installation is corrupt?")]
    MalformedRegistry,

    #[error("Your platform is not currently supported")]
    PlatformNotSupported,

    #[error("Couldn't find Plugins directory")]
    PluginsDirectoryNotFound,

    #[error("Couldn't find registry keys, Roblox might not be installed.")]
    RegistryError(#[source] io::Error),

    #[error("Environment variable misconfigured: {0}")]
    EnvironmentVariableError(String),

    #[error("Couldn't find Roblox Studio")]
    NotInstalled,
}

#[derive(Debug)]
#[must_use]
pub struct RobloxStudio {
    content: PathBuf,
    application: PathBuf,
    built_in_plugins: PathBuf,
    plugins: PathBuf,
    root: PathBuf,
}

impl RobloxStudio {
    /// Attempts to find a Roblox Studio installation. It will start by looking up
    /// into the environment variable `ROBLOX_STUDIO_PATH`. If the variable is not
    /// defined, it will find the usual installation on Windows and MacOS.
    ///
    /// On Windows (or WSL), the environment variable can point to a specific version (where
    /// the `RobloxStudioBeta.exe` file and `content` directory are located) or it
    /// can also point to the Roblox directory in AppData (`$APPDATA\Local\Roblox`)
    /// and it will find the latest version by itself.
    pub fn locate() -> Result<RobloxStudio> {
        Self::locate_from_env().unwrap_or_else(Self::locate_target_specific)
    }

    #[cfg(target_os = "windows")]
    fn locate_target_specific() -> Result<RobloxStudio> {
        let hkcu = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);

        let roblox_studio_reg = hkcu
            .open_subkey(r"Software\Roblox\RobloxStudio")
            .map_err(Error::RegistryError)?;

        let content_folder_value: String = roblox_studio_reg
            .get_value("ContentFolder")
            .map_err(Error::RegistryError)?;

        let content_folder_path = PathBuf::from(content_folder_value);

        let root = content_folder_path
            .parent()
            .ok_or(Error::MalformedRegistry)?
            .to_path_buf();

        let plugins = Self::locate_plugins_on_windows()?;

        Ok(RobloxStudio {
            content: content_folder_path,
            application: root.join("RobloxStudioBeta.exe"),
            built_in_plugins: root.join("BuiltInPlugins"),
            plugins,
            root,
        })
    }

    #[cfg(target_os = "windows")]
    fn locate_plugins_on_windows() -> Result<PathBuf> {
        Ok(
            dirs::home_dir().ok_or(Error::PluginsDirectoryNotFound)?
                .join("AppData")
                .join("Local")
                .join("Roblox")
                .join("Plugins")
        )
    }

    #[cfg(target_os = "macos")]
    fn locate_target_specific() -> Result<RobloxStudio> {
        let mut root = PathBuf::from("/Applications");
        root.push("RobloxStudio.app");
        Self::locate_from_directory(root)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    #[inline]
    fn locate_target_specific() -> Result<RobloxStudio> {
        Err(Error::PlatformNotSupported)
    }

    /// Assumes you are using grapejuice
    #[cfg(target_os = "linux")]
    fn locate_target_specific() -> Result<RobloxStudio> {
        let grapejuice_path =
            PathBuf::from(env::var("HOME").map_err(|_| Error::PlatformNotSupported)?)
                .join(".local")
                .join("share")
                .join("grapejuice")
                .join("wineprefix")
                .join("drive_c")
                .join("users")
                .join(env::var("USER").map_err(|_| Error::PlatformNotSupported)?)
                .join("AppData")
                .join("Local")
                .join("Roblox");
        Self::locate_from_directory(grapejuice_path)
    }

    #[cfg(target_os = "windows")]
    fn locate_from_directory(root: PathBuf) -> Result<RobloxStudio> {
        Self::locate_from_windows_directory(root)
    }

    #[cfg(target_os = "macos")]
    fn locate_from_directory(root: PathBuf) -> Result<RobloxStudio> {
        let contents = root.join("Contents");
        let application = contents.join("MacOS").join("RobloxStudio");
        let built_in_plugins = contents.join("Resources").join("BuiltInPlugins");
        let documents = dirs::document_dir().ok_or(Error::DocumentsDirectoryNotFound)?;
        let plugins = documents.join("Roblox").join("Plugins");
        let content = contents.join("Resources").join("content");

        Ok(RobloxStudio {
            content,
            application,
            built_in_plugins,
            plugins,
            root,
        })
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    #[inline]
    fn locate_from_directory(root: PathBuf) -> Result<RobloxStudio> {
        // for users running WSL, we need to find the Roblox Windows installation
        // even if we're not on Windows
        Self::locate_from_windows_directory(root).map_err(|_| Error::PlatformNotSupported)
    }

    #[cfg(not(target_os = "macos"))]
    fn locate_from_windows_directory(root: PathBuf) -> Result<RobloxStudio> {
        let content_folder_path = root.join("content");

        if content_folder_path.is_dir() {
            Ok(RobloxStudio {
                content: content_folder_path,
                application: root.join("RobloxStudioBeta.exe"),
                built_in_plugins: root.join("BuiltInPlugins"),
                plugins: root.parent().unwrap_or(&root).join("Plugins"),
                root,
            })
        } else {
            let versions = root.join("Versions");

            if versions.is_dir() {
                fs::read_dir(&versions)
                    .map_err(|_| Error::NotInstalled)?
                    .filter_map(|entry| entry.ok())
                    .find_map(|entry| {
                        let version = entry.path();
                        let application = version.join("RobloxStudioBeta.exe");

                        if application.is_file() {
                            Some(RobloxStudio {
                                application,
                                content: version.join("content"),
                                built_in_plugins: version.join("BuiltInPlugins"),
                                plugins: root.join("Plugins"),
                                root: version.to_owned(),
                            })
                        } else {
                            None
                        }
                    })
                    .ok_or(Error::NotInstalled)
            } else {
                Err(Error::NotInstalled)
            }
        }
    }

    #[deprecated(
        since = "0.2.0",
        note = "The contents of the studio directory are inconsistent across platforms. \
        Please use a dedicated method (like application_path) or file a feature request if one does not exist."
    )]
    #[must_use]
    #[inline]
    pub fn root_path(&self) -> &Path {
        &self.root
    }

    #[must_use]
    #[inline]
    /// Path to the Roblox Studio executable
    pub fn application_path(&self) -> &Path {
        &self.application
    }

    #[must_use]
    #[inline]
    /// Path to the content directory
    pub fn content_path(&self) -> &Path {
        &self.content
    }

    #[deprecated(since = "0.2.0", note = "Please use application_path instead.")]
    #[must_use]
    #[inline]
    pub fn exe_path(&self) -> PathBuf {
        self.application_path().to_owned()
    }

    #[must_use]
    #[inline]
    /// Path to built-in plugins directory
    pub fn built_in_plugins_path(&self) -> &Path {
        &self.built_in_plugins
    }

    #[must_use]
    #[inline]
    /// Path to the user's plugin directory. This directory may NOT exist if the Roblox Studio
    /// user has never opened it from Roblox Studio `Plugins Folder` button.
    pub fn plugins_path(&self) -> &Path {
        &self.plugins
    }

    fn locate_from_env() -> Option<Result<RobloxStudio>> {
        Some(
            env::var(ROBLOX_STUDIO_PATH_VARIABLE)
                .ok()?
                .parse()
                .map_err(|error| {
                    Error::EnvironmentVariableError(format!(
                        "could not convert environment variable `{}` to path ({})",
                        ROBLOX_STUDIO_PATH_VARIABLE, error,
                    ))
                })
                .and_then(Self::locate_from_directory),
        )
    }
}
