use std::{
    fmt, io,
    path::{Path, PathBuf},
};

#[cfg(target_os = "windows")]
use winreg::RegKey;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    MalformedRegistry,
    PlatformNotSupported,
    RegistryError(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::MalformedRegistry => write!(formatter, "The values of the registry keys used to find Roblox are malformed, maybe your Roblox installation is corrupt?"),
            Error::PlatformNotSupported => write!(formatter, "Your platform is not currently supported"),
            Error::RegistryError(error) => write!(formatter, "Couldn't find registry keys, Roblox might not be installed. ({})", error),
        }
    }
}

#[derive(Debug)]
#[must_use]
pub struct RobloxStudio {
    root: PathBuf,
}

impl RobloxStudio {
    #[cfg(target_os = "windows")]
    pub fn locate() -> Result<RobloxStudio> {
        let hkcu = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);

        let roblox_studio_reg = hkcu
            .open_subkey(r"Software\Roblox\RobloxStudio")
            .map_err(Error::RegistryError)?;

        let content_folder_value: String = roblox_studio_reg
            .get_value("ContentFolder")
            .map_err(Error::RegistryError)?;

        let content_folder_path = PathBuf::from(content_folder_value);

        Ok(RobloxStudio {
            root: content_folder_path
                .parent()
                .ok_or(Error::MalformedRegistry)?
                .to_owned(),
        })
    }

    #[cfg(not(target_os = "windows"))]
    #[inline]
    pub fn locate() -> Result<RobloxStudio> {
        Err(Error::PlatformNotSupported)
    }

    #[must_use]
    #[inline]
    pub fn root_path(&self) -> &Path {
        &self.root
    }

    #[must_use]
    #[inline]
    pub fn exe_path(&self) -> PathBuf {
        self.root.join("RobloxStudioBeta.exe")
    }

    #[must_use]
    #[inline]
    pub fn built_in_plugins_path(&self) -> PathBuf {
        self.root.join("BuiltInPlugins")
    }
}
