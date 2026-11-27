use serde::Deserialize;

use super::id::{
    Id,
    marker::{ApplicationMarker, CommandMarker},
};

pub mod option_type {
    pub const SUB_COMMAND: u8 = 1;
    pub const SUB_COMMAND_GROUP: u8 = 2;
    pub const STRING: u8 = 3;
    pub const INTEGER: u8 = 4;
    pub const BOOLEAN: u8 = 5;
    pub const USER: u8 = 6;
    pub const CHANNEL: u8 = 7;
    pub const ROLE: u8 = 8;
    pub const MENTIONABLE: u8 = 9;
    pub const NUMBER: u8 = 10;
    pub const ATTACHMENT: u8 = 11;
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct CommandChoice {
    pub name: String,
    pub value: serde_json::Value,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct CommandOption {
    #[serde(rename = "type")]
    pub kind: u8,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub choices: Vec<CommandChoice>,

    #[serde(default)]
    pub options: Vec<CommandOption>,
    #[serde(default)]
    pub autocomplete: bool,
}

fn chat_input() -> u8 {
    1
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ApplicationCommand {
    pub id: Id<CommandMarker>,
    pub application_id: Id<ApplicationMarker>,
    pub name: String,
    #[serde(default)]
    pub description: String,

    #[serde(rename = "type", default = "chat_input")]
    pub kind: u8,
    #[serde(default)]
    pub options: Vec<CommandOption>,
    #[serde(default)]
    pub version: Option<Id<CommandMarker>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CommandApp {
    pub id: Id<ApplicationMarker>,
    pub name: String,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct CommandIndex {
    #[serde(default)]
    pub application_commands: Vec<ApplicationCommand>,
    #[serde(default)]
    pub applications: Vec<CommandApp>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommandEntry {
    pub command: ApplicationCommand,

    pub path: Vec<(u8, String)>,

    pub options: Vec<CommandOption>,
    pub description: String,
}

impl CommandEntry {
    pub fn display(&self) -> String {
        let mut text = self.command.name.clone();
        for (_, name) in &self.path {
            text.push(' ');
            text.push_str(name);
        }
        text
    }
}

impl CommandIndex {
    pub fn app(&self, application_id: Id<ApplicationMarker>) -> Option<&CommandApp> {
        self.applications
            .iter()
            .find(|app| app.id == application_id)
    }

    pub fn entries(&self) -> Vec<CommandEntry> {
        let mut entries = Vec::new();
        for command in self.application_commands.iter().filter(|c| c.kind == 1) {
            let has_subcommands = command.options.iter().any(|o| {
                matches!(
                    o.kind,
                    option_type::SUB_COMMAND | option_type::SUB_COMMAND_GROUP
                )
            });
            if !has_subcommands {
                entries.push(CommandEntry {
                    command: command.clone(),
                    path: Vec::new(),
                    options: command.options.clone(),
                    description: command.description.clone(),
                });
                continue;
            }
            for option in &command.options {
                match option.kind {
                    option_type::SUB_COMMAND => entries.push(CommandEntry {
                        command: command.clone(),
                        path: vec![(option.kind, option.name.clone())],
                        options: option.options.clone(),
                        description: option.description.clone(),
                    }),
                    option_type::SUB_COMMAND_GROUP => {
                        for sub in option
                            .options
                            .iter()
                            .filter(|o| o.kind == option_type::SUB_COMMAND)
                        {
                            entries.push(CommandEntry {
                                command: command.clone(),
                                path: vec![
                                    (option.kind, option.name.clone()),
                                    (sub.kind, sub.name.clone()),
                                ],
                                options: sub.options.clone(),
                                description: sub.description.clone(),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
        entries
    }
}
