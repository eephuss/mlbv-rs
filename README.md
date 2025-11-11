# mlbv-rs

A rewrite of [kmac/mlbv](https://github.com/kmac/mlbv-archived) in Rust.

The original tool was great, and I wanted to learn Rust. This seemed like a good opportunity to kill two birds with one stone. This initial release implements only essential features, allowing users to stream live and archived games and check the schedule. I plan to re-add features from the original mlbv in subsequent releases.

## Purpose

This project is a command-line interface to MLB.tv and MLB's stats API.

**Paid Features** (requires MLB.tv subscription):
- Stream live and archived games

**Free Features** (no account required):
- View game schedules, status, and results
- Stream highlights and recaps

## Current Features

- Stream live or archived MLB games (requires MLB.tv subscription)
- Display game schedules for a given day or range of days

## Roadmap

- View highlights and recaps
- Favorites and colorization
- Complete line and box scores
- Record streams
- Display standings
- Display stats
- Filter relevant displays

## Attribution

- Original project by [kmac](https://github.com/kmac/mlbv-archived)
- API session authentication research from [mlbstreamer](https://github.com/tonycpsu/mlbstreamer) and [Kodi MLB.tv plugin](https://github.com/eracknaphobia/plugin.video.mlbtv)

## License

[GPL-3.0](LICENSE)
