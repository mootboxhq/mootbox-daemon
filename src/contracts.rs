#![allow(clippy::unwrap_used)]
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct ClaimRequest {
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub active_run_ids: ::std::vec::Vec<::uuid::Uuid>,
    #[serde(default = "defaults::default_u64::<i64, 1>")]
    pub capacity: i64,
    pub providers: ::std::vec::Vec<ClaimRequestProvidersItem>,
    #[serde(default = "defaults::default_u64::<i64, 25000>")]
    pub wait_ms: i64,
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
