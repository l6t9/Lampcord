## Strings of desktop/updater (the title-bar update button, read off the live client 2026-10-17, and the restart dialog)
updater-downloading = Downloading Update
updater-ready = Update Ready!
updater-confirm-title = Restart to update?
updater-confirm-body = Version { $version } is ready. The app will close, install it and open again.
updater-confirm-body-voice = Version { $version } is ready. The app will close, install it and open again, and you'll be put back in your voice channel.
updater-confirm-update = Update and restart
updater-failed = Could not start the update. The app was left as it is.

## Settings > App Updates (this client's own page; the live client has none)
update-settings-auto = Update automatically
update-settings-auto-description = Look for new versions in the background and download them. You choose when to restart into one.
update-settings-version = Version
update-settings-check = Check for Updates
update-settings-status-idle = Updates have not been checked yet.
update-settings-status-checking = Checking for updates…
update-settings-status-up-to-date = You're up to date.
update-settings-status-downloading = Downloading version { $version }…
update-settings-status-ready = Version { $version } is ready. Use the green button in the title bar to restart into it.
update-settings-status-failed = The last check did not work: { $reason }

## Update from a custom server (the restart dialog) and the update server options on that page
updater-confirm-body-custom = Version { $version } comes from { $host }, a custom update server you added. The app will close, install it and open again.
updater-confirm-body-custom-voice = Version { $version } comes from { $host }, a custom update server you added. The app will close, install it and open again, and you'll be put back in your voice channel.
update-settings-danger-show = Show update server options
update-settings-danger-hide = Hide update server options
update-settings-danger-subtitle = You almost certainly do not need these.
update-settings-warning-title = Stop. You almost never need this.
update-settings-warning-body = A custom update server can install any code on your computer, and this app will run it. The official servers are used automatically, and one of them being down is not a reason to change anything here. Only continue if the official servers are gone for good and you trust whoever runs the replacement.
update-settings-server-address = Update server address
update-settings-server-placeholder = https://updates.example.com
update-settings-continue = Continue
update-settings-fetching = Contacting the server…
update-settings-fetch-failed = Could not read the server's keys: { $reason }
update-settings-review-title = { $host } says it signs its updates with:
update-settings-review-hint = Compare these fingerprints with ones published by someone you trust. If they do not match, stop.
update-settings-type-host = Type { $host } to confirm
update-settings-use-server = Use this server
update-settings-cancel = Cancel
update-settings-final-title = Use a custom update server?
update-settings-final-body = From now on this app installs whatever { $host } signs, and the official servers are no longer used. You can switch back at any time on this page.
update-settings-custom-badge = Custom update server: { $host }
update-settings-restore = Restore official servers
