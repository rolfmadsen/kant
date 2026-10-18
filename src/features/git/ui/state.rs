use crate::features::git::{DomainChangeEvent, GitCommitInfo, RepoSyncStatus};
use crate::features::model::merge::ModelConflict;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum PublishAsyncResult {
    Success {
        sync_status: RepoSyncStatus,
    },
    Conflict {
        conflicts: Vec<ModelConflict>,
        sync_status: RepoSyncStatus,
    },
    Error(String),
}

#[derive(Debug, Clone)]
pub struct PublishModalState {
    pub message: String,
    pub preview_events: Vec<DomainChangeEvent>,
    pub is_publishing: bool,
    pub error: Option<String>,
}

impl PublishModalState {
    pub fn new(message: String, preview_events: Vec<DomainChangeEvent>) -> Self {
        Self {
            message,
            preview_events,
            is_publishing: false,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictChoice {
    Ours,
    Theirs,
}

#[derive(Debug, Clone)]
pub struct ModelHistoryModalState {
    pub target_element_id: Option<Uuid>,
    pub commits: Vec<GitCommitInfo>,
}

impl ModelHistoryModalState {
    pub fn new(target_element_id: Option<Uuid>, commits: Vec<GitCommitInfo>) -> Self {
        Self {
            target_element_id,
            commits,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConflictResolverModalState {
    pub conflicts: Vec<ModelConflict>,
    pub resolved_choices: HashMap<usize, ConflictChoice>,
}

impl ConflictResolverModalState {
    pub fn new(conflicts: Vec<ModelConflict>) -> Self {
        let mut resolved_choices = HashMap::new();
        for (i, _) in conflicts.iter().enumerate() {
            resolved_choices.insert(i, ConflictChoice::Ours);
        }
        Self {
            conflicts,
            resolved_choices,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitConnectionModalState {
    pub remote_url: String,
    pub target_dir: String,
    pub custom_target_dir: bool,
    pub token: String,
    pub author_name: String,
    pub author_email: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitCloneModalState {
    pub remote_url: String,
    pub target_dir: String,
    pub custom_target_dir: bool,
    pub token: String,
    pub is_cloning: bool,
    pub error_message: Option<String>,
}
