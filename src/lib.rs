use std::{io, path::PathBuf};

#[cfg(target_os = "windows")]
use winreg::RegKey;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    MalformedRegistry,
    PlatformNotSupported,
    RegistryError(io::Error),
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
    pub fn locate() -> Result<RobloxStudio> {
        Err(Error::PlatformNotSupported)
    }

    #[must_use]
    pub fn exe_path(&self) -> PathBuf {
        self.root.join("RobloxStudioBeta.exe")
    }

    #[must_use]
    pub fn built_in_plugins_path(&self) -> PathBuf {
        self.root.join("BuiltInPlugins")
    }
}
