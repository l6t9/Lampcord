declare module "@requiem/ui" {
  export type TextVariant =
    | "text-xxs/normal"
    | "text-xxs/medium"
    | "text-xxs/semibold"
    | "text-xxs/bold"
    | "text-xs/normal"
    | "text-xs/medium"
    | "text-xs/semibold"
    | "text-xs/bold"
    | "text-sm/normal"
    | "text-sm/medium"
    | "text-sm/semibold"
    | "text-sm/bold"
    | "text-md/normal"
    | "text-md/medium"
    | "text-md/semibold"
    | "text-md/bold"
    | "text-lg/normal"
    | "text-lg/medium"
    | "text-lg/semibold"
    | "text-lg/bold"
    | "heading-sm/normal"
    | "heading-sm/medium"
    | "heading-sm/semibold"
    | "heading-sm/bold"
    | "heading-sm/extrabold"
    | "heading-md/normal"
    | "heading-md/medium"
    | "heading-md/semibold"
    | "heading-md/bold"
    | "heading-md/extrabold"
    | "heading-lg/normal"
    | "heading-lg/medium"
    | "heading-lg/semibold"
    | "heading-lg/bold"
    | "heading-lg/extrabold"
    | "heading-xl/normal"
    | "heading-xl/medium"
    | "heading-xl/semibold"
    | "heading-xl/bold"
    | "heading-xl/extrabold"
    | "heading-xxl/normal"
    | "heading-xxl/medium"
    | "heading-xxl/semibold"
    | "heading-xxl/bold"
    | "heading-xxl/extrabold"
    | "display-sm"
    | "display-md"
    | "display-lg"
    | "eyebrow"
    | "code";
  export type TextColor =
    | "text-strong"
    | "text-default"
    | "text-secondary"
    | "text-muted"
    | "text-subtle"
    | "text-critical"
    | "text-link"
    | "text-anchor"
    | "text-positive";
  export type ButtonVariant =
    | "primary"
    | "secondary"
    | "critical-primary"
    | "critical-secondary"
    | "active"
    | "overlay-primary"
    | "overlay-secondary"
    | "expressive"
    | "icon-only"
    | "bare";
  export type StackDirection =
    | "vertical"
    | "horizontal"
    | "vertical-reverse"
    | "horizontal-reverse";
  export type StackAlign =
    | "start"
    | "center"
    | "end"
    | "stretch"
    | "baseline";
  export type StackJustify =
    | "start"
    | "center"
    | "end"
    | "space-between"
    | "space-around";
  export type ButtonSize =
    | "small"
    | "medium";

  export type TextChild = string | number | boolean | null | undefined;

  export interface TextProps {
    variant: TextVariant;
    color?: TextColor;
    lineClamp?: number;
    center?: boolean;
    className?: string;
    children?: TextChild | TextChild[];
  }

  export interface ClickEvent {
    shiftKey: boolean;
  }

  export interface ContextMenuEvent {
    x: number;
    y: number;
  }

  export interface ButtonProps {
    variant?: ButtonVariant;
    size?: ButtonSize;
    className?: string;
    disabled?: boolean;
    fullWidth?: boolean;
    onClick?: (event: ClickEvent) => void;
    onContextMenu?: (event: ContextMenuEvent) => void;
    children?: TextChild | TextChild[] | JSX.Children;
  }

  export interface IconProps {
    width?: number;
    height?: number;
    color?: TextColor;
    className?: string;
  }

  export interface ChevronProps {
    open: boolean;
    color?: TextColor;
    width?: number;
    height?: number;
    className?: string;
  }

  export interface StackProps {
    direction?: StackDirection;
    align?: StackAlign;
    justify?: StackJustify;
    wrap?: boolean;
    gap?: number;
    padding?: number;
    fullWidth?: boolean;
    className?: string;
    children?: JSX.Children;
  }

  export interface ScrollEvent {
    offset: number;
    max: number;
  }

  export interface ScrollPositionValue {
    pinned: boolean;
    key: string | null;
    offset: number;
    height: number;
  }

  export interface MeasureEvent {
    keys: string[];
    heights: number[];
  }

  export interface ScrollViewProps {
    className?: string;
    itemHeight?: number | string;
    itemGap?: number;
    initialOffset?: number;
    onScroll?: (event: ScrollEvent) => void;
    onNearEnd?: () => void;
    position?: ScrollPositionValue;
    pinThreshold?: number;
    onPosition?: (position: ScrollPositionValue) => void;
    onMeasure?: (event: MeasureEvent) => void;
    children?: JSX.Children;
  }

  export function Text(props: TextProps): JSX.Element;
  export function Button(props: ButtonProps): JSX.Element;
  export function ScrollView(props: ScrollViewProps): JSX.Element;
  export function Stack(props: StackProps): JSX.Element;
  export function Chevron(props: ChevronProps): JSX.Element;

  export function AccessibilityIcon(props: IconProps): JSX.Element;
  export function ActivitiesIcon(props: IconProps): JSX.Element;
  export function AnalyticsIcon(props: IconProps): JSX.Element;
  export function AngleBracketsIcon(props: IconProps): JSX.Element;
  export function AnnouncementsIcon(props: IconProps): JSX.Element;
  export function AnnouncementsLockIcon(props: IconProps): JSX.Element;
  export function AnnouncementsWarningIcon(props: IconProps): JSX.Element;
  export function AppsIcon(props: IconProps): JSX.Element;
  export function ArrowsUpDownIcon(props: IconProps): JSX.Element;
  export function ArrowAngleLeftUpIcon(props: IconProps): JSX.Element;
  export function ArrowAngleRightUpIcon(props: IconProps): JSX.Element;
  export function ArrowLargeLeftIcon(props: IconProps): JSX.Element;
  export function ArrowLargeRightIcon(props: IconProps): JSX.Element;
  export function ArrowSmallLeftIcon(props: IconProps): JSX.Element;
  export function ArrowSmallRightIcon(props: IconProps): JSX.Element;
  export function ArrowUpRightIcon(props: IconProps): JSX.Element;
  export function AtIcon(props: IconProps): JSX.Element;
  export function AttachmentIcon(props: IconProps): JSX.Element;
  export function BellIcon(props: IconProps): JSX.Element;
  export function BicycleIcon(props: IconProps): JSX.Element;
  export function BookCheckIcon(props: IconProps): JSX.Element;
  export function BoostGemIcon(props: IconProps): JSX.Element;
  export function CalendarIcon(props: IconProps): JSX.Element;
  export function CalendarPlusIcon(props: IconProps): JSX.Element;
  export function ChannelForumIcon(props: IconProps): JSX.Element;
  export function ChannelForumLockIcon(props: IconProps): JSX.Element;
  export function ChannelTextLockIcon(props: IconProps): JSX.Element;
  export function ChannelVoiceLockIcon(props: IconProps): JSX.Element;
  export function ChatIcon(props: IconProps): JSX.Element;
  export function ChatCheckIcon(props: IconProps): JSX.Element;
  export function ChatMarkUnreadIcon(props: IconProps): JSX.Element;
  export function ChatShieldIcon(props: IconProps): JSX.Element;
  export function ChatSpeakIcon(props: IconProps): JSX.Element;
  export function CheckmarkLargeIcon(props: IconProps): JSX.Element;
  export function CheckmarkSmallIcon(props: IconProps): JSX.Element;
  export function CheckmarkSmallBoldIcon(props: IconProps): JSX.Element;
  export function ChevronLargeDownIcon(props: IconProps): JSX.Element;
  export function ChevronLargeRightIcon(props: IconProps): JSX.Element;
  export function ChevronSmallDownIcon(props: IconProps): JSX.Element;
  export function ChevronSmallLeftIcon(props: IconProps): JSX.Element;
  export function ChevronSmallRightIcon(props: IconProps): JSX.Element;
  export function ChevronSmallUpIcon(props: IconProps): JSX.Element;
  export function CircleCheckIcon(props: IconProps): JSX.Element;
  export function CircleErrorIcon(props: IconProps): JSX.Element;
  export function CircleInformationIcon(props: IconProps): JSX.Element;
  export function CirclePlusIcon(props: IconProps): JSX.Element;
  export function CircleQuestionIcon(props: IconProps): JSX.Element;
  export function CircleXIcon(props: IconProps): JSX.Element;
  export function ClockIcon(props: IconProps): JSX.Element;
  export function ClydeIcon(props: IconProps): JSX.Element;
  export function CompassIcon(props: IconProps): JSX.Element;
  export function CopyIcon(props: IconProps): JSX.Element;
  export function CreditCardIcon(props: IconProps): JSX.Element;
  export function CrownIcon(props: IconProps): JSX.Element;
  export function DenyIcon(props: IconProps): JSX.Element;
  export function DiscordStatusIcon(props: IconProps): JSX.Element;
  export function DoorExitIcon(props: IconProps): JSX.Element;
  export function DoubleCheckmarkIcon(props: IconProps): JSX.Element;
  export function DoubleChevronSmallRightIcon(props: IconProps): JSX.Element;
  export function DownloadIcon(props: IconProps): JSX.Element;
  export function DragIcon(props: IconProps): JSX.Element;
  export function EmbedIcon(props: IconProps): JSX.Element;
  export function EmojiSlightlySmilingFaceIcon(props: IconProps): JSX.Element;
  export function EyeIcon(props: IconProps): JSX.Element;
  export function EyeDropperIcon(props: IconProps): JSX.Element;
  export function EyeSlashIcon(props: IconProps): JSX.Element;
  export function FileIcon(props: IconProps): JSX.Element;
  export function FileUpIcon(props: IconProps): JSX.Element;
  export function FiltersHorizontalIcon(props: IconProps): JSX.Element;
  export function FireIcon(props: IconProps): JSX.Element;
  export function FlagIcon(props: IconProps): JSX.Element;
  export function FolderIcon(props: IconProps): JSX.Element;
  export function FolderPlusIcon(props: IconProps): JSX.Element;
  export function FoodIcon(props: IconProps): JSX.Element;
  export function ForumIcon(props: IconProps): JSX.Element;
  export function ForumLockIcon(props: IconProps): JSX.Element;
  export function ForumWarningIcon(props: IconProps): JSX.Element;
  export function FriendsIcon(props: IconProps): JSX.Element;
  export function FullscreenEnterIcon(props: IconProps): JSX.Element;
  export function FullscreenExitIcon(props: IconProps): JSX.Element;
  export function GameControllerIcon(props: IconProps): JSX.Element;
  export function GiftIcon(props: IconProps): JSX.Element;
  export function GroupIcon(props: IconProps): JSX.Element;
  export function GroupPlusIcon(props: IconProps): JSX.Element;
  export function HeadphonesIcon(props: IconProps): JSX.Element;
  export function HeadphonesSlashIcon(props: IconProps): JSX.Element;
  export function HeartIcon(props: IconProps): JSX.Element;
  export function HeartOutlineIcon(props: IconProps): JSX.Element;
  export function HubIcon(props: IconProps): JSX.Element;
  export function IdIcon(props: IconProps): JSX.Element;
  export function ImageIcon(props: IconProps): JSX.Element;
  export function ImageLockIcon(props: IconProps): JSX.Element;
  export function ImagePlusIcon(props: IconProps): JSX.Element;
  export function ImageWarningIcon(props: IconProps): JSX.Element;
  export function InboxIcon(props: IconProps): JSX.Element;
  export function JumpToMessageIcon(props: IconProps): JSX.Element;
  export function KeybindArrowIcon(props: IconProps): JSX.Element;
  export function KeyboardIcon(props: IconProps): JSX.Element;
  export function LanguageIcon(props: IconProps): JSX.Element;
  export function LinkIcon(props: IconProps): JSX.Element;
  export function LockIcon(props: IconProps): JSX.Element;
  export function MagnifyingGlassIcon(props: IconProps): JSX.Element;
  export function MagnifyingGlassMinusIcon(props: IconProps): JSX.Element;
  export function MagnifyingGlassPlusIcon(props: IconProps): JSX.Element;
  export function MenuIcon(props: IconProps): JSX.Element;
  export function MicrophoneIcon(props: IconProps): JSX.Element;
  export function MicrophoneSlashIcon(props: IconProps): JSX.Element;
  export function MoreHorizontalIcon(props: IconProps): JSX.Element;
  export function NatureIcon(props: IconProps): JSX.Element;
  export function NitroWheelIcon(props: IconProps): JSX.Element;
  export function ObjectIcon(props: IconProps): JSX.Element;
  export function PaintPaletteIcon(props: IconProps): JSX.Element;
  export function PaperPlusIcon(props: IconProps): JSX.Element;
  export function PauseIcon(props: IconProps): JSX.Element;
  export function PencilIcon(props: IconProps): JSX.Element;
  export function PersonShieldIcon(props: IconProps): JSX.Element;
  export function PhoneCallIcon(props: IconProps): JSX.Element;
  export function PhoneHangUpIcon(props: IconProps): JSX.Element;
  export function PinIcon(props: IconProps): JSX.Element;
  export function PlayIcon(props: IconProps): JSX.Element;
  export function PlusLargeIcon(props: IconProps): JSX.Element;
  export function PlusMediumIcon(props: IconProps): JSX.Element;
  export function PlusSmallIcon(props: IconProps): JSX.Element;
  export function PollsIcon(props: IconProps): JSX.Element;
  export function PuzzlePieceIcon(props: IconProps): JSX.Element;
  export function QuestsIcon(props: IconProps): JSX.Element;
  export function RefreshIcon(props: IconProps): JSX.Element;
  export function ReplayIcon(props: IconProps): JSX.Element;
  export function RtcPingIcon(props: IconProps): JSX.Element;
  export function ScreenIcon(props: IconProps): JSX.Element;
  export function ScreenArrowIcon(props: IconProps): JSX.Element;
  export function ScreenStreamIcon(props: IconProps): JSX.Element;
  export function SendMessageIcon(props: IconProps): JSX.Element;
  export function ServerDeafenIcon(props: IconProps): JSX.Element;
  export function ServerMuteIcon(props: IconProps): JSX.Element;
  export function SettingsIcon(props: IconProps): JSX.Element;
  export function ShieldIcon(props: IconProps): JSX.Element;
  export function ShieldLockIcon(props: IconProps): JSX.Element;
  export function ShieldUserIcon(props: IconProps): JSX.Element;
  export function ShopIcon(props: IconProps): JSX.Element;
  export function SlashBoxIcon(props: IconProps): JSX.Element;
  export function SoundboardIcon(props: IconProps): JSX.Element;
  export function SparklesIcon(props: IconProps): JSX.Element;
  export function StaffBadgeIcon(props: IconProps): JSX.Element;
  export function StageIcon(props: IconProps): JSX.Element;
  export function StageLockIcon(props: IconProps): JSX.Element;
  export function StarIcon(props: IconProps): JSX.Element;
  export function StarOutlineIcon(props: IconProps): JSX.Element;
  export function StickerIcon(props: IconProps): JSX.Element;
  export function SubscriptionIcon(props: IconProps): JSX.Element;
  export function TextIcon(props: IconProps): JSX.Element;
  export function TextLockIcon(props: IconProps): JSX.Element;
  export function TextWarningIcon(props: IconProps): JSX.Element;
  export function ThreadIcon(props: IconProps): JSX.Element;
  export function ThreadLockIcon(props: IconProps): JSX.Element;
  export function ThreadPlusIcon(props: IconProps): JSX.Element;
  export function ThreadSpineIcon(props: IconProps): JSX.Element;
  export function ThreadWarningIcon(props: IconProps): JSX.Element;
  export function TrashIcon(props: IconProps): JSX.Element;
  export function TrophyIcon(props: IconProps): JSX.Element;
  export function UserIcon(props: IconProps): JSX.Element;
  export function UserCheckIcon(props: IconProps): JSX.Element;
  export function UserCircleIcon(props: IconProps): JSX.Element;
  export function UserCircleStatusIcon(props: IconProps): JSX.Element;
  export function UserPlatformIcon(props: IconProps): JSX.Element;
  export function UserPlusIcon(props: IconProps): JSX.Element;
  export function VideoIcon(props: IconProps): JSX.Element;
  export function VideoSlashIcon(props: IconProps): JSX.Element;
  export function VoiceLockIcon(props: IconProps): JSX.Element;
  export function VoiceLowIcon(props: IconProps): JSX.Element;
  export function VoiceNormalIcon(props: IconProps): JSX.Element;
  export function VoiceWarningIcon(props: IconProps): JSX.Element;
  export function VoiceXIcon(props: IconProps): JSX.Element;
  export function WarningIcon(props: IconProps): JSX.Element;
  export function WaveformIcon(props: IconProps): JSX.Element;
  export function WaveformSlashIcon(props: IconProps): JSX.Element;
  export function WindowLaunchIcon(props: IconProps): JSX.Element;
  export function WinCloseIcon(props: IconProps): JSX.Element;
  export function WinMaximizeIcon(props: IconProps): JSX.Element;
  export function WinMinimizeIcon(props: IconProps): JSX.Element;
  export function XLargeIcon(props: IconProps): JSX.Element;
  export function XNeutralIcon(props: IconProps): JSX.Element;
  export function XSmallIcon(props: IconProps): JSX.Element;
}
