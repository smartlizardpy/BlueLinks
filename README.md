<p align="center">
  <img src="docs/assets/bluelink-logo.png" width="116" alt="BlueLink logo" />
</p>

<h1 align="center">BlueLink</h1>

<p align="center"><strong>A Windows-first Wikipedia speedrun game.</strong><br />Start on one article, reach the target using only links inside Wikipedia, and race the clock—or your click count.</p>

[![Windows build](https://github.com/smartlizardpy/BlueLinks/actions/workflows/windows.yml/badge.svg)](https://github.com/smartlizardpy/BlueLinks/actions/workflows/windows.yml)
[![Latest release](https://img.shields.io/github/v/release/smartlizardpy/BlueLinks?display_name=tag&label=release)](https://github.com/smartlizardpy/BlueLinks/releases/latest)

[Download BlueLink for Windows](https://github.com/smartlizardpy/BlueLinks/releases/latest) · [Visit the website](https://smartlizardpy.github.io/BlueLinks/) · [Report an issue](https://github.com/smartlizardpy/BlueLinks/issues)

## The challenge

```text
START
Minecraft
    ↓
TARGET
Sweden
```

BlueLink generates a start and target locally, then opens the real English Wikipedia inside a restricted game window. The timer stops when you reach the target. One completed article transition counts as one click, including redirects, so impatient double-clicking does not inflate the score.

## Game modes

| Mode | Goal |
| --- | --- |
| **Normal** | Reach the target at your own pace. |
| **Max Clicks** | Finish without exceeding the click budget. |
| **Time Limit** | Reach the target before the countdown expires. |
| **Fewest Clicks** | Find the shortest route you can. |
| **Speedrun** | Optimize for the fastest time. |
| **Evil** | Take on deliberately harder article pairings. |
| **2 Player** | Pass the PC and compete on the same route. |
| **Gauntlet** | Clear five targets on one continuous timer. |

Runs use a monotonic, LiveSplit-style timer. BlueLink also keeps exact-pair personal bests, streaks, settings, and the latest 100 runs locally on your PC. The challenge selector rejects redirects, disambiguation pages, dead ends, near-duplicate titles, and obvious direct neighbors to produce hard-but-fair routes.

## Install on Windows

1. Open the [latest GitHub release](https://github.com/smartlizardpy/BlueLinks/releases/latest).
2. Download the Windows setup `.exe` from **Assets**.
3. Run the installer, then launch **BlueLink** from the Start menu.

Until the first signed `v1.0.0` tag lands, the latest release is an unsigned build. It is the complete game with the full article database; it simply cannot update itself, and Windows will warn before running it.

BlueLink is built for 64-bit Windows. The installed game does not require Node.js, Rust, Python, or a BlueLink server. If Windows displays a reputation warning for a new release, confirm that the installer came from this repository's Releases page before proceeding.

## How it works

The trusted React/Tauri shell renders BlueLink's controls. During a run, Rust opens English Wikipedia in a separate child WebView below the game bar. That remote page has no access to BlueLink's privileged command bridge.

Rust owns challenge selection, allowed-URL checks, navigation transactions, route recording, timing, target detection, and local persistence. Only English Wikipedia main-namespace article navigation is accepted. See [Wikimedia attribution](data/ATTRIBUTION.md) for data-source details.

> [!IMPORTANT]
> Wikipedia pages and links are live third-party content and require an internet connection. BlueLink restricts navigation, but it does not control Wikipedia's availability or page content. Challenge data, settings, history, and personal bests remain on your computer.

## Build from source

You need Node.js 20+, stable Rust, the [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/), and Python 3 only if you rebuild the article dataset.

```bash
npm install
python tools/build_dataset.py --development --output data/articles.sqlite
npm run tauri dev
```

Run the frontend tests, production web build, and Rust tests together:

```bash
npm run check
```

The included database is deliberately development-sized, and that is all you need to work on BlueLink. A production package requires a validated English Wikipedia dataset and refuses to fall back to development data, but GitHub Actions builds that dataset during a release—no contributor has to download a Wikipedia dump. See [the release guide](docs/RELEASING.md).

## Releases and updates

Tagged `vX.Y.Z` releases are tested and packaged for Windows by GitHub Actions. Published builds use Tauri's signed updater metadata, allowing an existing installation to check for and install a newer release. Local unsigned development builds intentionally leave update checking unavailable.

Maintainers: follow [docs/RELEASING.md](docs/RELEASING.md) before creating a release tag. Signing secrets must stay in GitHub Actions and must never be committed.

## Contributing

Bug reports and focused pull requests are welcome. Before opening a PR:

```bash
npm ci
npm run check
```

Keep changes scoped, explain any gameplay behavior change, and include a Windows test note when touching the WebView, installer, or updater. Please do not commit private signing keys, Wikipedia dump archives, generated build folders, or personal run data.
