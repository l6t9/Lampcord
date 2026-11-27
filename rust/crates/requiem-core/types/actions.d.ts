declare const actions: {
  goToThread(threadId: Types.ChannelId, shiftKey: boolean): void;
  openChannelMenu(channelId: Types.ChannelId, guildId: Types.GuildId | null, x: number, y: number): void;
  loadMoreThreads(channelId: Types.ChannelId): void;
  rememberThreadScroll(offset: number): void;
  startThread(channelId: Types.ChannelId): void;
  threadSearchChanged(query: string): void;
  submitThreadSearch(channelId: Types.ChannelId, query: string): void;
  closeThreadBrowser(): void;
  messageListScrolled(channelId: Types.ChannelId, pinned: boolean, key: string | null, offset: number, height: number): void;
  messageRowsMeasured(keys: (string)[], heights: (number)[]): void;
};
