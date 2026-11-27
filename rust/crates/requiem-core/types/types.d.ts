declare namespace Types {
  export type ApplicationId = string & { readonly __brand: "ApplicationId" };
  export type AttachmentId = string & { readonly __brand: "AttachmentId" };
  export type ChannelId = string & { readonly __brand: "ChannelId" };
  export type CommandId = string & { readonly __brand: "CommandId" };
  export type EmojiId = string & { readonly __brand: "EmojiId" };
  export type GuildId = string & { readonly __brand: "GuildId" };
  export type MessageId = string & { readonly __brand: "MessageId" };
  export type RoleId = string & { readonly __brand: "RoleId" };
  export type UserId = string & { readonly __brand: "UserId" };
  export type WebhookId = string & { readonly __brand: "WebhookId" };
  export type BrowserAccess = {
    canRead: boolean,
    canCreate: boolean,
  };
  export type BrowserEntry = {
    key: string,
    kind: EntryKind,
    section: Section,
    count: number,
    height: number,
    thread: ChannelView | null,
  };
  export type BrowserView = {
    phase: Phase,
    canCreate: boolean,
    entries: BrowserEntry[],
  };
  export type ChannelView = {
    id: ChannelId,
    guildId: GuildId | null,
    parentId: ChannelId | null,
    name: string | null,
    kind: number,
    isThread: boolean,
    archived: boolean,
    ownerId: UserId | null,
    lastMessageId: MessageId | null,
  };
  export type EntryKind = "pad" | "header" | "thread";
  export type MessageView = {
    id: MessageId,
    channelId: ChannelId,
    authorId: UserId,
    content: string,
    contentPreview: string,
    timestamp: string,
    edited: boolean,
  };
  export type Phase = "loading" | "empty" | "list";
  export type RowKind = "history" | "message" | "tail";
  export type RowView = {
    key: string,
    kind: RowKind,
    height: number,
    day: string | null,
    unread: boolean,
    startsGroup: boolean,
    loading: boolean,
  };
  export type ScrollPositionView = {
    pinned: boolean,
    key: string | null,
    offset: number,
    height: number,
  };
  export type Section = "joined" | "other" | "older" | "results";
  export type UserView = {
    id: UserId,
    username: string,
    displayName: string,
    avatar: string | null,
    bot: boolean,
  };
}

declare module "@requiem/types" {
  export type ApplicationId = Types.ApplicationId;
  export type AttachmentId = Types.AttachmentId;
  export type ChannelId = Types.ChannelId;
  export type CommandId = Types.CommandId;
  export type EmojiId = Types.EmojiId;
  export type GuildId = Types.GuildId;
  export type MessageId = Types.MessageId;
  export type RoleId = Types.RoleId;
  export type UserId = Types.UserId;
  export type WebhookId = Types.WebhookId;
  export type BrowserAccess = Types.BrowserAccess;
  export type BrowserEntry = Types.BrowserEntry;
  export type BrowserView = Types.BrowserView;
  export type ChannelView = Types.ChannelView;
  export type EntryKind = Types.EntryKind;
  export type MessageView = Types.MessageView;
  export type Phase = Types.Phase;
  export type RowKind = Types.RowKind;
  export type RowView = Types.RowView;
  export type ScrollPositionView = Types.ScrollPositionView;
  export type Section = Types.Section;
  export type UserView = Types.UserView;
}
