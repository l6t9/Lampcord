declare module "@requiem/app" {
  export interface SearchBarProps {
    placeholder: string;
    text?: string;
    autofocus?: boolean;
    clearable?: boolean;
    onChange: (query: string) => void;
    onSubmit?: (query: string) => void;
    className?: string;
  }
  export interface AssetImageProps {
    name: string;
    width: number;
    height: number;
    className?: string;
  }
  export interface AvatarProps {
    userId: Types.UserId;
    className?: string;
  }
  export interface AuthorNameProps {
    userId: Types.UserId;
    guildId?: Types.GuildId | null;
    className?: string;
  }
  export interface MessageRowHostProps {
    channelId: Types.ChannelId;
    messageId: string;
    startsGroup: boolean;
  }
  export interface MessageDayDividerProps {
    day?: string | null;
    unread: boolean;
    startsGroup: boolean;
  }
  export interface MessagePendingNoticesProps {
    channelId: Types.ChannelId;
    messageId: string;
  }
  export interface MessagePendingBottomProps {
    channelId: Types.ChannelId;
  }
  export interface MessageHistoryHeaderProps {
    channelId: Types.ChannelId;
    loading: boolean;
  }

  export function SearchBar(props: SearchBarProps): JSX.Element;
  export function AssetImage(props: AssetImageProps): JSX.Element;
  export function Avatar(props: AvatarProps): JSX.Element;
  export function AuthorName(props: AuthorNameProps): JSX.Element;
  export function MessageRowHost(props: MessageRowHostProps): JSX.Element;
  export function MessageDayDivider(props: MessageDayDividerProps): JSX.Element;
  export function MessagePendingNotices(props: MessagePendingNoticesProps): JSX.Element;
  export function MessagePendingBottom(props: MessagePendingBottomProps): JSX.Element;
  export function MessageHistoryHeader(props: MessageHistoryHeaderProps): JSX.Element;
}
