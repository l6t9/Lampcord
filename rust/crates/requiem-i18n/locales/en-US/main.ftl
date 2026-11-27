## English (United States): the fallback every other locale falls back to.
##
## Values are Discord markdown, drawn by `message_text`: **bold**, __underline__, ~~strike~~, `code`,
## <@123> / <#123> mentions and <:name:123> custom emoji all work. A literal "{" is {"{"}.

upload-failed =
    Your message could not be delivered.

    You can see the full list of reasons here: https://support.discord.com/hc/en-us/articles/360060145013
dismiss-message = Dismiss message

welcome-back = Welcome back, **{ $name }**!
mentioned-you = { $who } mentioned you in { $channel }.
invited = { $who } invited you to **{ $server }** <:wave:1234567890123456789>

unread-messages =
    { $count ->
        [one] **{ $count }** new message
       *[other] **{ $count }** new messages
    }
member-count =
    { $count } { $count ->
        [one] member
       *[other] members
    }

## Deliberately not translated: the fallback test.
i18n-test-english-only = English only
