use crate::features::collab::{CollabKey, RoomId, SessionTicket};
use serde::{Deserialize, Serialize};

pub const START_SESSION_BTN_PRESET: usize = 3;
pub const START_SESSION_BTN_REMEMBER: usize = 11;
pub const START_SESSION_BTN_COPY: usize = 0;
pub const START_SESSION_BTN_CANCEL: usize = 1;
pub const START_SESSION_BTN_START: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RelayServerPreset {
    #[default]
    Koyeb,
    InternalOrg,
    LocalDocker,
    Custom,
}

impl RelayServerPreset {
    pub const ALL: [RelayServerPreset; 4] = [
        RelayServerPreset::Koyeb,
        RelayServerPreset::InternalOrg,
        RelayServerPreset::LocalDocker,
        RelayServerPreset::Custom,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Koyeb => "Koyeb Cloud (Standard - Frankfurt)",
            Self::InternalOrg => "Intern Organisation",
            Self::LocalDocker => "Lokal Docker (ws://localhost:8080/ws)",
            Self::Custom => "Brugerdefineret URL...",
        }
    }

    pub fn default_url(&self) -> &'static str {
        match self {
            Self::Koyeb => "wss://kant.koyeb.app/ws",
            Self::InternalOrg => "wss://collab.intern.org/ws",
            Self::LocalDocker => "ws://localhost:8080/ws",
            Self::Custom => "",
        }
    }
}

impl std::fmt::Display for RelayServerPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone)]
pub struct StartSessionModalState {
    pub preset: RelayServerPreset,
    pub custom_url: String,
    pub remember_choice: bool,
    pub ticket: SessionTicket,
    pub copied: bool,
}

impl StartSessionModalState {
    pub fn new() -> Self {
        let preset = RelayServerPreset::Koyeb;
        let room = RoomId::generate();
        let key = CollabKey::generate();
        let ticket = SessionTicket::new(preset.default_url(), room, key);
        Self {
            preset,
            custom_url: String::new(),
            remember_choice: false,
            ticket,
            copied: false,
        }
    }

    pub fn current_url(&self) -> &str {
        if (self.preset == RelayServerPreset::Custom
            || self.preset == RelayServerPreset::InternalOrg)
            && !self.custom_url.is_empty()
        {
            &self.custom_url
        } else {
            self.preset.default_url()
        }
    }

    pub fn set_preset(&mut self, preset: RelayServerPreset) {
        self.preset = preset;
        let room = self.ticket.room_id.clone();
        let key = self.ticket.key.clone();
        let target_url = if (preset == RelayServerPreset::Custom
            || preset == RelayServerPreset::InternalOrg)
            && !self.custom_url.is_empty()
        {
            self.custom_url.as_str()
        } else {
            preset.default_url()
        };
        self.ticket = SessionTicket::new(target_url, room, key);
    }

    pub fn set_custom_url(&mut self, url: String) {
        self.custom_url = url.clone();
        if self.preset == RelayServerPreset::Custom || self.preset == RelayServerPreset::InternalOrg
        {
            let room = self.ticket.room_id.clone();
            let key = self.ticket.key.clone();
            self.ticket = SessionTicket::new(url, room, key);
        }
    }
}

impl Default for StartSessionModalState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct JoinSessionModalState {
    pub token_input: String,
    pub parsed_ticket: Option<SessionTicket>,
    pub error_message: Option<String>,
}

impl JoinSessionModalState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_token_input(&mut self, token: String) {
        let trimmed = token.trim().to_string();
        self.token_input = token;
        if trimmed.is_empty() {
            self.parsed_ticket = None;
            self.error_message = None;
        } else {
            match SessionTicket::from_token(&trimmed) {
                Ok(ticket) => {
                    self.parsed_ticket = Some(ticket);
                    self.error_message = None;
                }
                Err(err) => {
                    self.parsed_ticket = None;
                    self.error_message = Some(format!("Ugyldig sessionskode: {err}"));
                }
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        self.parsed_ticket.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct GuestEndedNoticeModalState {
    pub message: String,
}
