use std::collections::BTreeMap;

use serde_json::Value;

pub const KNOWN_COMPONENTS: &[&str] = &[
    "div",
    "span",
    "mana:text",
    "mana:heading",
    "mana:button",
    "mana:switch",
    "mana:text-input",
    "mana:text-field",
    "mana:form-item",
    "mana:stack",
    "mana:image",
    "mana:avatar",
    "mana:icon",
    "mana:profile",
    "mana:tab",
    "mana:section",
];

pub const KNOWN_ITEM_SLOTS: &[&str] = &[
    "composer.buttons",
    "composer.attach-menu",
    "chat-header.toolbar",
    "user-panel.buttons",
    "message.toolbar",
    "guild-header.menu",
    "dm-list.header",
    "guild-rail.top",
    "guild-rail.bottom",
    "profile.actions",
    "voice-panel.buttons",
    "title-bar.buttons",
    "settings.sidebar",
];

pub const KNOWN_CONTENT_SLOTS: &[&str] = &[
    "composer.banner",
    "chat.above-messages",
    "profile.section",
    "profile.collection",
    "member-list.top",
    "channel-list.top",
    "voice-panel.content",
];

pub const MAX_NODES: usize = 2000;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ViewNode {
    pub parent: Option<u32>,
    pub tag: String,
    pub key: String,
    pub props: BTreeMap<String, Value>,
    pub text: Option<String>,
    pub classes: Vec<String>,
    pub style: Option<String>,

    pub events: Vec<(String, String)>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ViewTree {
    pub nodes: Vec<ViewNode>,
}

impl ViewTree {
    pub fn roots(&self) -> impl Iterator<Item = u32> + '_ {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.parent.is_none())
            .map(|(i, _)| i as u32)
    }

    pub fn children(&self, id: u32) -> impl Iterator<Item = u32> + '_ {
        self.nodes
            .iter()
            .enumerate()
            .filter(move |(_, n)| n.parent == Some(id))
            .map(|(i, _)| i as u32)
    }

    pub fn node(&self, id: u32) -> &ViewNode {
        &self.nodes[id as usize]
    }

    pub fn action(&self, id: u32, event: &str) -> Option<&str> {
        self.node(id)
            .events
            .iter()
            .find(|(e, _)| e == event)
            .map(|(_, a)| a.as_str())
    }
}

#[derive(Default)]
pub(crate) struct ViewBuilderState {
    pub tree: ViewTree,
    pub overflowed: bool,
}

impl ViewBuilderState {
    pub fn element(&mut self, parent: Option<u32>, tag: String, key: String) -> u32 {
        let parent = parent.filter(|p| (*p as usize) < self.tree.nodes.len());
        if self.tree.nodes.len() >= MAX_NODES {
            self.overflowed = true;
            return u32::MAX;
        }
        self.tree.nodes.push(ViewNode {
            parent,
            tag,
            key,
            ..Default::default()
        });
        (self.tree.nodes.len() - 1) as u32
    }

    pub fn node_mut(&mut self, id: u32) -> Option<&mut ViewNode> {
        self.tree.nodes.get_mut(id as usize)
    }
}
