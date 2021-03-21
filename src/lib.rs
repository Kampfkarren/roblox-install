use std::{
    env, fmt, fs, io,
    path::{Path, PathBuf},
};

#[cfg(target_os = "windows")]
use winreg::RegKey;

pub type Result<T> = std::result::Result<T, Error>;

const ROBLOX_STUDIO_PATH_VARIABLE: &'static str = "ROBLOX_STUDIO_PATH";

#[derive(Debug)]
pub enum Error {
    DocumentsDirectoryNotFound,
    MalformedRegistry,
    PlatformNotSupported,
    PluginsDirectoryNotFound,
    RegistryError(io::Error),
    EnvironmentVariableError(String),
    NotInstalled,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DocumentsDirectoryNotFound => write!(
                formatter,
                "Couldn't find Documents directory",
            ),
            Error::PluginsDirectoryNotFound => write!(
                formatter,
                "Couldn't find Plugins directory",
            ),
            Error::MalformedRegistry => write!(
                formatter,
                "The values of the registry keys used to find Roblox are malformed, maybe your Roblox installation is corrupt?",
            ),
            Error::PlatformNotSupported => write!(
                formatter,
                "Your platform is not currently supported",
            ),
            Error::RegistryError(error) => write!(
                formatter,
                "Couldn't find registry keys, Roblox might not be installed. ({})",
                error,
            ),
            Error::EnvironmentVariableError(reason) => write!(
                formatter,
                "environment variable misconfigured: {}",
                reason,
            ),
            Error::NotInstalled => write!(formatter, "Couldn't find Roblox Studio"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Error::RegistryError(error) = self {
            Some(error)
        } else {
            None
        }
    }
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
    pub fn locate() -> Result<RobloxStudio> {
        Self::locate_from_env()
            .unwrap_or_else(|| Self::locate_target_specific())
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

    #[cfg(not(target_os = "macos"))]
    fn locate_plugins_on_windows() -> Result<PathBuf> {
        let mut plugin_dir = dirs::home_dir().ok_or(Error::PluginsDirectoryNotFound)?;
        plugin_dir.push("AppData");
        plugin_dir.push("Local");
        plugin_dir.push("Roblox");
        plugin_dir.push("Plugins");
        Ok(plugin_dir)
    }

    #[cfg(target_os = "macos")]
    fn locate_target_specific() -> Result<RobloxStudio> {
        let mut root = PathBuf::from("/Applications");
        root.push("RobloxStudio.app");
        Self::locate_from_directory(root)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    #[inline]
    fn locate_target_specific() -> Result<RobloxStudio> {
        Err(Error::PlatformNotSupported)
    }

    #[cfg(target_os = "windows")]
    fn locate_from_directory(root: PathBuf) -> Result<RobloxStudio> {
        Self::locate_from_windows_directory(root)
    }

    #[cfg(not(target_os = "macos"))]
    fn locate_from_windows_directory(root: PathBuf) -> Result<RobloxStudio> {
        let content_folder_path = root.join("content");
        let plugins = Self::locate_plugins_on_windows()?;

        if content_folder_path.is_dir() {
            Ok(RobloxStudio {
                content: content_folder_path,
                application: root.join("RobloxStudioBeta.exe"),
                built_in_plugins: root.join("BuiltInPlugins"),
                plugins,
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
                                content: version.join("content"),
                                application,
                                built_in_plugins: version.join("BuiltInPlugins"),
                                plugins: plugins.clone(),
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
        Self::locate_from_windows_directory(root)
            .map_err(|_| Error::PlatformNotSupported)
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
    pub fn application_path(&self) -> &Path {
        &self.application
    }

    #[must_use]
    #[inline]
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
    pub fn built_in_plugins_path(&self) -> &Path {
        &self.built_in_plugins
    }

    #[must_use]
    #[inline]
    pub fn plugins_path(&self) -> &Path {
        &self.plugins
    }

    fn locate_from_env() -> Option<Result<RobloxStudio>> {
        let variable_value = env::var(ROBLOX_STUDIO_PATH_VARIABLE)
            .ok()?;

        let result = variable_value.parse()
            .map_err(|error| {
                Error::EnvironmentVariableError(
                    format!(
                        "could not convert environment variable `{}` to path ({})",
                        ROBLOX_STUDIO_PATH_VARIABLE,
                        error,
                    )
                )
            })
            .and_then(|path: PathBuf| Self::locate_from_directory(path));

        Some(result)
    }
}
