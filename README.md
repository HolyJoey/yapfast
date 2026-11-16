> [!NOTE]
> **This is HolyJoey's unofficial fork of [Yapfast](https://github.com/yapfast/yapfast).**
> It makes Yapfast look and feel as much like Discord's own app as possible.
> It isn't made or supported by Yapfast's developers or by Discord, so please
> report problems with this fork [here](https://github.com/HolyJoey/yapfast/issues),
> not upstream. Upstream's own README, with its feature tour, is
> [on Yapfast's page](https://github.com/yapfast/yapfast#readme).

# Yapfast, Discord look

Yapfast is a Discord client without the web browser: a native app in Rust that
talks to Discord's API itself and draws its own window with
[Slint](https://slint.dev) and Skia. Upstream measured it at about 30 MB of RAM
idle, where Discord's app takes about 720 MB, and opening in 0.22 seconds
against Discord's 8 ([their measurements](docs/measurements/README.md), of
upstream's build). Servers, DMs, friends, voice, video, screen sharing, forums
and polls are all there.

This fork keeps all of that and makes it look and work like Discord's own app,
compared screen by screen with it.

## What's different from upstream

- **Friends page like Discord's:** the Active Now column, a ⋮ menu on each
  friend (video call, voice call, remove friend), a Pending tab, the game and
  custom status together on one line, and stacked avatars for group DMs.
- **Discord's title bar,** with back/forward history and the page's name.
- **Servers:** server tags after names, avatar decorations, nameplates, and
  display name fonts (names keep their role colour, as in Discord's lists).
  Turn all of these off in
  **Settings → Accessibility → Display Name Styles**. Also bot APP badges,
  channel icons for rules and announcements, an invite button on channels, and
  replies that show the colour, the tag and attachments of the message they
  answer.
- **Server list and header like Discord's:** the server's banner, Discord's
  verified, partnered and discoverable badges, the Boost Goal bar, Channels &
  Roles, a "+" on categories and a gear on channels for those who may manage
  them, and locks on private channels.
- **Messages like Discord's:** Discord's dates, channel links as chips that
  open the channel, the "Welcome to #channel!" header, file cards with edit,
  remove and download, GIF badges and Discord's image grid, embeds with custom
  emoji and coloured mentions, Mark Unread, Copy Message Link and OP badges.
  The hover bar has Add Reaction only, no quick reactions.
- **Stickers:** a sticker picker and button in the message box.
- **Image viewer:** who posted it and when, with Discord's buttons.
- **People:** Discord's right-click menu (Start a Call, Add Note, Play on
  Spotify, Roles) and full profile, with Member Since, roles, connected
  accounts, activities and badges.
- **Voice:** hover someone who's live for Discord's stream preview, with their
  game and Watch Stream; a camera menu, joining with video, and Mic Test.
- **Settings like Discord's:** its pages, groups and names (Data & Privacy,
  Messaging Permissions, Accessibility, System, Language & Time, Activity
  Privacy), with each option where Discord has it.
- **Discord's sounds:** its own notification and call sounds, downloaded from
  Discord like gg sans (see below).
- **Emoji picker like Discord's:** its categories, 9 columns, skin tones and
  `:shortcode:` names.
- **Composer:** no send button (Enter sends), and messages no longer show
  "Sending…" while Discord confirms them.
- **Discord's typeface, gg sans,** and Discord's colours: the idle dot, the
  "New" line and the Onyx theme.
- **Fixes:** member lists refresh after 30 seconds rather than 10 minutes, and
  the connection to Discord stays open while idle, so a message after a quiet
  minute goes out without a new handshake.
- **Security:** rustls, the TLS library every connection to Discord goes
  through, is updated to 0.23.46, which fixes
  [GHSA-c8m5-c952-6cm4](https://github.com/advisories/GHSA-c8m5-c952-6cm4):
  printing a TLS session for debugging could show its keys.

### About gg sans

gg sans is Discord's own typeface, and its licence doesn't allow passing it on,
so it isn't in this repository or in the downloads. The first time Yapfast
starts, it downloads the same font files Discord's web client loads, from
`discord.com`, checks them against the checksums in
[`ui_font.rs`](crates/yapfast-discord/src/ui_font.rs), and keeps them in its
data folder. That's 10 files, about 1 MB together, so it usually takes a
second. There's no popup: you just see the text switch to gg sans once,
without a restart. If the download fails (offline, say), Yapfast keeps its own
typeface, Atkinson Hyperlegible Next, notes it in its log and tries again on
the next start. The downloaded files are in the `fonts` folder of Yapfast's
data folder.

The fonts for display name styles are free fonts under the SIL Open Font
License; see [`ui/fonts/styles`](ui/fonts/styles/README.md). Two of Discord's
name styles, and Discord's notification and call sounds, come from Discord the
same way: downloaded on first start, checked against their checksums, and
never in this repository or the downloads.

## Download

Every [release](https://github.com/HolyJoey/yapfast/releases/latest) has:

| System | File | How |
| --- | --- | --- |
| Debian 13, Ubuntu 24.04, Fedora 40, Mint 22 and newer, and other distros with glibc 2.39+ | `Yapfast-…-x86_64.AppImage` | `chmod +x` it (or "Allow executing" in its properties) and run it. It adds itself to your app menu. |
| Arch Linux and its derivatives | `yapfast-…-x86_64.pkg.tar.zst` | `sudo pacman -U yapfast-…-x86_64.pkg.tar.zst` |
| Windows 10 and 11 (beta) | `Yapfast-…-windows-x64.zip` | Unzip it anywhere and run `yapfast.exe`. It isn't signed, so SmartScreen may ask: **More info → Run anyway**. |

Older distros, such as Debian 12 or Ubuntu 22.04, need to
[build it themselves](#build-it-from-source). There's no macOS version.

Then link your Discord account by scanning the QR code with Discord's phone
app (**Settings → Scan QR Code**). Read
[Will Discord ban me?](#will-discord-ban-me) first.

Check a download against the release's checksums:

```sh
sha256sum -c SHA256SUMS.txt --ignore-missing
```

### Versions

A version is the upstream release it's based on, then `-dl.` and this fork's
release number: `0.3.0-dl.1` is upstream 0.3.0 with this fork's first
release of changes. A new upstream release becomes `…-dl.1` again.

The releases are built from the tagged commit with the same steps as
[`release.yml`](.github/workflows/release.yml): Nix's pinned toolchain, and
upstream's packaging scripts, which refuse a program that names the build
machine's home folder.

## Build it from source

Measured building `0.3.0-dl.1` on a 12-core laptop with 16 GB of RAM:

| Build | Free disk space | Time, first build |
| --- | --- | --- |
| Linux, with Nix (AppImage) | about 15 GB | about 20 minutes, downloads included |
| Windows, with Nix, on top of that | about 8 GB more | about 25 minutes more |

Without Nix, plan on about 15 GB as well. Building needs at least 8 GB of RAM,
since one part of the interface takes 7 to 8 GB on its own; 16 GB is better.
On a machine with fewer cores it takes longer. Most of the space is temporary
build files: once Yapfast is built, `cargo clean` (or deleting `target/`)
gives it back.

### Linux, with Nix (the way the releases are built)

Install [Nix](https://nixos.org/download), then:

```sh
git clone https://github.com/HolyJoey/yapfast.git
cd yapfast
# The Linux AppImage, into target/dist/linux
scripts/build-appimage
# Or just run it
nix develop path:./nix --command cargo run --locked --release --no-default-features --features desktop,discord
```

### Linux, without Nix

Install [rustup](https://rustup.rs); it fetches Rust 1.97.1 from
`rust-toolchain.toml` itself. Then the build tools and libraries:

```sh
# Debian and Ubuntu
sudo apt install build-essential clang cmake git curl python3 nasm protobuf-compiler \
  pkg-config libfontconfig-dev libglib2.0-dev libpulse-dev libwayland-dev \
  libxkbcommon-dev libxkbcommon-x11-dev libx11-dev libxcb1-dev libxcursor-dev \
  libxi-dev libgl-dev libegl-dev libasound2-dev
# Fedora
sudo dnf install gcc gcc-c++ clang cmake git curl python3 nasm protobuf-compiler \
  pkgconf-pkg-config fontconfig-devel glib2-devel pulseaudio-libs-devel wayland-devel \
  libxkbcommon-devel libxkbcommon-x11-devel libX11-devel libxcb-devel \
  libXcursor-devel libXi-devel libglvnd-devel alsa-lib-devel
# Arch
sudo pacman -S --needed base-devel clang cmake git curl python nasm protobuf pkgconf \
  fontconfig glib2 libpulse wayland libxkbcommon libxkbcommon-x11 libx11 libxcb \
  libxcursor libxi libglvnd alsa-lib
```

These follow the dependencies of upstream's [Arch package](packaging/arch/PKGBUILD).
This fork is developed on Debian 13; the Fedora list is untested, so a missing
`-devel` package shows up as a build error naming it.

Then build and install it:

```sh
git clone https://github.com/HolyJoey/yapfast.git
cd yapfast
export CC=clang CXX=clang++ CFLAGS="-ffile-prefix-map=$HOME=/build" CXXFLAGS="-ffile-prefix-map=$HOME=/build"
cargo build --locked --release --no-default-features --features desktop,discord \
  --config "build.rustflags=['--remap-path-prefix=$HOME=/build']"
install -Dm755 target/release/yapfast ~/.local/bin/yapfast
```

Cargo downloads the pinned Skia and libwebrtc archives itself. The
path flags keep your home folder's path, and so your user name, out of the
program, in case you share it: check with `strings target/release/yapfast | grep "$HOME"`,
which should print nothing. Yapfast adds itself to your app menu
the first time it runs. [docs/building.md](docs/building.md) has the details.

### Windows

Windows builds are cross-compiled from Linux, with Nix:

```sh
nix develop path:./nix#windows --command scripts/build-windows discord --no-installer
```

The ZIP lands in `target/dist/discord`. The installer needs Inno Setup,
which doesn't run under Wine, so cross-built releases come as the ZIP only
(add `--no-installer`). Building on Windows
itself is possible but only meant for development; see
[docs/building.md](docs/building.md#windows).

### Updating to a new upstream version

This fork keeps its changes on the `discord-look` branch, on top of
upstream's release tag. `main` follows upstream's `main`.

```sh
git remote add upstream https://github.com/yapfast/yapfast.git
git fetch upstream --tags
git rebase --onto v0.4.0 v0.3.0 discord-look
```

## Not there yet

- Media channels and Activities.
- Making and changing server folders.
- A server's form for new members, such as its rules, when you join: Yapfast
  sends you to Discord in your browser to fill it in.
- Animated custom emoji in messages, which show their first frame.
- Discord's newer profile extras: the Board tab, Wishlist and profile-wide
  themes.
- macOS, and a signed Windows build with an installer.

Use Discord in a browser for these until then.

## Will Discord ban me?

> [!WARNING]
> Probably not, but nobody can promise it. Third-party clients, this one
> included, are against Discord's Terms of Service. In practice Discord rarely
> bans anyone for using one; the accounts that get banned are almost always
> spamming or running automation. Yapfast only does what you click, and keeps
> to the same limits Discord's own app does.
>
> If losing your account would be a disaster, don't use any third-party
> client, this one or another. And don't post screenshots of it in a server
> whose moderators might ban you for it. Yapfast tells you all this again
> before it shows the sign-in code.

## Where your messages go

Straight from your PC to Discord. Neither Yapfast nor this fork runs a server
in between that could keep a copy.

- You link your account by scanning a QR code with Discord's phone app, so
  Yapfast never sees your password.
- The login token stays in your system's credential store, and only ever goes
  to Discord. Unlinking signs that session out and deletes it.
- It sends no analytics or crash reports, and doesn't check for updates.
- Pictures and GIFs load from wherever they're hosted, such as Discord or
  Tenor; gg sans, two name style fonts and Discord's sounds come from
  `discord.com`, as [above](#about-gg-sans).

The code's all here to check.

## More documentation

| Read this | For |
| --- | --- |
| [Building](docs/building.md) | Toolchain, Cargo features, checks, releases |
| [Architecture](docs/architecture.md) | How `yapfast-core` and the Slint client fit together |
| [Discord](docs/discord/README.md) | How each Discord feature works, and the fake server the tests use |
| [Voice](docs/voice.md) | Calls, devices and audio processing |
| [Windows](docs/windows-beta.md) | Packaging and Windows checks |
| [Measurements](docs/measurements/README.md) | How memory and CPU are measured |

## License

Yapfast is free software under the [GNU General Public License](LICENSE),
version 3 or later, and so is this fork. Every release carries its
dependencies' own licenses.

## Thanks

To [Yapfast](https://github.com/yapfast/yapfast) and its developers, whose app
this is; upstream's README says how it was made, AI help included. Yapfast
speaks to Discord the way its own apps do, as documented by
[Discord Userdoccers](https://docs.discord.food). It stands on
[Slint](https://slint.dev), [Skia](https://skia.org),
[LiveKit](https://livekit.io) and WebRTC, and
[DeepFilterNet](https://github.com/Rikorose/DeepFilterNet).

<sub>Neither Yapfast nor this fork is affiliated with or endorsed by Discord
Inc. Discord is a trademark of Discord Inc. gg sans is Discord's typeface and
isn't distributed here.</sub>
