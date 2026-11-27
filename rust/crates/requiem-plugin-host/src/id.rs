use std::fmt;
use std::marker::PhantomData;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Id<T> {
    raw: u64,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Id<T> {
    pub const fn new(raw: u64) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    pub const fn get(self) -> u64 {
        self.raw
    }
}

impl<T: Copy> Copy for Id<T> {}

impl<T> Default for Id<T> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Id({})", self.raw)
    }
}

impl<T> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl<T> From<u64> for Id<T> {
    fn from(raw: u64) -> Self {
        Self::new(raw)
    }
}

impl<T> From<Id<T>> for u64 {
    fn from(id: Id<T>) -> Self {
        id.raw
    }
}

pub mod marker {
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct UserMarker;
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct GuildMarker;
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct ChannelMarker;
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct MessageMarker;
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct RoleMarker;
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct AttachmentMarker;
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct WebhookMarker;
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct ApplicationMarker;
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct EmojiMarker;
}

pub type UserId = Id<marker::UserMarker>;
pub type GuildId = Id<marker::GuildMarker>;
pub type ChannelId = Id<marker::ChannelMarker>;
pub type MessageId = Id<marker::MessageMarker>;
pub type RoleId = Id<marker::RoleMarker>;
pub type AttachmentId = Id<marker::AttachmentMarker>;
pub type WebhookId = Id<marker::WebhookMarker>;
pub type ApplicationId = Id<marker::ApplicationMarker>;
pub type EmojiId = Id<marker::EmojiMarker>;
