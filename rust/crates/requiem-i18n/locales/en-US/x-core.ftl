## Hand-added strings (not produced by the codemod).

reactions-unable-to-remove-reaction = Unable to remove reaction
reactions-unable-to-add-reaction = Unable to add reaction
quick-switcher-server = SERVER
search-state-hint-user = user
search-state-hint-channel = channel
search-state-hint-date = date
attachments-upload-to-channel = Upload to #{ $name }
attachments-too-many-uploads-body = You can only upload { $max } files at a time!
channel-start-welcome = Welcome to #{ $name }!
channel-start-description = This is the start of the #{ $name } channel.
channel-start-description-private = This is the start of the #{ $name } **private** channel.
context-menus-leave-server-title = Leave '{ $name }'
context-menus-leave-server-body = Are you sure you want to leave { $name }? You won't be able to rejoin this server unless you are re-invited.
create-guild-discovery-note = Check out Discoverable communities in Server Discovery.
create-guild-default-server-name = { $name }'s server
emoji-picker-top-emoji-in = Top Emoji in { $name }
full-profile-view-main-profile = View Main Profile
full-profile-view-per-server-profile = View Per-server Profile
full-profile-aka = AKA { $aka }
full-profile-no-activity-title = { $name } doesn’t have any activity to share here
full-profile-no-activity-body = So mysterious! ask them to tell you a secret
full-profile-block-title = Block { $name }?
mutual-friends =
    { $count ->
        [one] { $count } Mutual Friend
       *[other] { $count } Mutual Friends
    }
mutual-servers =
    { $count ->
        [one] { $count } Mutual Server
       *[other] { $count } Mutual Servers
    }
## Dates and times. The message arguments are preformatted pieces; a locale decides their order and wording.
time-clock = { $hour12 }:{ $minute } { $meridiem }
time-date-medium = { $month } { $day }, { $year }
time-date-long = { $month } { $day }, { $year }
time-month-year-long = { $month } { $year }
time-month-year-short = { $month } { $year }
time-date-numeric = { $month }/{ $day }/{ $year }
time-date-time = { $date }, { $time }
time-today-at = Today at { $time }
time-yesterday-at = Yesterday at { $time }
time-month-long-1 = January
time-month-long-2 = February
time-month-long-3 = March
time-month-long-4 = April
time-month-long-5 = May
time-month-long-6 = June
time-month-long-7 = July
time-month-long-8 = August
time-month-long-9 = September
time-month-long-10 = October
time-month-long-11 = November
time-month-long-12 = December
time-month-short-1 = Jan
time-month-short-2 = Feb
time-month-short-3 = Mar
time-month-short-4 = Apr
time-month-short-5 = May
time-month-short-6 = Jun
time-month-short-7 = Jul
time-month-short-8 = Aug
time-month-short-9 = Sep
time-month-short-10 = Oct
time-month-short-11 = Nov
time-month-short-12 = Dec
custom-status-clear-option = { $span } ({ $time })
custom-status-clear-option-tomorrow = { $span } (tomorrow at { $time })
custom-status-clear-at = Clear at { $time }
custom-status-clear-tomorrow-at = Clear tomorrow at { $time }
home-mute-user = Mute @{ $name }
home-group-members =
    { $count ->
        [one] { $count } Member
       *[other] { $count } Members
    }
home-section-title = { $title } — { $count }
home-friend-request-sent = Success! Your friend request to { $username } was sent.
member-list-online = Online
member-list-offline = Offline
member-list-group-header = { $name } — { $count }
message-list-placeholder-group = Message { $name }
message-list-placeholder-dm = Message @{ $name }
message-list-placeholder-channel = Message #{ $name }
message-toolbar-remove-reaction = Click to remove { $name }
message-toolbar-add-reaction-named = Click to react with { $name }
pending-uploads-uploading-files = Uploading { $count } Files
profile-popout-message-placeholder = Message @{ $name }
profile-popout-member-since = Discord Member Since { $date }
search-search-server = Search { $name }
search-n-results =
    { $count ->
        [one] { $count } Result
       *[other] { $count } Results
    }
search-filters-count = Filters ({ $count })
search-filters-clear-count = Clear Filters ({ $count })
search-filters-clear = Clear Filters
search-state-search-for = Search for { $query }
uploads-server-rejected = The server rejected the upload ({ $code }).
uploads-read-failed = Couldn't read the file: { $error }
uploads-network-error = Network error: { $error }
## Shared by the UI engine's own components.
ui-cancel = Cancel
ui-new-badge = new
ui-copy = Copy
ui-copied = Copied
ui-copy-1 = Copied!
ui-copy-2 = Double Copy!
ui-copy-3 = Triple Copy!
ui-copy-4 = Dominating!!
ui-copy-5 = Rampage!!
ui-copy-6 = Mega Copy!!
ui-copy-7 = Unstoppable!!
ui-copy-8 = Wicked Sick!!
ui-copy-9 = Monster Copy!!!
ui-copy-10 = GODLIKE!!!
ui-copy-11 = BEYOND GODLIKE!!!!
common-spoiler = SPOILER
common-app-tag = APP
auth-captcha-unsupported = This instance wants a captcha solved, which this client can't show yet.
auth-captcha-failed = The captcha wasn't accepted. Please try again.
auth-no-token = The server's response had no token in it.
emoji-picker-from = from
create-guild-guidelines = By creating a server, you agree to Discord's [Community Guidelines](https://discord.com/guidelines).
format-bytes =
    { $count ->
        [one] { $count } byte
       *[other] { $count } bytes
    }
format-size = { $value } { $unit }
format-unit-kb = KB
format-unit-mb = MB
format-unit-gb = GB
format-unit-tb = TB
search-filters-example = ex. { $example }
search-filters-example-user = someone
search-filters-example-channel = general
time-meridiem-am = AM
time-meridiem-pm = PM
## Language & Time settings page.
settings-language-select = Select a Language
settings-language-description = Choose the language you want Requiem to display.
settings-language-time-format = Time format
settings-language-time-auto = Auto
settings-language-time-12h = 12-hour
settings-language-time-24h = 24-hour
time-clock-12h = { $hour12 }:{ $minute } { $meridiem }
time-clock-24h = { $hour24 }:{ $minute }
language-name-da = Danish
language-name-de = German
language-name-en-GB = English, UK
language-name-en-US = English, US
language-name-es-ES = Spanish
language-name-es-419 = Spanish, LATAM
language-name-fr = French
language-name-hr = Croatian
language-name-it = Italian
language-name-lt = Lithuanian
language-name-hu = Hungarian
language-name-nl = Dutch
language-name-no = Norwegian
language-name-pl = Polish
language-name-pt-BR = Portuguese, Brazilian
language-name-ro = Romanian, Romania
language-name-fi = Finnish
language-name-sv-SE = Swedish
language-name-vi = Vietnamese
language-name-tr = Turkish
language-name-cs = Czech
language-name-el = Greek
language-name-bg = Bulgarian
language-name-ru = Russian
language-name-uk = Ukrainian
language-name-hi = Hindi
language-name-th = Thai
language-name-zh-CN = Chinese, China
language-name-ja = Japanese
language-name-zh-TW = Chinese, Taiwan
language-name-ko = Korean
message-edit-cancel = cancel
message-edited = (edited)
message-edit-save = save
ui-select-no-results = No results found
dm-start-description = This is the beginning of your direct message history with **{ $name }**.
dm-profile-mutual-servers = Mutual Servers — { $count }
dm-header-pinned-messages = Pinned Messages
dm-header-add-friends-to-dm = Add Friends to DM
dm-header-show-user-profile = Show User Profile
dm-header-hide-user-profile = Hide User Profile
member-list-show = Show Member List
member-list-hide = Hide Member List
profile-popout-friend = Friend
dm-profile-members = Members
full-profile-no-mutual-friends = No Mutual Friends
full-profile-no-mutual-servers = No Mutual Servers
full-profile-no-mutual-friends-yet = No mutual friends yet – perfect time to make a new connection
full-profile-join-the-same-servers-to-meet = Join the same servers to meet more mutual friends 🤝
full-profile-no-servers-in-common = No servers in common
context-menus-copy-user-id = Copy User ID
context-menus-id-copied = Copied!
context-menus-copy-message-id = Copy Message ID
