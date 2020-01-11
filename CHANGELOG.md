# roblox-install Changelog

## Unreleased Changes
* Added `RobloxStudio::content_path`, returning the path to the `content` folder.

## 0.2.2
* Fixed locating the Roblox Studio plugins directory on Windows and macOS

## 0.2.1
* Fixed Roblox Studio install detection on macOS

## 0.2.0
* Added `RobloxStudio::plugins_path`
* Added `RobloxStudio::application_path`
* Deprecated `RobloxStudio::root_path` as it is not portable
* Deprecated `RobloxStudio::exe_path` in favor of `application_path`

## 0.1.2
* `Error` now implements `std::error::Error`

## 0.1.1
* `Error` now implements `Display`

## 0.1.0
* Initial release