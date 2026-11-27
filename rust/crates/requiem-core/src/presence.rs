use crate::gateway::opcodes;
use crate::state::wire;
use crate::utils::time;
use requiem_ui::widget::primitives::PresenceStatus;
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OwnStatus {
    #[default]
    Online,
    Idle,
    Dnd,
    Invisible,
}

impl OwnStatus {
    pub const ALL: [OwnStatus; 4] = [
        OwnStatus::Online,
        OwnStatus::Idle,
        OwnStatus::Dnd,
        OwnStatus::Invisible,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            OwnStatus::Online => "online",
            OwnStatus::Idle => "idle",
            OwnStatus::Dnd => "dnd",
            OwnStatus::Invisible => "invisible",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.as_str() == value)
    }

    pub fn label(self) -> String {
        match self {
            OwnStatus::Online => requiem_i18n::t!("presence-online"),
            OwnStatus::Idle => requiem_i18n::t!("presence-idle"),
            OwnStatus::Dnd => requiem_i18n::t!("presence-do-not-disturb"),
            OwnStatus::Invisible => requiem_i18n::t!("presence-invisible"),
        }
    }

    pub fn description(self) -> Option<String> {
        match self {
            OwnStatus::Dnd => Some(requiem_i18n::t!(
                "presence-you-will-not-receive-desktop-notificatio"
            )),
            OwnStatus::Invisible => Some(requiem_i18n::t!("presence-you-will-appear-offline")),
            _ => None,
        }
    }

    pub fn indicator(self) -> PresenceStatus {
        match self {
            OwnStatus::Online => PresenceStatus::Online,
            OwnStatus::Idle => PresenceStatus::Idle,
            OwnStatus::Dnd => PresenceStatus::Dnd,
            OwnStatus::Invisible => PresenceStatus::Offline,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusEmoji {
    pub id: Option<u64>,
    pub name: String,
    pub animated: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CustomStatus {
    pub text: String,
    pub emoji: Option<StatusEmoji>,
    pub expires_at_ms: Option<u64>,
}

impl CustomStatus {
    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty() && self.emoji.is_none()
    }

    pub fn is_expired(&self, now_ms: u64) -> bool {
        self.expires_at_ms.is_some_and(|at| at <= now_ms)
    }

    pub fn from_activities(activities: &[wire::Activity]) -> Option<Self> {
        let activity = activities.iter().find(|a| a.kind == 4)?;
        let text = activity.state.clone().unwrap_or_default();
        let emoji = activity.emoji.as_ref().and_then(|e| {
            Some(StatusEmoji {
                id: e.id.map(|id| id.get()),
                name: e.name.clone()?,
                animated: e.animated,
            })
        });
        let status = CustomStatus {
            text,
            emoji,
            expires_at_ms: None,
        };
        (!status.is_empty()).then_some(status)
    }

    pub fn to_activity(&self) -> Value {
        let emoji = self.emoji.as_ref().map(|e| json!({ "id": e.id.map(|id| id.to_string()), "name": e.name, "animated": e.animated }));
        json!({
            "name": requiem_i18n::t!("presence-custom-status"),
            "type": 4,
            "state": if self.text.trim().is_empty() { Value::Null } else { json!(self.text) },
            "emoji": emoji,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresenceField {
    Status,
    CustomStatus,
}

#[derive(Clone, Debug)]
pub struct PresenceUpdate {
    pub status: OwnStatus,
    pub custom: Option<CustomStatus>,
    pub changed: PresenceField,
}

pub fn update_payload(status: OwnStatus, custom: Option<&CustomStatus>) -> Value {
    let activities: Vec<Value> = custom
        .filter(|c| !c.is_empty() && !c.is_expired(time::now_ms()))
        .map(CustomStatus::to_activity)
        .into_iter()
        .collect();
    json!({
        "op": opcodes::PRESENCE_UPDATE,
        "d": { "status": status.as_str(), "since": 0, "activities": activities, "afk": false },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_status_round_trips_through_activity() {
        let status = CustomStatus {
            text: "Working".into(),
            emoji: Some(StatusEmoji {
                id: None,
                name: "\u{1f349}".into(),
                animated: false,
            }),
            expires_at_ms: None,
        };
        let activities = |value: Value| {
            serde_json::from_str::<Vec<wire::Activity>>(&value.to_string()).unwrap()
        };
        let parsed = CustomStatus::from_activities(&activities(
            json!([{ "type": 0, "name": "Game" }, status.to_activity()]),
        ))
        .unwrap();
        assert_eq!(parsed, status);
        assert!(
            CustomStatus::from_activities(&activities(
                json!([{ "type": 4, "state": null, "emoji": null }])
            ))
            .is_none()
        );
    }

    #[test]
    fn payload_drops_expired_status() {
        let expired = CustomStatus {
            text: "x".into(),
            emoji: None,
            expires_at_ms: Some(1),
        };
        assert_eq!(
            update_payload(OwnStatus::Online, Some(&expired))["d"]["activities"],
            json!([])
        );
        let live = CustomStatus {
            text: "x".into(),
            ..Default::default()
        };
        assert_eq!(
            update_payload(OwnStatus::Idle, Some(&live))["d"]["activities"][0]["state"],
            json!("x")
        );
    }
}
