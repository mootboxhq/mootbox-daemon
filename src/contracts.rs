#![allow(clippy::unwrap_used)]
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ClaimPairingCodeRequest {
    pub device: ClaimPairingCodeRequestDevice,
    pub name: ClaimPairingCodeRequestName,
    pub user_code: ClaimPairingCodeRequestUserCode,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ClaimPairingCodeRequestDevice {
    pub daemon_version: ClaimPairingCodeRequestDeviceDaemonVersion,
    pub os: ClaimPairingCodeRequestDeviceOs,
    #[serde(deserialize_with = "::std::option::Option::deserialize")]
    pub os_version: ::std::option::Option<ClaimPairingCodeRequestDeviceOsVersion>,
    pub providers: ::std::vec::Vec<ClaimPairingCodeRequestDeviceProvidersItem>,
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ClaimPairingCodeRequestDeviceDaemonVersion(::std::string::String);
impl ::std::ops::Deref for ClaimPairingCodeRequestDeviceDaemonVersion {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ClaimPairingCodeRequestDeviceDaemonVersion> for ::std::string::String {
    fn from(value: ClaimPairingCodeRequestDeviceDaemonVersion) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ClaimPairingCodeRequestDeviceDaemonVersion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ClaimPairingCodeRequestDeviceDaemonVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClaimPairingCodeRequestDeviceDaemonVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ClaimPairingCodeRequestDeviceDaemonVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ClaimPairingCodeRequestDeviceOs {
    #[serde(rename = "macOS")]
    MacOs,
    Linux,
    Windows,
    #[serde(rename = "other")]
    Other,
}
impl ::std::fmt::Display for ClaimPairingCodeRequestDeviceOs {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::MacOs => f.write_str("macOS"),
            Self::Linux => f.write_str("Linux"),
            Self::Windows => f.write_str("Windows"),
            Self::Other => f.write_str("other"),
        }
    }
}
impl ::std::str::FromStr for ClaimPairingCodeRequestDeviceOs {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "macOS" => Ok(Self::MacOs),
            "Linux" => Ok(Self::Linux),
            "Windows" => Ok(Self::Windows),
            "other" => Ok(Self::Other),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ClaimPairingCodeRequestDeviceOs {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClaimPairingCodeRequestDeviceOs {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ClaimPairingCodeRequestDeviceOsVersion(::std::string::String);
impl ::std::ops::Deref for ClaimPairingCodeRequestDeviceOsVersion {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ClaimPairingCodeRequestDeviceOsVersion> for ::std::string::String {
    fn from(value: ClaimPairingCodeRequestDeviceOsVersion) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ClaimPairingCodeRequestDeviceOsVersion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ClaimPairingCodeRequestDeviceOsVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClaimPairingCodeRequestDeviceOsVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ClaimPairingCodeRequestDeviceOsVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ClaimPairingCodeRequestDeviceProvidersItem {
    pub installed: bool,
    pub name: ClaimPairingCodeRequestDeviceProvidersItemName,
    #[serde(deserialize_with = "::std::option::Option::deserialize")]
    pub signed_in: ::std::option::Option<bool>,
    #[serde(deserialize_with = "::std::option::Option::deserialize")]
    pub version: ::std::option::Option<ClaimPairingCodeRequestDeviceProvidersItemVersion>,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ClaimPairingCodeRequestDeviceProvidersItemName {
    #[serde(rename = "claude-code")]
    ClaudeCode,
    #[serde(rename = "codex")]
    Codex,
}
impl ::std::fmt::Display for ClaimPairingCodeRequestDeviceProvidersItemName {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::ClaudeCode => f.write_str("claude-code"),
            Self::Codex => f.write_str("codex"),
        }
    }
}
impl ::std::str::FromStr for ClaimPairingCodeRequestDeviceProvidersItemName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "claude-code" => Ok(Self::ClaudeCode),
            "codex" => Ok(Self::Codex),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ClaimPairingCodeRequestDeviceProvidersItemName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClaimPairingCodeRequestDeviceProvidersItemName {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ClaimPairingCodeRequestDeviceProvidersItemVersion(::std::string::String);
impl ::std::ops::Deref for ClaimPairingCodeRequestDeviceProvidersItemVersion {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ClaimPairingCodeRequestDeviceProvidersItemVersion> for ::std::string::String {
    fn from(value: ClaimPairingCodeRequestDeviceProvidersItemVersion) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ClaimPairingCodeRequestDeviceProvidersItemVersion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ClaimPairingCodeRequestDeviceProvidersItemVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClaimPairingCodeRequestDeviceProvidersItemVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ClaimPairingCodeRequestDeviceProvidersItemVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ClaimPairingCodeRequestName(::std::string::String);
impl ::std::ops::Deref for ClaimPairingCodeRequestName {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ClaimPairingCodeRequestName> for ::std::string::String {
    fn from(value: ClaimPairingCodeRequestName) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ClaimPairingCodeRequestName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ClaimPairingCodeRequestName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClaimPairingCodeRequestName {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ClaimPairingCodeRequestName {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ClaimPairingCodeRequestUserCode(::std::string::String);
impl ::std::ops::Deref for ClaimPairingCodeRequestUserCode {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ClaimPairingCodeRequestUserCode> for ::std::string::String {
    fn from(value: ClaimPairingCodeRequestUserCode) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ClaimPairingCodeRequestUserCode {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 20usize {
            return Err("longer than 20 characters".into());
        }
        if value.chars().count() < 4usize {
            return Err("shorter than 4 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ClaimPairingCodeRequestUserCode {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClaimPairingCodeRequestUserCode {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ClaimPairingCodeRequestUserCode {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ClaimPairingCodeResponse {
    pub credential: ::std::string::String,
    pub runner_id: ::uuid::Uuid,
    pub workspace_id: ::uuid::Uuid,
    pub workspace_name: ::std::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ClaimRequest {
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub active_run_ids: ::std::vec::Vec<::uuid::Uuid>,
    #[serde(default = "defaults::default_u64::<i64, 1>")]
    pub capacity: i64,
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub device: ::std::option::Option<ClaimRequestDevice>,
    pub providers: ::std::vec::Vec<ClaimRequestProvidersItem>,
    #[serde(default = "defaults::default_u64::<i64, 25000>")]
    pub wait_ms: i64,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ClaimRequestDevice {
    pub daemon_version: ClaimRequestDeviceDaemonVersion,
    pub os: ClaimRequestDeviceOs,
    #[serde(deserialize_with = "::std::option::Option::deserialize")]
    pub os_version: ::std::option::Option<ClaimRequestDeviceOsVersion>,
    pub providers: ::std::vec::Vec<ClaimRequestDeviceProvidersItem>,
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ClaimRequestDeviceDaemonVersion(::std::string::String);
impl ::std::ops::Deref for ClaimRequestDeviceDaemonVersion {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ClaimRequestDeviceDaemonVersion> for ::std::string::String {
    fn from(value: ClaimRequestDeviceDaemonVersion) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ClaimRequestDeviceDaemonVersion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ClaimRequestDeviceDaemonVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClaimRequestDeviceDaemonVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ClaimRequestDeviceDaemonVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ClaimRequestDeviceOs {
    #[serde(rename = "macOS")]
    MacOs,
    Linux,
    Windows,
    #[serde(rename = "other")]
    Other,
}
impl ::std::fmt::Display for ClaimRequestDeviceOs {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::MacOs => f.write_str("macOS"),
            Self::Linux => f.write_str("Linux"),
            Self::Windows => f.write_str("Windows"),
            Self::Other => f.write_str("other"),
        }
    }
}
impl ::std::str::FromStr for ClaimRequestDeviceOs {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "macOS" => Ok(Self::MacOs),
            "Linux" => Ok(Self::Linux),
            "Windows" => Ok(Self::Windows),
            "other" => Ok(Self::Other),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ClaimRequestDeviceOs {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClaimRequestDeviceOs {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ClaimRequestDeviceOsVersion(::std::string::String);
impl ::std::ops::Deref for ClaimRequestDeviceOsVersion {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ClaimRequestDeviceOsVersion> for ::std::string::String {
    fn from(value: ClaimRequestDeviceOsVersion) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ClaimRequestDeviceOsVersion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ClaimRequestDeviceOsVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClaimRequestDeviceOsVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ClaimRequestDeviceOsVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ClaimRequestDeviceProvidersItem {
    pub installed: bool,
    pub name: ClaimRequestDeviceProvidersItemName,
    #[serde(deserialize_with = "::std::option::Option::deserialize")]
    pub signed_in: ::std::option::Option<bool>,
    #[serde(deserialize_with = "::std::option::Option::deserialize")]
    pub version: ::std::option::Option<ClaimRequestDeviceProvidersItemVersion>,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ClaimRequestDeviceProvidersItemName {
    #[serde(rename = "claude-code")]
    ClaudeCode,
    #[serde(rename = "codex")]
    Codex,
}
impl ::std::fmt::Display for ClaimRequestDeviceProvidersItemName {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::ClaudeCode => f.write_str("claude-code"),
            Self::Codex => f.write_str("codex"),
        }
    }
}
impl ::std::str::FromStr for ClaimRequestDeviceProvidersItemName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "claude-code" => Ok(Self::ClaudeCode),
            "codex" => Ok(Self::Codex),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ClaimRequestDeviceProvidersItemName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClaimRequestDeviceProvidersItemName {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ClaimRequestDeviceProvidersItemVersion(::std::string::String);
impl ::std::ops::Deref for ClaimRequestDeviceProvidersItemVersion {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ClaimRequestDeviceProvidersItemVersion> for ::std::string::String {
    fn from(value: ClaimRequestDeviceProvidersItemVersion) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ClaimRequestDeviceProvidersItemVersion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ClaimRequestDeviceProvidersItemVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClaimRequestDeviceProvidersItemVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ClaimRequestDeviceProvidersItemVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ClaimRequestProvidersItem {
    #[serde(rename = "claude-code")]
    ClaudeCode,
    #[serde(rename = "codex")]
    Codex,
}
impl ::std::fmt::Display for ClaimRequestProvidersItem {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::ClaudeCode => f.write_str("claude-code"),
            Self::Codex => f.write_str("codex"),
        }
    }
}
impl ::std::str::FromStr for ClaimRequestProvidersItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "claude-code" => Ok(Self::ClaudeCode),
            "codex" => Ok(Self::Codex),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ClaimRequestProvidersItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClaimRequestProvidersItem {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
#[serde(tag = "status")]
pub enum ClaimResponse {
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "reconcile_required")]
    ReconcileRequired {
        run_ids: ::std::vec::Vec<::uuid::Uuid>,
        runs: ::std::vec::Vec<ClaimResponseRunsItem>,
    },
    #[serde(rename = "claimed")]
    Claimed {
        run: ::std::boxed::Box<ClaimResponseClaimedRun>,
    },
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ClaimResponseClaimedRun {
    pub branch: ::std::string::String,
    pub chat_id: ::uuid::Uuid,
    pub connection_id: ::uuid::Uuid,
    pub generation: i64,
    pub lease_expires_at: ::chrono::DateTime<::chrono::offset::Utc>,
    pub manifest: ClaimResponseClaimedRunManifest,
    pub manifest_hash: ::std::string::String,
    pub provider: ClaimResponseClaimedRunProvider,
    pub repository_id: ::uuid::Uuid,
    pub run_id: ::uuid::Uuid,
    pub run_token: ::std::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ClaimResponseClaimedRunManifest {
    pub chat: ClaimResponseClaimedRunManifestChat,
    pub context: ClaimResponseClaimedRunManifestContext,
    pub context_messages: ::std::vec::Vec<ClaimResponseClaimedRunManifestContextMessagesItem>,
    pub instruction: ClaimResponseClaimedRunManifestInstruction,
    pub permitted_tools: ::std::vec::Vec<::std::string::String>,
    pub provider: ClaimResponseClaimedRunManifestProvider,
    pub resume: bool,
    pub run: ClaimResponseClaimedRunManifestRun,
    pub session_key: ::uuid::Uuid,
    pub target: ClaimResponseClaimedRunManifestTarget,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ClaimResponseClaimedRunManifestChat {
    pub id: ::uuid::Uuid,
    pub project_name: ::std::string::String,
    pub title: ::std::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ClaimResponseClaimedRunManifestContext {
    #[serde(deserialize_with = "::std::option::Option::deserialize")]
    pub description: ::std::option::Option<::std::string::String>,
    #[serde(deserialize_with = "::std::option::Option::deserialize")]
    pub instructions: ::std::option::Option<::std::string::String>,
    pub linked_chats: ::std::vec::Vec<ClaimResponseClaimedRunManifestContextLinkedChatsItem>,
    pub pinned_files: ::std::vec::Vec<::std::string::String>,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ClaimResponseClaimedRunManifestContextLinkedChatsItem {
    pub project: ::std::string::String,
    pub title: ::std::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ClaimResponseClaimedRunManifestContextMessagesItem {
    pub author: ::std::string::String,
    pub author_kind: ClaimResponseClaimedRunManifestContextMessagesItemAuthorKind,
    pub body: ::std::string::String,
    pub created_at: ::chrono::DateTime<::chrono::offset::Utc>,
    pub id: ::uuid::Uuid,
    #[serde(deserialize_with = "::std::option::Option::deserialize")]
    pub reply_to_message_id: ::std::option::Option<::uuid::Uuid>,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ClaimResponseClaimedRunManifestContextMessagesItemAuthorKind {
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "human")]
    Human,
}
impl ::std::fmt::Display for ClaimResponseClaimedRunManifestContextMessagesItemAuthorKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Assistant => f.write_str("assistant"),
            Self::Human => f.write_str("human"),
        }
    }
}
impl ::std::str::FromStr for ClaimResponseClaimedRunManifestContextMessagesItemAuthorKind {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "assistant" => Ok(Self::Assistant),
            "human" => Ok(Self::Human),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ClaimResponseClaimedRunManifestContextMessagesItemAuthorKind {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClaimResponseClaimedRunManifestContextMessagesItemAuthorKind {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ClaimResponseClaimedRunManifestInstruction(::std::string::String);
impl ::std::ops::Deref for ClaimResponseClaimedRunManifestInstruction {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ClaimResponseClaimedRunManifestInstruction> for ::std::string::String {
    fn from(value: ClaimResponseClaimedRunManifestInstruction) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ClaimResponseClaimedRunManifestInstruction {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ClaimResponseClaimedRunManifestInstruction {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClaimResponseClaimedRunManifestInstruction {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ClaimResponseClaimedRunManifestInstruction {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ClaimResponseClaimedRunManifestProvider {
    #[serde(rename = "claude-code")]
    ClaudeCode,
    #[serde(rename = "codex")]
    Codex,
}
impl ::std::fmt::Display for ClaimResponseClaimedRunManifestProvider {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::ClaudeCode => f.write_str("claude-code"),
            Self::Codex => f.write_str("codex"),
        }
    }
}
impl ::std::str::FromStr for ClaimResponseClaimedRunManifestProvider {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "claude-code" => Ok(Self::ClaudeCode),
            "codex" => Ok(Self::Codex),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ClaimResponseClaimedRunManifestProvider {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClaimResponseClaimedRunManifestProvider {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ClaimResponseClaimedRunManifestRun {
    pub branch: ::std::string::String,
    pub generation: i64,
    pub id: ::uuid::Uuid,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ClaimResponseClaimedRunManifestTarget {
    pub base_ref: ::std::string::String,
    pub remote_url: ::std::string::String,
    pub repository_id: ::uuid::Uuid,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ClaimResponseClaimedRunProvider {
    #[serde(rename = "claude-code")]
    ClaudeCode,
    #[serde(rename = "codex")]
    Codex,
}
impl ::std::fmt::Display for ClaimResponseClaimedRunProvider {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::ClaudeCode => f.write_str("claude-code"),
            Self::Codex => f.write_str("codex"),
        }
    }
}
impl ::std::str::FromStr for ClaimResponseClaimedRunProvider {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "claude-code" => Ok(Self::ClaudeCode),
            "codex" => Ok(Self::Codex),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ClaimResponseClaimedRunProvider {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ClaimResponseClaimedRunProvider {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ClaimResponseRunsItem {
    pub generation: ::std::num::NonZeroU64,
    pub run_id: ::uuid::Uuid,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct CompleteRunRequest {
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub changes: ::std::option::Option<CompleteRunRequestChanges>,
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub effective_model: ::std::option::Option<CompleteRunRequestEffectiveModel>,
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub error: ::std::option::Option<CompleteRunRequestError>,
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub estimated_cost_usd: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "::serde_json::Map::is_empty")]
    pub exit: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub final_answer: ::std::option::Option<CompleteRunRequestFinalAnswer>,
    pub generation: ::std::num::NonZeroU64,
    pub outcome: CompleteRunRequestOutcome,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct CompleteRunRequestChanges {
    pub commits: ::std::vec::Vec<CompleteRunRequestChangesCommitsItem>,
    pub files: ::std::vec::Vec<CompleteRunRequestChangesFilesItem>,
    pub files_truncated: bool,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct CompleteRunRequestChangesCommitsItem {
    pub sha: CompleteRunRequestChangesCommitsItemSha,
    pub subject: CompleteRunRequestChangesCommitsItemSubject,
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct CompleteRunRequestChangesCommitsItemSha(::std::string::String);
impl ::std::ops::Deref for CompleteRunRequestChangesCommitsItemSha {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CompleteRunRequestChangesCommitsItemSha> for ::std::string::String {
    fn from(value: CompleteRunRequestChangesCommitsItemSha) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CompleteRunRequestChangesCommitsItemSha {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^[0-9a-f]{40}([0-9a-f]{24})?$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^[0-9a-f]{40}([0-9a-f]{24})?$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CompleteRunRequestChangesCommitsItemSha {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CompleteRunRequestChangesCommitsItemSha {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CompleteRunRequestChangesCommitsItemSha {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct CompleteRunRequestChangesCommitsItemSubject(::std::string::String);
impl ::std::ops::Deref for CompleteRunRequestChangesCommitsItemSubject {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CompleteRunRequestChangesCommitsItemSubject> for ::std::string::String {
    fn from(value: CompleteRunRequestChangesCommitsItemSubject) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CompleteRunRequestChangesCommitsItemSubject {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 200usize {
            return Err("longer than 200 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CompleteRunRequestChangesCommitsItemSubject {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CompleteRunRequestChangesCommitsItemSubject {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CompleteRunRequestChangesCommitsItemSubject {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct CompleteRunRequestChangesFilesItem {
    #[serde(deserialize_with = "::std::option::Option::deserialize")]
    pub additions: ::std::option::Option<i64>,
    pub binary: bool,
    #[serde(deserialize_with = "::std::option::Option::deserialize")]
    pub deletions: ::std::option::Option<i64>,
    #[serde(deserialize_with = "::std::option::Option::deserialize")]
    pub patch: ::std::option::Option<CompleteRunRequestChangesFilesItemPatch>,
    pub patch_truncated: bool,
    pub path: CompleteRunRequestChangesFilesItemPath,
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub previous_path: ::std::option::Option<CompleteRunRequestChangesFilesItemPreviousPath>,
    pub status: CompleteRunRequestChangesFilesItemStatus,
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct CompleteRunRequestChangesFilesItemPatch(::std::string::String);
impl ::std::ops::Deref for CompleteRunRequestChangesFilesItemPatch {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CompleteRunRequestChangesFilesItemPatch> for ::std::string::String {
    fn from(value: CompleteRunRequestChangesFilesItemPatch) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CompleteRunRequestChangesFilesItemPatch {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 65536usize {
            return Err("longer than 65536 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CompleteRunRequestChangesFilesItemPatch {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CompleteRunRequestChangesFilesItemPatch {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CompleteRunRequestChangesFilesItemPatch {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct CompleteRunRequestChangesFilesItemPath(::std::string::String);
impl ::std::ops::Deref for CompleteRunRequestChangesFilesItemPath {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CompleteRunRequestChangesFilesItemPath> for ::std::string::String {
    fn from(value: CompleteRunRequestChangesFilesItemPath) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CompleteRunRequestChangesFilesItemPath {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 1024usize {
            return Err("longer than 1024 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CompleteRunRequestChangesFilesItemPath {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CompleteRunRequestChangesFilesItemPath {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CompleteRunRequestChangesFilesItemPath {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct CompleteRunRequestChangesFilesItemPreviousPath(::std::string::String);
impl ::std::ops::Deref for CompleteRunRequestChangesFilesItemPreviousPath {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CompleteRunRequestChangesFilesItemPreviousPath> for ::std::string::String {
    fn from(value: CompleteRunRequestChangesFilesItemPreviousPath) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CompleteRunRequestChangesFilesItemPreviousPath {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 1024usize {
            return Err("longer than 1024 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CompleteRunRequestChangesFilesItemPreviousPath {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CompleteRunRequestChangesFilesItemPreviousPath {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CompleteRunRequestChangesFilesItemPreviousPath {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CompleteRunRequestChangesFilesItemStatus {
    #[serde(rename = "added")]
    Added,
    #[serde(rename = "modified")]
    Modified,
    #[serde(rename = "deleted")]
    Deleted,
    #[serde(rename = "renamed")]
    Renamed,
}
impl ::std::fmt::Display for CompleteRunRequestChangesFilesItemStatus {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Added => f.write_str("added"),
            Self::Modified => f.write_str("modified"),
            Self::Deleted => f.write_str("deleted"),
            Self::Renamed => f.write_str("renamed"),
        }
    }
}
impl ::std::str::FromStr for CompleteRunRequestChangesFilesItemStatus {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "added" => Ok(Self::Added),
            "modified" => Ok(Self::Modified),
            "deleted" => Ok(Self::Deleted),
            "renamed" => Ok(Self::Renamed),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CompleteRunRequestChangesFilesItemStatus {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CompleteRunRequestChangesFilesItemStatus {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct CompleteRunRequestEffectiveModel(::std::string::String);
impl ::std::ops::Deref for CompleteRunRequestEffectiveModel {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CompleteRunRequestEffectiveModel> for ::std::string::String {
    fn from(value: CompleteRunRequestEffectiveModel) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CompleteRunRequestEffectiveModel {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 200usize {
            return Err("longer than 200 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CompleteRunRequestEffectiveModel {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CompleteRunRequestEffectiveModel {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CompleteRunRequestEffectiveModel {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct CompleteRunRequestError(::std::string::String);
impl ::std::ops::Deref for CompleteRunRequestError {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CompleteRunRequestError> for ::std::string::String {
    fn from(value: CompleteRunRequestError) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CompleteRunRequestError {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 2000usize {
            return Err("longer than 2000 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CompleteRunRequestError {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CompleteRunRequestError {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CompleteRunRequestError {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct CompleteRunRequestFinalAnswer(::std::string::String);
impl ::std::ops::Deref for CompleteRunRequestFinalAnswer {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<CompleteRunRequestFinalAnswer> for ::std::string::String {
    fn from(value: CompleteRunRequestFinalAnswer) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for CompleteRunRequestFinalAnswer {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 20000usize {
            return Err("longer than 20000 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for CompleteRunRequestFinalAnswer {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CompleteRunRequestFinalAnswer {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for CompleteRunRequestFinalAnswer {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CompleteRunRequestOutcome {
    #[serde(rename = "succeeded")]
    Succeeded,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
}
impl ::std::fmt::Display for CompleteRunRequestOutcome {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Succeeded => f.write_str("succeeded"),
            Self::Failed => f.write_str("failed"),
            Self::Cancelled => f.write_str("cancelled"),
        }
    }
}
impl ::std::str::FromStr for CompleteRunRequestOutcome {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CompleteRunRequestOutcome {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CompleteRunRequestOutcome {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct CompleteRunResponse {
    #[serde(deserialize_with = "::std::option::Option::deserialize")]
    pub failure_reason: ::std::option::Option<::std::string::String>,
    pub run_id: ::uuid::Uuid,
    pub state: CompleteRunResponseState,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CompleteRunResponseState {
    #[serde(rename = "claimed")]
    Claimed,
    #[serde(rename = "preparing")]
    Preparing,
    #[serde(rename = "executing")]
    Executing,
    #[serde(rename = "waiting_for_input")]
    WaitingForInput,
    #[serde(rename = "succeeded")]
    Succeeded,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancel_requested")]
    CancelRequested,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "lost")]
    Lost,
}
impl ::std::fmt::Display for CompleteRunResponseState {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Claimed => f.write_str("claimed"),
            Self::Preparing => f.write_str("preparing"),
            Self::Executing => f.write_str("executing"),
            Self::WaitingForInput => f.write_str("waiting_for_input"),
            Self::Succeeded => f.write_str("succeeded"),
            Self::Failed => f.write_str("failed"),
            Self::CancelRequested => f.write_str("cancel_requested"),
            Self::Cancelled => f.write_str("cancelled"),
            Self::Lost => f.write_str("lost"),
        }
    }
}
impl ::std::str::FromStr for CompleteRunResponseState {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "claimed" => Ok(Self::Claimed),
            "preparing" => Ok(Self::Preparing),
            "executing" => Ok(Self::Executing),
            "waiting_for_input" => Ok(Self::WaitingForInput),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "cancel_requested" => Ok(Self::CancelRequested),
            "cancelled" => Ok(Self::Cancelled),
            "lost" => Ok(Self::Lost),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CompleteRunResponseState {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CompleteRunResponseState {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct Error {
    pub error: ErrorError,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ErrorError {
    pub category: ErrorErrorCategory,
    pub message: ::std::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorErrorCategory {
    #[serde(rename = "permission_denied")]
    PermissionDenied,
    #[serde(rename = "version_conflict")]
    VersionConflict,
    #[serde(rename = "invalid_transition")]
    InvalidTransition,
    #[serde(rename = "idempotency_conflict")]
    IdempotencyConflict,
    #[serde(rename = "not_found")]
    NotFound,
    #[serde(rename = "retryable")]
    Retryable,
    #[serde(rename = "rate_limited")]
    RateLimited,
    #[serde(rename = "invalid_input")]
    InvalidInput,
    #[serde(rename = "lease_lost")]
    LeaseLost,
    #[serde(rename = "upgrade_required")]
    UpgradeRequired,
    #[serde(rename = "workspace_not_selected")]
    WorkspaceNotSelected,
    #[serde(rename = "repository_not_found")]
    RepositoryNotFound,
}
impl ::std::fmt::Display for ErrorErrorCategory {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::PermissionDenied => f.write_str("permission_denied"),
            Self::VersionConflict => f.write_str("version_conflict"),
            Self::InvalidTransition => f.write_str("invalid_transition"),
            Self::IdempotencyConflict => f.write_str("idempotency_conflict"),
            Self::NotFound => f.write_str("not_found"),
            Self::Retryable => f.write_str("retryable"),
            Self::RateLimited => f.write_str("rate_limited"),
            Self::InvalidInput => f.write_str("invalid_input"),
            Self::LeaseLost => f.write_str("lease_lost"),
            Self::UpgradeRequired => f.write_str("upgrade_required"),
            Self::WorkspaceNotSelected => f.write_str("workspace_not_selected"),
            Self::RepositoryNotFound => f.write_str("repository_not_found"),
        }
    }
}
impl ::std::str::FromStr for ErrorErrorCategory {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "permission_denied" => Ok(Self::PermissionDenied),
            "version_conflict" => Ok(Self::VersionConflict),
            "invalid_transition" => Ok(Self::InvalidTransition),
            "idempotency_conflict" => Ok(Self::IdempotencyConflict),
            "not_found" => Ok(Self::NotFound),
            "retryable" => Ok(Self::Retryable),
            "rate_limited" => Ok(Self::RateLimited),
            "invalid_input" => Ok(Self::InvalidInput),
            "lease_lost" => Ok(Self::LeaseLost),
            "upgrade_required" => Ok(Self::UpgradeRequired),
            "workspace_not_selected" => Ok(Self::WorkspaceNotSelected),
            "repository_not_found" => Ok(Self::RepositoryNotFound),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ErrorErrorCategory {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ErrorErrorCategory {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ExchangeCliLoginRequest {
    pub device_secret: ExchangeCliLoginRequestDeviceSecret,
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ExchangeCliLoginRequestDeviceSecret(::std::string::String);
impl ::std::ops::Deref for ExchangeCliLoginRequestDeviceSecret {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ExchangeCliLoginRequestDeviceSecret> for ::std::string::String {
    fn from(value: ExchangeCliLoginRequestDeviceSecret) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ExchangeCliLoginRequestDeviceSecret {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 200usize {
            return Err("longer than 200 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ExchangeCliLoginRequestDeviceSecret {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ExchangeCliLoginRequestDeviceSecret {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ExchangeCliLoginRequestDeviceSecret {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
#[serde(tag = "status")]
pub enum ExchangeCliLoginResponse {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "signed_in")]
    SignedIn {
        session_token: ::std::string::String,
        user: ::std::boxed::Box<ExchangeCliLoginResponseSignedInUser>,
    },
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ExchangeCliLoginResponseSignedInUser {
    pub email: ::std::string::String,
    #[serde(deserialize_with = "::std::option::Option::deserialize")]
    pub name: ::std::option::Option<::std::string::String>,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ExchangePairingRequest {
    pub device_secret: ExchangePairingRequestDeviceSecret,
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ExchangePairingRequestDeviceSecret(::std::string::String);
impl ::std::ops::Deref for ExchangePairingRequestDeviceSecret {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ExchangePairingRequestDeviceSecret> for ::std::string::String {
    fn from(value: ExchangePairingRequestDeviceSecret) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for ExchangePairingRequestDeviceSecret {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 200usize {
            return Err("longer than 200 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ExchangePairingRequestDeviceSecret {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ExchangePairingRequestDeviceSecret {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ExchangePairingRequestDeviceSecret {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
#[serde(tag = "status")]
pub enum ExchangePairingResponse {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "paired")]
    Paired {
        credential: ::std::string::String,
        runner_id: ::uuid::Uuid,
        workspace_id: ::uuid::Uuid,
        workspace_name: ::std::string::String,
    },
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
#[serde(tag = "status")]
pub enum ExchangeRepositoryLinkResponse {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "linked")]
    Linked {
        chat_url: ::std::string::String,
        project: ::std::boxed::Box<ExchangeRepositoryLinkResponseLinkedProject>,
        repository_id: ::uuid::Uuid,
        #[serde(skip_serializing_if = "::std::option::Option::is_none")]
        runner: ::std::option::Option<ExchangeRepositoryLinkResponseLinkedRunner>,
        workspace: ::std::boxed::Box<ExchangeRepositoryLinkResponseLinkedWorkspace>,
    },
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ExchangeRepositoryLinkResponseLinkedProject {
    pub id: ::uuid::Uuid,
    pub name: ::std::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ExchangeRepositoryLinkResponseLinkedRunner {
    pub credential: ::std::string::String,
    pub runner_id: ::uuid::Uuid,
    pub workspace_id: ::uuid::Uuid,
    pub workspace_name: ::std::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ExchangeRepositoryLinkResponseLinkedWorkspace {
    pub id: ::uuid::Uuid,
    pub name: ::std::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct HealthResponse {
    pub protocol_version: ::std::num::NonZeroU64,
    pub runner_id: ::uuid::Uuid,
    pub server_time: ::chrono::DateTime<::chrono::offset::Utc>,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct HeartbeatRequest {
    pub generation: ::std::num::NonZeroU64,
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub state: ::std::option::Option<HeartbeatRequestState>,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum HeartbeatRequestState {
    #[serde(rename = "preparing")]
    Preparing,
    #[serde(rename = "executing")]
    Executing,
}
impl ::std::fmt::Display for HeartbeatRequestState {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Preparing => f.write_str("preparing"),
            Self::Executing => f.write_str("executing"),
        }
    }
}
impl ::std::str::FromStr for HeartbeatRequestState {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "preparing" => Ok(Self::Preparing),
            "executing" => Ok(Self::Executing),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for HeartbeatRequestState {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for HeartbeatRequestState {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct HeartbeatResponse {
    pub cancel_requested: bool,
    pub lease_expires_at: ::chrono::DateTime<::chrono::offset::Utc>,
    pub reassessment_requested: bool,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct MatchRepositoriesRequest {
    pub remotes: ::std::vec::Vec<MatchRepositoriesRequestRemotesItem>,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct MatchRepositoriesRequestRemotesItem {
    pub name: MatchRepositoriesRequestRemotesItemName,
    pub url: MatchRepositoriesRequestRemotesItemUrl,
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct MatchRepositoriesRequestRemotesItemName(::std::string::String);
impl ::std::ops::Deref for MatchRepositoriesRequestRemotesItemName {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MatchRepositoriesRequestRemotesItemName> for ::std::string::String {
    fn from(value: MatchRepositoriesRequestRemotesItemName) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MatchRepositoriesRequestRemotesItemName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MatchRepositoriesRequestRemotesItemName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MatchRepositoriesRequestRemotesItemName {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MatchRepositoriesRequestRemotesItemName {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct MatchRepositoriesRequestRemotesItemUrl(::std::string::String);
impl ::std::ops::Deref for MatchRepositoriesRequestRemotesItemUrl {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<MatchRepositoriesRequestRemotesItemUrl> for ::std::string::String {
    fn from(value: MatchRepositoriesRequestRemotesItemUrl) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for MatchRepositoriesRequestRemotesItemUrl {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 500usize {
            return Err("longer than 500 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for MatchRepositoriesRequestRemotesItemUrl {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MatchRepositoriesRequestRemotesItemUrl {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for MatchRepositoriesRequestRemotesItemUrl {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
#[serde(tag = "kind")]
pub enum MatchRepositoriesResponse {
    #[serde(rename = "one")]
    One {
        #[serde(rename = "match")]
        match_: ::std::boxed::Box<MatchRepositoriesResponseOneMatch>,
    },
    #[serde(rename = "many")]
    Many {
        matches: ::std::vec::Vec<MatchRepositoriesResponseMatchesItem>,
    },
    #[serde(rename = "none")]
    None {
        registrable: ::std::vec::Vec<MatchRepositoriesResponseRegistrableItem>,
    },
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct MatchRepositoriesResponseMatchesItem {
    pub project: MatchRepositoriesResponseMatchesItemProject,
    pub remote_name: ::std::string::String,
    pub repository_id: ::uuid::Uuid,
    pub repository_name: ::std::string::String,
    pub team: MatchRepositoriesResponseMatchesItemTeam,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct MatchRepositoriesResponseMatchesItemProject {
    pub id: ::uuid::Uuid,
    pub name: ::std::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct MatchRepositoriesResponseMatchesItemTeam {
    pub id: ::uuid::Uuid,
    pub name: ::std::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct MatchRepositoriesResponseOneMatch {
    pub project: MatchRepositoriesResponseOneMatchProject,
    pub remote_name: ::std::string::String,
    pub repository_id: ::uuid::Uuid,
    pub repository_name: ::std::string::String,
    pub team: MatchRepositoriesResponseOneMatchTeam,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct MatchRepositoriesResponseOneMatchProject {
    pub id: ::uuid::Uuid,
    pub name: ::std::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct MatchRepositoriesResponseOneMatchTeam {
    pub id: ::uuid::Uuid,
    pub name: ::std::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct MatchRepositoriesResponseRegistrableItem {
    pub projects: ::std::vec::Vec<MatchRepositoriesResponseRegistrableItemProjectsItem>,
    pub team_id: ::uuid::Uuid,
    pub team_name: ::std::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct MatchRepositoriesResponseRegistrableItemProjectsItem {
    pub id: ::uuid::Uuid,
    pub name: ::std::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct RegisterRepositoryRequest {
    #[serde(default = "defaults::register_repository_request_default_branch")]
    pub default_branch: RegisterRepositoryRequestDefaultBranch,
    pub name: RegisterRepositoryRequestName,
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub new_project: ::std::option::Option<RegisterRepositoryRequestNewProject>,
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub project_id: ::std::option::Option<::uuid::Uuid>,
    pub remote_url: RegisterRepositoryRequestRemoteUrl,
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RegisterRepositoryRequestDefaultBranch(::std::string::String);
impl ::std::ops::Deref for RegisterRepositoryRequestDefaultBranch {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RegisterRepositoryRequestDefaultBranch> for ::std::string::String {
    fn from(value: RegisterRepositoryRequestDefaultBranch) -> Self {
        value.0
    }
}
impl ::std::default::Default for RegisterRepositoryRequestDefaultBranch {
    fn default() -> Self {
        RegisterRepositoryRequestDefaultBranch("main".to_string())
    }
}
impl ::std::str::FromStr for RegisterRepositoryRequestDefaultBranch {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 200usize {
            return Err("longer than 200 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RegisterRepositoryRequestDefaultBranch {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RegisterRepositoryRequestDefaultBranch {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RegisterRepositoryRequestDefaultBranch {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RegisterRepositoryRequestName(::std::string::String);
impl ::std::ops::Deref for RegisterRepositoryRequestName {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RegisterRepositoryRequestName> for ::std::string::String {
    fn from(value: RegisterRepositoryRequestName) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RegisterRepositoryRequestName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RegisterRepositoryRequestName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RegisterRepositoryRequestName {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RegisterRepositoryRequestName {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct RegisterRepositoryRequestNewProject {
    pub name: RegisterRepositoryRequestNewProjectName,
    pub team_id: ::uuid::Uuid,
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RegisterRepositoryRequestNewProjectName(::std::string::String);
impl ::std::ops::Deref for RegisterRepositoryRequestNewProjectName {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RegisterRepositoryRequestNewProjectName> for ::std::string::String {
    fn from(value: RegisterRepositoryRequestNewProjectName) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RegisterRepositoryRequestNewProjectName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RegisterRepositoryRequestNewProjectName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RegisterRepositoryRequestNewProjectName {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RegisterRepositoryRequestNewProjectName {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RegisterRepositoryRequestRemoteUrl(::std::string::String);
impl ::std::ops::Deref for RegisterRepositoryRequestRemoteUrl {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<RegisterRepositoryRequestRemoteUrl> for ::std::string::String {
    fn from(value: RegisterRepositoryRequestRemoteUrl) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for RegisterRepositoryRequestRemoteUrl {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 500usize {
            return Err("longer than 500 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for RegisterRepositoryRequestRemoteUrl {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RegisterRepositoryRequestRemoteUrl {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for RegisterRepositoryRequestRemoteUrl {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct RegisterRepositoryResponse {
    pub project_id: ::uuid::Uuid,
    pub repository_id: ::uuid::Uuid,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct RunEventRequest {
    pub generation: ::std::num::NonZeroU64,
    pub payload: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    #[serde(rename = "type")]
    pub type_: RunEventRequestType,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RunEventRequestType {
    #[serde(rename = "checkpoint")]
    Checkpoint,
    #[serde(rename = "progress")]
    Progress,
}
impl ::std::fmt::Display for RunEventRequestType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Checkpoint => f.write_str("checkpoint"),
            Self::Progress => f.write_str("progress"),
        }
    }
}
impl ::std::str::FromStr for RunEventRequestType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "checkpoint" => Ok(Self::Checkpoint),
            "progress" => Ok(Self::Progress),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for RunEventRequestType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RunEventRequestType {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct RunEventResponse {
    pub recorded: bool,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct StartCliLoginRequest {
    pub name: StartCliLoginRequestName,
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct StartCliLoginRequestName(::std::string::String);
impl ::std::ops::Deref for StartCliLoginRequestName {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<StartCliLoginRequestName> for ::std::string::String {
    fn from(value: StartCliLoginRequestName) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for StartCliLoginRequestName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for StartCliLoginRequestName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for StartCliLoginRequestName {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for StartCliLoginRequestName {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct StartCliLoginResponse {
    pub confirm_url: ::std::string::String,
    pub device_secret: ::std::string::String,
    pub expires_at: ::chrono::DateTime<::chrono::offset::Utc>,
    pub login_id: ::uuid::Uuid,
    pub user_code: ::std::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct StartPairingRequest {
    pub name: StartPairingRequestName,
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct StartPairingRequestName(::std::string::String);
impl ::std::ops::Deref for StartPairingRequestName {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<StartPairingRequestName> for ::std::string::String {
    fn from(value: StartPairingRequestName) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for StartPairingRequestName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for StartPairingRequestName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for StartPairingRequestName {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for StartPairingRequestName {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct StartPairingResponse {
    pub device_secret: ::std::string::String,
    pub expires_at: ::chrono::DateTime<::chrono::offset::Utc>,
    pub pairing_id: ::uuid::Uuid,
    pub user_code: ::std::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct StartRepositoryLinkRequest {
    #[serde(default = "defaults::start_repository_link_request_default_branch")]
    pub default_branch: StartRepositoryLinkRequestDefaultBranch,
    pub device: StartRepositoryLinkRequestDevice,
    pub name: StartRepositoryLinkRequestName,
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub paired_workspace_ids: ::std::vec::Vec<::uuid::Uuid>,
    pub remotes: ::std::vec::Vec<StartRepositoryLinkRequestRemotesItem>,
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct StartRepositoryLinkRequestDefaultBranch(::std::string::String);
impl ::std::ops::Deref for StartRepositoryLinkRequestDefaultBranch {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<StartRepositoryLinkRequestDefaultBranch> for ::std::string::String {
    fn from(value: StartRepositoryLinkRequestDefaultBranch) -> Self {
        value.0
    }
}
impl ::std::default::Default for StartRepositoryLinkRequestDefaultBranch {
    fn default() -> Self {
        StartRepositoryLinkRequestDefaultBranch("main".to_string())
    }
}
impl ::std::str::FromStr for StartRepositoryLinkRequestDefaultBranch {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 200usize {
            return Err("longer than 200 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for StartRepositoryLinkRequestDefaultBranch {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for StartRepositoryLinkRequestDefaultBranch {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for StartRepositoryLinkRequestDefaultBranch {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct StartRepositoryLinkRequestDevice {
    pub daemon_version: StartRepositoryLinkRequestDeviceDaemonVersion,
    pub name: StartRepositoryLinkRequestDeviceName,
    pub os: StartRepositoryLinkRequestDeviceOs,
    #[serde(deserialize_with = "::std::option::Option::deserialize")]
    pub os_version: ::std::option::Option<StartRepositoryLinkRequestDeviceOsVersion>,
    pub providers: ::std::vec::Vec<StartRepositoryLinkRequestDeviceProvidersItem>,
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct StartRepositoryLinkRequestDeviceDaemonVersion(::std::string::String);
impl ::std::ops::Deref for StartRepositoryLinkRequestDeviceDaemonVersion {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<StartRepositoryLinkRequestDeviceDaemonVersion> for ::std::string::String {
    fn from(value: StartRepositoryLinkRequestDeviceDaemonVersion) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for StartRepositoryLinkRequestDeviceDaemonVersion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for StartRepositoryLinkRequestDeviceDaemonVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for StartRepositoryLinkRequestDeviceDaemonVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for StartRepositoryLinkRequestDeviceDaemonVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct StartRepositoryLinkRequestDeviceName(::std::string::String);
impl ::std::ops::Deref for StartRepositoryLinkRequestDeviceName {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<StartRepositoryLinkRequestDeviceName> for ::std::string::String {
    fn from(value: StartRepositoryLinkRequestDeviceName) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for StartRepositoryLinkRequestDeviceName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for StartRepositoryLinkRequestDeviceName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for StartRepositoryLinkRequestDeviceName {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for StartRepositoryLinkRequestDeviceName {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StartRepositoryLinkRequestDeviceOs {
    #[serde(rename = "macOS")]
    MacOs,
    Linux,
    Windows,
    #[serde(rename = "other")]
    Other,
}
impl ::std::fmt::Display for StartRepositoryLinkRequestDeviceOs {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::MacOs => f.write_str("macOS"),
            Self::Linux => f.write_str("Linux"),
            Self::Windows => f.write_str("Windows"),
            Self::Other => f.write_str("other"),
        }
    }
}
impl ::std::str::FromStr for StartRepositoryLinkRequestDeviceOs {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "macOS" => Ok(Self::MacOs),
            "Linux" => Ok(Self::Linux),
            "Windows" => Ok(Self::Windows),
            "other" => Ok(Self::Other),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for StartRepositoryLinkRequestDeviceOs {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for StartRepositoryLinkRequestDeviceOs {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct StartRepositoryLinkRequestDeviceOsVersion(::std::string::String);
impl ::std::ops::Deref for StartRepositoryLinkRequestDeviceOsVersion {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<StartRepositoryLinkRequestDeviceOsVersion> for ::std::string::String {
    fn from(value: StartRepositoryLinkRequestDeviceOsVersion) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for StartRepositoryLinkRequestDeviceOsVersion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for StartRepositoryLinkRequestDeviceOsVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for StartRepositoryLinkRequestDeviceOsVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for StartRepositoryLinkRequestDeviceOsVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct StartRepositoryLinkRequestDeviceProvidersItem {
    pub installed: bool,
    pub name: StartRepositoryLinkRequestDeviceProvidersItemName,
    #[serde(deserialize_with = "::std::option::Option::deserialize")]
    pub signed_in: ::std::option::Option<bool>,
    #[serde(deserialize_with = "::std::option::Option::deserialize")]
    pub version: ::std::option::Option<StartRepositoryLinkRequestDeviceProvidersItemVersion>,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StartRepositoryLinkRequestDeviceProvidersItemName {
    #[serde(rename = "claude-code")]
    ClaudeCode,
    #[serde(rename = "codex")]
    Codex,
}
impl ::std::fmt::Display for StartRepositoryLinkRequestDeviceProvidersItemName {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::ClaudeCode => f.write_str("claude-code"),
            Self::Codex => f.write_str("codex"),
        }
    }
}
impl ::std::str::FromStr for StartRepositoryLinkRequestDeviceProvidersItemName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "claude-code" => Ok(Self::ClaudeCode),
            "codex" => Ok(Self::Codex),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for StartRepositoryLinkRequestDeviceProvidersItemName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for StartRepositoryLinkRequestDeviceProvidersItemName {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct StartRepositoryLinkRequestDeviceProvidersItemVersion(::std::string::String);
impl ::std::ops::Deref for StartRepositoryLinkRequestDeviceProvidersItemVersion {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<StartRepositoryLinkRequestDeviceProvidersItemVersion> for ::std::string::String {
    fn from(value: StartRepositoryLinkRequestDeviceProvidersItemVersion) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for StartRepositoryLinkRequestDeviceProvidersItemVersion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for StartRepositoryLinkRequestDeviceProvidersItemVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for StartRepositoryLinkRequestDeviceProvidersItemVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for StartRepositoryLinkRequestDeviceProvidersItemVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct StartRepositoryLinkRequestName(::std::string::String);
impl ::std::ops::Deref for StartRepositoryLinkRequestName {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<StartRepositoryLinkRequestName> for ::std::string::String {
    fn from(value: StartRepositoryLinkRequestName) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for StartRepositoryLinkRequestName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for StartRepositoryLinkRequestName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for StartRepositoryLinkRequestName {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for StartRepositoryLinkRequestName {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct StartRepositoryLinkRequestRemotesItem {
    pub name: StartRepositoryLinkRequestRemotesItemName,
    pub url: StartRepositoryLinkRequestRemotesItemUrl,
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct StartRepositoryLinkRequestRemotesItemName(::std::string::String);
impl ::std::ops::Deref for StartRepositoryLinkRequestRemotesItemName {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<StartRepositoryLinkRequestRemotesItemName> for ::std::string::String {
    fn from(value: StartRepositoryLinkRequestRemotesItemName) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for StartRepositoryLinkRequestRemotesItemName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for StartRepositoryLinkRequestRemotesItemName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for StartRepositoryLinkRequestRemotesItemName {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for StartRepositoryLinkRequestRemotesItemName {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct StartRepositoryLinkRequestRemotesItemUrl(::std::string::String);
impl ::std::ops::Deref for StartRepositoryLinkRequestRemotesItemUrl {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<StartRepositoryLinkRequestRemotesItemUrl> for ::std::string::String {
    fn from(value: StartRepositoryLinkRequestRemotesItemUrl) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for StartRepositoryLinkRequestRemotesItemUrl {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 500usize {
            return Err("longer than 500 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for StartRepositoryLinkRequestRemotesItemUrl {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for StartRepositoryLinkRequestRemotesItemUrl {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for StartRepositoryLinkRequestRemotesItemUrl {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
#[serde(tag = "status")]
pub enum StartRepositoryLinkResponse {
    #[serde(rename = "linked")]
    Linked {
        chat_url: ::std::string::String,
        project: ::std::boxed::Box<StartRepositoryLinkResponseLinkedProject>,
        repository_id: ::uuid::Uuid,
        #[serde(skip_serializing_if = "::std::option::Option::is_none")]
        runner: ::std::option::Option<StartRepositoryLinkResponseLinkedRunner>,
        workspace: ::std::boxed::Box<StartRepositoryLinkResponseLinkedWorkspace>,
    },
    #[serde(rename = "choose")]
    Choose {
        choose_url: ::std::string::String,
        expires_at: ::chrono::DateTime<::chrono::offset::Utc>,
        link_id: ::uuid::Uuid,
    },
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct StartRepositoryLinkResponseLinkedProject {
    pub id: ::uuid::Uuid,
    pub name: ::std::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct StartRepositoryLinkResponseLinkedRunner {
    pub credential: ::std::string::String,
    pub runner_id: ::uuid::Uuid,
    pub workspace_id: ::uuid::Uuid,
    pub workspace_name: ::std::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct StartRepositoryLinkResponseLinkedWorkspace {
    pub id: ::uuid::Uuid,
    pub name: ::std::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct UpsertConnectionRequest {
    pub branch: UpsertConnectionRequestBranch,
    #[serde(default)]
    pub dirty: bool,
    pub head_sha: UpsertConnectionRequestHeadSha,
    pub local_key: UpsertConnectionRequestLocalKey,
    pub project_id: ::uuid::Uuid,
    pub repository_id: ::uuid::Uuid,
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct UpsertConnectionRequestBranch(::std::string::String);
impl ::std::ops::Deref for UpsertConnectionRequestBranch {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<UpsertConnectionRequestBranch> for ::std::string::String {
    fn from(value: UpsertConnectionRequestBranch) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for UpsertConnectionRequestBranch {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 200usize {
            return Err("longer than 200 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for UpsertConnectionRequestBranch {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for UpsertConnectionRequestBranch {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for UpsertConnectionRequestBranch {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct UpsertConnectionRequestHeadSha(::std::string::String);
impl ::std::ops::Deref for UpsertConnectionRequestHeadSha {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<UpsertConnectionRequestHeadSha> for ::std::string::String {
    fn from(value: UpsertConnectionRequestHeadSha) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for UpsertConnectionRequestHeadSha {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^[0-9a-f]{40}([0-9a-f]{24})?$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^[0-9a-f]{40}([0-9a-f]{24})?$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for UpsertConnectionRequestHeadSha {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for UpsertConnectionRequestHeadSha {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for UpsertConnectionRequestHeadSha {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct UpsertConnectionRequestLocalKey(::std::string::String);
impl ::std::ops::Deref for UpsertConnectionRequestLocalKey {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<UpsertConnectionRequestLocalKey> for ::std::string::String {
    fn from(value: UpsertConnectionRequestLocalKey) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for UpsertConnectionRequestLocalKey {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^[0-9a-f]{64}$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^[0-9a-f]{64}$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for UpsertConnectionRequestLocalKey {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for UpsertConnectionRequestLocalKey {
    type Error = self::error::ConversionError;
    fn try_from(value: ::std::string::String) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for UpsertConnectionRequestLocalKey {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct UpsertConnectionResponse {
    pub connection_id: ::uuid::Uuid,
}
pub mod defaults {
    pub(super) fn default_u64<T, const V: u64>() -> T
    where
        T: ::std::convert::TryFrom<u64>,
        <T as ::std::convert::TryFrom<u64>>::Error: ::std::fmt::Debug,
    {
        T::try_from(V).unwrap()
    }
    pub(super) fn register_repository_request_default_branch() -> super::RegisterRepositoryRequestDefaultBranch {
        super::RegisterRepositoryRequestDefaultBranch("main".to_string())
    }
    pub(super) fn start_repository_link_request_default_branch() -> super::StartRepositoryLinkRequestDefaultBranch {
        super::StartRepositoryLinkRequestDefaultBranch("main".to_string())
    }
}
pub mod error {
    pub struct ConversionError(::std::borrow::Cow<'static, str>);
    impl ::std::error::Error for ConversionError {}
    impl ::std::fmt::Display for ConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
            ::std::fmt::Display::fmt(&self.0, f)
        }
    }
    impl ::std::fmt::Debug for ConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
            ::std::fmt::Debug::fmt(&self.0, f)
        }
    }
    impl From<&'static str> for ConversionError {
        fn from(value: &'static str) -> Self {
            Self(value.into())
        }
    }
    impl From<String> for ConversionError {
        fn from(value: String) -> Self {
            Self(value.into())
        }
    }
}
