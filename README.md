# Lampcord
An M3E non-electron discord client written from the ground up.

---

## What the hell is a "lampcord" ?
Lampcord, our own discord client built using KMP (Kotlin Multiplatform) to have the Material 3 Expressive (M3E) design language.

### Here's the concept:
Discord uses Electron which is known to be **slow**, **bloated** and **limited**. **Lampcord** is built on KMP which **isn't bloated at all** and in fact **inceridbly customizable**, and **_very fast_**.

# Screenshots.. please ?
Sure, here you go!

<img width="1264" height="2780" alt="lampcord-p1" src="https://github.com/user-attachments/assets/8963d016-ec49-4273-9ab3-f92f7ec6f50a" />
<img width="2876" height="1796" alt="lampcord-p" src="https://github.com/user-attachments/assets/a35ecc79-a978-4b14-83a3-0197821b5ce9" />
<img width="1264" height="2780" alt="lampcord-s1" src="https://github.com/user-attachments/assets/6c62d1fe-36c9-4a6d-acf8-aa604603648e" />
<img width="2870" height="1796" alt="lampcord-s" src="https://github.com/user-attachments/assets/59aa0f6d-0636-41bc-9935-00a9a3ba84d3" />
<img width="1264" height="2780" alt="lampcord-1" src="https://github.com/user-attachments/assets/405d73cf-f0d1-4610-b545-bedcc6803bc1" />
<img width="2874" height="1796" alt="lampcord" src="https://github.com/user-attachments/assets/de2a75e1-2e5a-40e6-a0b2-00bac33d9ae9" />

---

>[!Warning]
>**Lampcord is in BETA**
>
>The client is still in developement and many issues that we're currently fixing.
>
>So.. don't delete your current client yet. Unless you're sure about it..

# What are the features ?
Well, it has many. So many that I cannot list all of them here. But hey, if you really want to know what does it have for you, then.. why not give it a try ?
Anyways, here is a quick summary of what it can do:
- **M3E**: Also known as Material 3 Expressive. It's a fun, non-corporal design language built by Google to be super flexible and match the stock Android theme.
- **Cross platform**: Works on macOS, iOS, Linux, and even WINDOWS (Yuck..) and all of your preferences are synced!
- **Native encryption**: Are you paranoid ? Uh-hum.. (*clears throat*) sorry.. uhmm.. Do you care about your privacy ? Well, lampcord does ! And this is why we encrypt whatever the heck you keep spamming in your least favourite server. Wheter in Voice chat or in a text channel.
- **No more nitro-only emojis**: Ever had the perfect emoji to react to something only to find out it belongs to another server and you have to buy nitro in order to use it ? Well, Lampcord removes that barrier. By doing some black magic and turning the emojis into gifs, you can send any emoji you like onto the chat without risking getting a cease and desist letter from discord.

And that's not all ! Try it to discover the rest for yourself ;)

---

<div align="center">

<h1><a id="download-now"></a>Download Now</h1>

<h2>Stable Release</h2>

<table>
  <tr>
    <th align="center">Obtainium</th>
  </tr>
  <tr>
    <td align="center">
      <a href="https://apps.obtainium.imranr.dev/redirect?r=obtainium://add/https://github.com/l6t9/Lampcord/">
        <img src="https://github.com/ImranR98/Obtainium/blob/main/assets/graphics/badge_obtainium.png" alt="Download from Obtainium" height="50">
      </a>
    </td>
  </tr>
  <tr>
    <th align="center" colspan="2">GitHub</th>
  </tr>
  <tr>
    <td align="center" colspan="2">
      <a href="https://github.com/l6t9/Lampcord/releases/latest/">
        <img src="https://github.com/machiav3lli/oandbackupx/blob/034b226cea5c1b30eb4f6a6f313e4dadcbb0ece4/badge_github.png" alt="Download from GitHub" height="75">
      </a>
    </td>
  </tr>
</table>

<h2>Nightly Build</h2>

<table>
  <thead>
    <tr>
      <th align="center">GitHub</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td align="center">
        <a href="https://github.com/l6t9/Lampcord/releases/tag/nightly">
          <img src="https://github.com/machiav3lli/oandbackupx/blob/034b226cea5c1b30eb4f6a6f313e4dadcbb0ece4/badge_github.png" alt="Download from GitHub" height="75">
        </a>
      </td>
    </tr>
  </tbody>
</table>

</div>

---

## In case you're a nerd and hate yourself..
Then you surely want to **build it from surce** (not a typo btw)

### Prerequisites

- **JDK 17+**
- **Android SDK** (API 34+)
- **CMake & NDK** (for native voice/DAVE components)

### Desktop (Linux / macOS / Windows)

```bash
# Run Desktop App
./gradlew :desktopApp:run

# Package Linux AppImage
./gradlew :desktopApp:createAppImageLocal

# Package macOS DMG (macOS only)
./gradlew :desktopApp:packageReleaseDmg

# Package Windows Zip
./gradlew :desktopApp:createDistributable
```

The macOS DMG is unsigned, like the iOS IPA: there is no `codesign` identity or notarisation
in the build, so macOS will need `xattr -d com.apple.quarantine` after first download.

### Android

```bash
# For assembling the debug APK
./gradlew :androidApp:assembleDebug

# For assembling the release APK
./gradlew :androidApp:assembleRelease
```

---

## Disclaimer & Notice

Lampcord is a third-party open-source client and is **not** affiliated with, endorsed by, or sponsored by Discord Inc. Using third-party clients may violate Discord's Terms of Service. Use at your own risk.

---

## License

Lampcord is released under the [GNU General Public License v3.0 (GPL-3.0)](LICENSE).

# Thank you for trying out Lampcord :)
