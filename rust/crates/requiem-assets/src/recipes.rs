use crate::resolve::Recipe;


fn icon(name: &str, export: &str) -> Recipe {
    Recipe::SvgComponent {
        name: name.into(),
        export: export.into(),
    }
}
fn by_path(name: &str, d_prefix: &str) -> Recipe {
    Recipe::SvgByPath {
        name: name.into(),
        d_prefix: d_prefix.into(),
    }
}
fn by_markup(name: &str, needles: &[&str]) -> Recipe {
    Recipe::SvgByMarkup {
        name: name.into(),
        needles: needles.iter().map(|n| n.to_string()).collect(),
    }
}
fn font(name: &str, family: &str, weight: u16, italic: bool) -> Recipe {
    Recipe::FontFace {
        name: name.into(),
        family: family.into(),
        weight,
        italic,
    }
}
fn lottie(name: &str, nm: &str) -> Recipe {
    Recipe::LottieJson {
        name: name.into(),
        nm: nm.into(),
    }
}
fn context_file(name: &str, file: &str) -> Recipe {
    Recipe::ContextFile {
        name: name.into(),
        file: file.into(),
    }
}
fn path_literal(name: &str, d_prefix: &str) -> Recipe {
    Recipe::PathLiteral {
        name: name.into(),
        d_prefix: d_prefix.into(),
    }
}
fn content_url(name: &str, hash_prefix: &str) -> Recipe {
    Recipe::ContentUrl {
        name: name.into(),
        hash_prefix: hash_prefix.into(),
    }
}
fn defaulted(name: &str, key: &str) -> Recipe {
    Recipe::DefaultedAsset {
        name: name.into(),
        key: key.into(),
    }
}
fn pinned(name: &str, file: &str) -> Recipe {
    Recipe::AssetFile {
        name: name.into(),
        file: file.into(),
    }
}

pub fn icons() -> Vec<Recipe> {
    vec![
        icon("icon/AccessibilityIcon", "AccessibilityIcon"),
        icon("icon/AnalyticsIcon", "AnalyticsIcon"),
        icon("icon/AppsIcon", "AppsIcon"),
        icon("icon/ArrowAngleLeftUpIcon", "ArrowAngleLeftUpIcon"),
        icon("icon/ArrowAngleRightUpIcon", "ArrowAngleRightUpIcon"),
        icon("icon/ArrowLargeLeftIcon", "ArrowLargeLeftIcon"),
        icon("icon/ArrowLargeRightIcon", "ArrowLargeRightIcon"),
        icon("icon/ArrowSmallLeftIcon", "ArrowSmallLeftIcon"),
        icon("icon/ArrowSmallRightIcon", "ArrowSmallRightIcon"),
        icon("icon/ArrowsUpDownIcon", "ArrowsUpDownIcon"),
        icon("icon/AtIcon", "AtIcon"),
        icon("icon/AttachmentIcon", "AttachmentIcon"),
        icon("icon/BellIcon", "BellIcon"),
        icon("icon/BicycleIcon", "BicycleIcon"),
        icon("icon/BoostGemIcon", "BoostGemIcon"),
        icon("icon/CalendarIcon", "CalendarIcon"),
        icon("icon/CalendarPlusIcon", "CalendarPlusIcon"),
        icon("icon/ChatIcon", "ChatIcon"),
        icon("icon/ChatCheckIcon", "ChatCheckIcon"),
        icon("icon/ChatMarkUnreadIcon", "ChatMarkUnreadIcon"),
        icon("icon/ChatShieldIcon", "ChatShieldIcon"),
        icon("icon/ChatSpeakIcon", "ChatSpeakIcon"),
        icon("icon/CheckmarkLargeIcon", "CheckmarkLargeIcon"),
        icon("icon/CheckmarkSmallBoldIcon", "CheckmarkSmallBoldIcon"),
        icon("icon/ChevronLargeDownIcon", "ChevronLargeDownIcon"),
        icon("icon/ChevronLargeRightIcon", "ChevronLargeRightIcon"),
        icon("icon/ChevronSmallDownIcon", "ChevronSmallDownIcon"),
        icon("icon/ChevronSmallLeftIcon", "ChevronSmallLeftIcon"),
        icon("icon/ChevronSmallRightIcon", "ChevronSmallRightIcon"),
        icon("icon/ChevronSmallUpIcon", "ChevronSmallUpIcon"),
        icon("icon/CircleCheckIcon", "CircleCheckIcon"),
        icon("icon/CircleErrorIcon", "CircleErrorIcon"),
        icon("icon/CirclePlusIcon", "CirclePlusIcon"),
        icon("icon/CircleInformationIcon", "CircleInformationIcon"),
        icon("icon/CircleQuestionIcon", "CircleQuestionIcon"),
        icon("icon/CircleXIcon", "CircleXIcon"),
        icon("icon/ClockIcon", "ClockIcon"),
        icon("icon/ClydeIcon", "ClydeIcon"),
        icon("icon/CompassIcon", "CompassIcon"),
        icon("icon/CopyIcon", "CopyIcon"),
        icon("icon/CreditCardIcon", "CreditCardIcon"),
        icon("icon/CrownIcon", "CrownIcon"),
        icon("icon/DenyIcon", "DenyIcon"),
        icon("icon/DoorExitIcon", "DoorExitIcon"),
        icon("icon/DownloadIcon", "DownloadIcon"),
        icon("icon/DragIcon", "DragIcon"),
        icon("icon/EmbedIcon", "EmbedIcon"),
        icon(
            "icon/EmojiSlightlySmilingFaceIcon",
            "EmojiSlightlySmilingFaceIcon",
        ),
        icon("icon/EyeDropperIcon", "EyeDropperIcon"),
        icon("icon/EyeIcon", "EyeIcon"),
        icon("icon/EyeSlashIcon", "EyeSlashIcon"),
        icon("icon/FileIcon", "FileIcon"),
        icon("icon/FileUpIcon", "FileUpIcon"),
        icon("icon/FiltersHorizontalIcon", "FiltersHorizontalIcon"),
        icon("icon/FireIcon", "FireIcon"),
        icon("icon/FlagIcon", "FlagIcon"),
        icon("icon/FolderIcon", "FolderIcon"),
        icon("icon/FolderPlusIcon", "FolderPlusIcon"),
        icon("icon/FoodIcon", "FoodIcon"),
        icon("icon/FriendsIcon", "FriendsIcon"),
        icon("icon/FullscreenEnterIcon", "FullscreenEnterIcon"),
        icon("icon/FullscreenExitIcon", "FullscreenExitIcon"),
        icon("icon/GameControllerIcon", "GameControllerIcon"),
        icon("icon/GiftIcon", "GiftIcon"),
        icon("icon/GroupIcon", "GroupIcon"),
        icon("icon/AngleBracketsIcon", "AngleBracketsIcon"),
        icon("icon/GroupPlusIcon", "GroupPlusIcon"),
        icon("icon/IdIcon", "IdIcon"),
        icon("icon/HeartIcon", "HeartIcon"),
        icon("icon/HeartOutlineIcon", "HeartOutlineIcon"),
        icon("icon/ImageIcon", "ImageIcon"),
        icon("icon/ImagePlusIcon", "ImagePlusIcon"),
        icon("icon/InboxIcon", "InboxIcon"),
        icon("icon/KeyboardIcon", "KeyboardIcon"),
        icon("icon/LanguageIcon", "LanguageIcon"),
        icon("icon/LinkIcon", "LinkIcon"),
        icon("icon/LockIcon", "LockIcon"),
        icon("icon/MagnifyingGlassIcon", "MagnifyingGlassIcon"),
        icon("icon/MagnifyingGlassMinusIcon", "MagnifyingGlassMinusIcon"),
        icon("icon/MagnifyingGlassPlusIcon", "MagnifyingGlassPlusIcon"),
        icon("icon/MicrophoneIcon", "MicrophoneIcon"),
        icon("icon/MoreHorizontalIcon", "MoreHorizontalIcon"),
        icon("icon/NatureIcon", "NatureIcon"),
        icon("icon/NitroWheelIcon", "NitroWheelIcon"),
        icon("icon/ObjectIcon", "ObjectIcon"),
        icon("icon/PaintPaletteIcon", "PaintPaletteIcon"),
        icon("icon/PaperPlusIcon", "PaperPlusIcon"),
        icon("icon/PauseIcon", "PauseIcon"),
        icon("icon/PencilIcon", "PencilIcon"),
        icon("icon/PhoneCallIcon", "PhoneCallIcon"),
        icon("icon/PinIcon", "PinIcon"),
        icon("icon/PlayIcon", "PlayIcon"),
        icon("icon/PlusLargeIcon", "PlusLargeIcon"),
        icon("icon/PlusMediumIcon", "PlusMediumIcon"),
        icon("icon/PlusSmallIcon", "PlusSmallIcon"),
        icon("icon/PollsIcon", "PollsIcon"),
        icon("icon/PuzzlePieceIcon", "PuzzlePieceIcon"),
        icon("icon/QuestsIcon", "QuestsIcon"),
        icon("icon/RefreshIcon", "RefreshIcon"),
        icon("icon/ScreenIcon", "ScreenIcon"),
        icon("icon/SettingsIcon", "SettingsIcon"),
        icon("icon/ShieldIcon", "ShieldIcon"),
        icon("icon/ShieldLockIcon", "ShieldLockIcon"),
        icon("icon/ShieldUserIcon", "ShieldUserIcon"),
        icon("icon/ShopIcon", "ShopIcon"),
        icon("icon/SoundboardIcon", "SoundboardIcon"),
        icon("icon/SparklesIcon", "SparklesIcon"),
        icon("icon/StaffBadgeIcon", "StaffBadgeIcon"),
        icon("icon/StarIcon", "StarIcon"),
        icon("icon/StarOutlineIcon", "StarOutlineIcon"),
        icon("icon/StickerIcon", "StickerIcon"),
        icon("icon/SubscriptionIcon", "SubscriptionIcon"),
        icon("icon/TextIcon", "TextIcon"),
        icon("icon/TextLockIcon", "TextLockIcon"),
        icon("icon/ThreadIcon", "ThreadIcon"),
        icon("icon/ThreadPlusIcon", "ThreadPlusIcon"),
        icon("icon/TrashIcon", "TrashIcon"),
        icon("icon/TrophyIcon", "TrophyIcon"),
        icon("icon/UserCheckIcon", "UserCheckIcon"),
        icon("icon/UserCircleIcon", "UserCircleIcon"),
        icon("icon/UserCircleStatusIcon", "UserCircleStatusIcon"),
        icon("icon/UserIcon", "UserIcon"),
        icon("icon/UserPlatformIcon", "UserPlatformIcon"),
        icon("icon/UserPlusIcon", "UserPlusIcon"),
        icon("icon/VideoIcon", "VideoIcon"),
        icon("icon/VideoSlashIcon", "VideoSlashIcon"),
        icon("icon/MicrophoneSlashIcon", "MicrophoneSlashIcon"),
        icon("icon/HeadphonesIcon", "HeadphonesIcon"),
        icon("icon/HeadphonesSlashIcon", "HeadphonesSlashIcon"),
        icon("icon/PhoneHangUpIcon", "PhoneHangUpIcon"),
        icon("icon/ScreenArrowIcon", "ScreenArrowIcon"),
        icon("icon/ScreenStreamIcon", "ScreenStreamIcon"),
        icon("icon/ActivitiesIcon", "ActivitiesIcon"),
        icon("icon/WaveformIcon", "WaveformIcon"),
        icon("icon/WaveformSlashIcon", "WaveformSlashIcon"),
        by_path(
            "icon/slash_box",
            "M5 2a3 3 0 0 0-3 3v14a3 3 0 0 0 3 3h14a3 3 0 0 0 3-3V5a3 3 0 0 0-3-3H5Zm12.79 3.37",
        ),
        by_path("icon/rtc_ping", "M2 3a1 1 0 0 1 1-1 19 19 0 0 1 19 19"),
        by_path(
            "icon/server_mute",
            "M12 2c.33 0 .51.35.4.66a6.99 6.99 0 0 0 3.04 8.37",
        ),
        by_path(
            "icon/server_deafen",
            "M12.38 1c.38.02.58.45.4.78-.15.3-.3.62-.4.95",
        ),
        icon("icon/VoiceLowIcon", "VoiceLowIcon"),
        icon("icon/VoiceNormalIcon", "VoiceNormalIcon"),
        icon("icon/VoiceXIcon", "VoiceXIcon"),
        icon("icon/TextWarningIcon", "TextWarningIcon"),
        icon("icon/VoiceLockIcon", "VoiceLockIcon"),
        icon("icon/VoiceWarningIcon", "VoiceWarningIcon"),
        icon("icon/AnnouncementsIcon", "AnnouncementsIcon"),
        icon("icon/AnnouncementsLockIcon", "AnnouncementsLockIcon"),
        icon("icon/AnnouncementsWarningIcon", "AnnouncementsWarningIcon"),
        icon("icon/ThreadLockIcon", "ThreadLockIcon"),
        icon("icon/ThreadWarningIcon", "ThreadWarningIcon"),
        by_markup("icon/ThreadSpine", &["M0 13H2c0 1.6569", "M6 18V16H9v2H6"]),
        icon("icon/StageIcon", "StageIcon"),
        icon("icon/StageLockIcon", "StageLockIcon"),
        icon("icon/ForumIcon", "ForumIcon"),
        icon("icon/ForumLockIcon", "ForumLockIcon"),
        icon("icon/ForumWarningIcon", "ForumWarningIcon"),
        icon("icon/ImageLockIcon", "ImageLockIcon"),
        icon("icon/ImageWarningIcon", "ImageWarningIcon"),
        icon("icon/HubIcon", "HubIcon"),
        icon("icon/BookCheckIcon", "BookCheckIcon"),
        icon("icon/WarningIcon", "WarningIcon"),
        icon("icon/WindowLaunchIcon", "WindowLaunchIcon"),
        icon("icon/XLargeIcon", "XLargeIcon"),
        icon("icon/XNeutralIcon", "XNeutralIcon"),
        icon("icon/XSmallIcon", "XSmallIcon"),
        icon(
            "icon/DoubleChevronSmallRightIcon",
            "DoubleChevronSmallRightIcon",
        ),
        icon("icon/MenuIcon", "MenuIcon"),
        icon("icon/SendMessageIcon", "SendMessageIcon"),
        by_path("icon/replay", "M12,5 L12,1 L7,6 L12,11"),
        by_path("icon/checkmark_small", "M19.06 6.94a1.5 1.5"),
        by_path("icon/keybind_arrow", "M4.59185 1.69642L1.32007"),
        by_path(
            "icon/double_checkmark",
            "M16.7 8.7a1 1 0 0 0-1.4-1.4l-3.26 3.24",
        ),
        by_path(
            "icon/jump_to_message",
            "M12.2 22c.5-.01.66-.84.37-1.24A3 3 0 0 1 15 16h.46",
        ),
        by_markup(
            "icon/win_minimize",
            &["viewBox=\"0 0 12 12\"", "<rect", "height=\"1\""],
        ),
        by_markup(
            "icon/win_maximize",
            &["viewBox=\"0 0 12 12\"", "<rect", "stroke=\"currentColor\""],
        ),
        by_markup("icon/win_close", &["viewBox=\"0 0 12 12\"", "<polygon"]),
        by_path("icon/discord_status", "M6.99471698,9.67522659"),
        by_path("icon/channel_text_lock", "M12.5 8c.28 0 .5.22.5.5V9"),
        by_path(
            "icon/channel_voice_lock",
            "M16 4h.5v-.5a2.5 2.5 0 0 1 5 0V4 && M16.5 12c-.28 0-.5.23-.54.5a3",
        ),
        by_path("icon/channel_forum_lock", "M13.58 3.23c.24-.33.16-.86"),
        by_path(
            "icon/channel_forum",
            "M18.91 12.98a5.45 5.45 0 0 1 2.18 6.2 && M19.24 10.86c.32.16.72-.02",
        ),
        by_path(
            "icon/person_shield",
            "M20.3 5.41h-.39c-.84 0-1.52-.65-1.52-1.46 && Zm-8.32.03a3.15 3.15 0 1 1-.01 6.3 3.15 3.15 0 0 1 .01-6.3Zm4.52 11.67",
        ),
    ]
}

pub fn fonts() -> Vec<Recipe> {
    vec![
        font("font/ggsans", "gg sans", 400, false),
        font("font/ggsansitalic", "gg sans", 400, true),
        font("font/ggsansmedium", "gg sans", 500, false),
        font("font/ggsansmediumitalic", "gg sans", 500, true),
        font("font/ggsanssemibold", "gg sans", 600, false),
        font("font/ggsanssemibolditalic", "gg sans", 600, true),
        font("font/ggsansbold", "gg sans", 700, false),
        font("font/ggsansbolditalic", "gg sans", 700, true),
        font("font/ggsansextrabold", "gg sans", 800, false),
        font("font/ggsansextrabolditalic", "gg sans", 800, true),
        font("font/ggsansmono", "gg mono", 400, false),
        font("font/ggsansmonobold", "gg mono", 700, false),
    ]
}

pub fn lottie_animations() -> Vec<Recipe> {
    vec![
        lottie("lottie/mic", "Lottie_HoverOnOff_Microphone"),
        lottie("lottie/deafen", "Lottie_HoverOnOff_Deafen"),
        lottie("lottie/gif_button", "Lottie_HoverOn_GIF"),
        lottie(
            "lottie/noise_suppression",
            "Lottie_HoverOnOff_NoiseSuppression",
        ),
        lottie("lottie/camera", "Lottie_HoverOnOff_Camera"),
        lottie("lottie/end_call", "Lottie_Hover_EndCall"),
        lottie("lottie/screenshare", "Lottie_Hover_Screenshare"),
        lottie("lottie/soundboard", "Lottie_HoverOn_Soundboard"),
        lottie("lottie/settings", "L"),
    ]
}

pub fn file_icons() -> Vec<Recipe> {
    vec![
        context_file("file_icon/acrobat", "icon-file-acrobat.svg"),
        context_file("file_icon/ae", "icon-file-ae.svg"),
        context_file("file_icon/ai", "icon-file-ai.svg"),
        context_file("file_icon/archive", "icon-file-archive.svg"),
        context_file("file_icon/audio", "icon-file-audio.svg"),
        context_file("file_icon/code", "icon-file-code.svg"),
        context_file("file_icon/document", "icon-file-document.svg"),
        context_file("file_icon/drop_image", "icon-file-image.svg"),
        context_file("file_icon/photoshop", "icon-file-ps.svg"),
        context_file("file_icon/sketch", "icon-file-sketch.svg"),
        context_file("file_icon/spreadsheet", "icon-file-spreadsheet.svg"),
        context_file("file_icon/unknown", "icon-file-unknown.svg"),
        context_file("file_icon/video", "icon-file-video.svg"),
        context_file("file_icon/webcode", "icon-file-webcode.svg"),
    ]
}

pub fn sounds() -> Vec<Recipe> {
    vec![
        context_file("sound/message1", "message1.mp3"),
        context_file("sound/message3", "message3.mp3"),
        context_file("sound/mention1", "mention1.mp3"),
        context_file("sound/mute", "mute.mp3"),
        context_file("sound/unmute", "unmute.mp3"),
        context_file("sound/deafen", "deafen.mp3"),
        context_file("sound/undeafen", "undeafen.mp3"),
        context_file("sound/disconnect", "disconnect.mp3"),
        context_file("sound/reconnect", "reconnect.mp3"),
        context_file("sound/ptt_start", "ptt_start.mp3"),
        context_file("sound/ptt_stop", "ptt_stop.mp3"),
        context_file("sound/user_join", "user_join.mp3"),
        context_file("sound/user_leave", "user_leave.mp3"),
        context_file("sound/user_moved", "user_moved.mp3"),
        context_file("sound/call_calling", "call_calling.mp3"),
        context_file("sound/call_ringing", "call_ringing.mp3"),
        context_file("sound/stream_started", "stream_started.mp3"),
        context_file("sound/stream_ended", "stream_ended.mp3"),
        context_file("sound/stream_user_joined", "stream_user_joined.mp3"),
        context_file("sound/stream_user_left", "stream_user_left.mp3"),
        context_file("sound/camera_on", "camera_on.mp3"),
        context_file("sound/camera_off", "camera_off.mp3"),
        context_file("sound/activity_launch", "activity_launch.mp3"),
        context_file("sound/activity_end", "activity_end.mp3"),
        context_file("sound/activity_user_join", "activity_user_join.mp3"),
        context_file("sound/activity_user_left", "activity_user_left.mp3"),
    ]
}

pub fn pinned_files() -> Vec<Recipe> {
    vec![
        pinned("connection_icon/steam", "b68f2f4df4af9085.png"),
        pinned("connection_icon/github", "1cc92a6d139af44d.png"),
        pinned("connection_icon/twitch", "45900de231038a6d.png"),
        pinned("connection_icon/spotify", "40518c6ced9fed93.png"),
        pinned("connection_icon/twitter", "90d18081c2eb2ea3.png"),
        pinned("connection_icon/twitter_legacy", "7520545bf8990675.png"),
        pinned("connection_icon/reddit", "afc3f4fa75178004.png"),
        pinned("connection_icon/youtube", "c569f6e10f1fa390.png"),
        pinned("connection_icon/facebook", "bda550439e2a1af0.png"),
        pinned("connection_icon/instagram", "ea0692691904fca9.png"),
        pinned("connection_icon/tiktok", "b4002d11fb3b5c1c.png"),
        pinned("connection_icon/xbox", "8cea2073e188eabe.png"),
        pinned("connection_icon/playstation", "ae74d3de4958bf78.png"),
        pinned("connection_icon/battlenet", "0e1a419c00a471a2.png"),
        pinned("connection_icon/leagueoflegends", "fc30b78cd5e1db04.png"),
        pinned("connection_icon/epicgames", "e87d94bf0041b63e.png"),
        pinned("connection_icon/riotgames", "fa9e14fc0fd3680f.png"),
        pinned("connection_icon/roblox", "268c81933d752854.png"),
        pinned("connection_icon/paypal", "81a39a4672d62d60.png"),
        pinned("connection_icon/ebay", "d0f7fa72a2ca6388.png"),
        pinned("connection_icon/mastodon", "53fcd6d102d44c2b.png"),
        pinned("connection_icon/bluesky", "8a42ef6bc4dd4e0d.png"),
        pinned("connection_icon/crunchyroll", "4e56bb1ecb9b9dc3.png"),
        pinned("connection_icon/domain", "8d54a5919eb645f9.png"),
        pinned("connection_icon/bungie", "9b762e8a42709cc4.png"),
        pinned("connection_icon/skype", "4f2cb7753e5d5e61.png"),
        pinned("connection_icon/amazon-music", "cda6fb74e92e32f1.png"),
        pinned("connection_icon/samsung", "d50e7459b395e5d0.png"),
        pinned("connection_item/dota2", "44e2229fd99bc14e.png"),
        pinned("connection_item/tf2", "9eb53b4fa3cc12d5.png"),
        by_path("icon/arrow_up_right", "M8 5a1 1 0 0 0 0 2h7.59L5.29 17.3"),
        pinned("connection/battlenet", "0e1a419c00a471a2.png"),
        pinned("connection/bungie", "a35b652323167799.png"),
        pinned("connection/crunchyroll", "4e56bb1ecb9b9dc3.png"),
        pinned("connection/epicgames", "ecf035f3cf08a860.png"),
        pinned("connection/leagueoflegends", "fc30b78cd5e1db04.png"),
        pinned("connection/playstation", "46a23515a87620b9.png"),
        pinned("connection/riotgames", "fa9e14fc0fd3680f.png"),
        pinned("connection/roblox", "5d078bf414a0124c.png"),
        pinned("connection/spotify", "40518c6ced9fed93.png"),
        pinned("connection/steam", "07ca603d9273b502.png"),
        pinned("connection/twitch", "45900de231038a6d.png"),
        pinned("connection/xbox", "09781ebf5509f48e.png"),
        pinned("connection/youtube", "c569f6e10f1fa390.png"),
        pinned("create_guild/arrow", "050c2ac76232eff6.svg"),
        pinned("create_guild/artists", "d8fed3f03866afe2.svg"),
        pinned("create_guild/create_my_own", "b30f13ee315c2568.svg"),
        pinned("create_guild/discover", "7cd1df4da22b9bff.svg"),
        pinned("create_guild/friends", "d804200b134c9327.svg"),
        pinned("create_guild/gaming", "261f952bf028fa34.svg"),
        pinned("create_guild/intent_community", "2f0ebe181e1cce00.svg"),
        pinned("create_guild/intent_friends", "7f384b2c31f586cd.svg"),
        pinned("create_guild/local_community", "31f3db39524533b6.svg"),
        pinned("create_guild/school_club", "2f1587b0c86b42e2.svg"),
        pinned("create_guild/study_group", "4900b53e7b34c3a5.svg"),
        pinned("image/clyde_avatar", "9380e4b5bd8d267c.png"),
        pinned("image/role_icon_preview_dark", "6263c90899ad6dae.svg"),
        pinned("image/role_icon_preview_light", "36799e0a695f945e.svg"),
        content_url("image/wumpus_avatar", "2532e8cd0600ccfbc61ff81c21959bde"),
        content_url("image/threads_empty", "a55e53565529b99537f24e9111b39ad8"),
        pinned("image/emoji_button_sprite", "1b05b930a77fbff9.png"),
        pinned("image/application_command", "3934256e60927112.svg"),
    ]
}

fn images() -> Vec<Recipe> {
    vec![
        pinned("image/emoji_button_sprite_grey", "decf85f79e02d276.png"),
        pinned("image/no_results", "05207174d5432737.svg"),
        pinned("image/auth_wordmark", "131c318dd45b7aa4.svg"),
        pinned("image/auth_background", "d8680b1c1576ecc8.svg"),
        pinned("image/captcha_graphic", "a1c385fb82c39bab.svg"),
        pinned("image/search_no_results", "0dfcd735f1a5349c.svg"),
        pinned("image/pins_empty", "1772d54cb566d95b.svg"),
        pinned("image/tutorial_arrow_1", "a8546d03debd645f.svg"),
        pinned("image/tutorial_arrow_2", "43a0c71f5cc76261.svg"),
        pinned("image/tutorial_arrow_3", "6c67a101e81dc0ad.svg"),
    ]
}

fn shapes() -> Vec<Recipe> {
    vec![
        path_literal(
            "shape/upload_camera_lens",
            "M40 29C37.794 29 36 30.794 36 33C36 35.207",
        ),
        path_literal(
            "shape/upload_camera_body",
            "M48 26.001H46.07C45.402 26.001 44.777",
        ),
        path_literal(
            "shape/upload_plus_badge",
            "M73.3332 11.4075H68.5924V6.66675H67.4072V",
        ),
        path_literal("shape/flower_star", "M5.52995 0.431867C6.27995"),
        path_literal(
            "shape/unread_pill_cap",
            "M8.16639 0.5H9C10.933 0.5 12.5 2.067 12.5 4V9",
        ),
        path_literal("shape/badge_check", "M18.7 7.3a1 1 0 0 1 0 1.4l-8 8"),
        path_literal(
            "shape/partner_mark_a",
            "M10.5906 6.39993L9.19223 7.29993C8.99246",
        ),
        path_literal(
            "shape/partner_mark_b",
            "M13.4871 7.79985C13.4871 8.19985 13.2874",
        ),
        path_literal(
            "shape/squircle_mask",
            "M0 17.4545C0 11.3449 0 8.29005 1.18902",
        ),
        path_literal(
            "shape/switch_thumb_x_a",
            "M4.10585 5.3837L5.37864 4.11091L11.884",
        ),
        path_literal(
            "shape/switch_thumb_x_b",
            "M10.61632 4.11091L11.88912 5.3837L5.3837",
        ),
        path_literal(
            "shape/switch_thumb_check_a",
            "M6.31649 11.88304L5.0437 10.61032L11.44792",
        ),
        path_literal(
            "shape/switch_thumb_check_b",
            "M3.26914 8.87224L4.54194 7.59943L7.5588",
        ),
    ]
}

pub fn videos() -> Vec<Recipe> {
    vec![defaulted("video/app_spinner", "webmDark")]
}

pub fn all() -> Vec<Recipe> {
    let mut all = icons();
    all.extend(videos());
    all.extend(images());
    all.extend(sounds());
    all.extend(shapes());
    all.extend(fonts());
    all.extend(lottie_animations());
    all.extend(file_icons());
    all.extend(pinned_files());
    all
}

pub const NOT_FROM_DISCORD: &[&str] = &[
    "arrow-down",
    "arrow-up",
    "check-checkbox",
    "check-context-menu",
    "chevron-right-emoji-picker",
    "close-title-bar",
    "double-chevron-right",
    "emoji",
    "icon-caret",
    "list-arrow",
    "maximize",
    "menu",
    "message-list-4",
    "message-list-5",
    "send",
    "status",
];

