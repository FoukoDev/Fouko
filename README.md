<div align="center">

<img src="https://fouko.xyz/assets/brand/logoViolet.png" alt="FoukoBot" width="96" height="96">

# FoukoBot

**The reference bot for [FoukoApi](https://github.com/FoukoDev/FoukoApi).**
One codebase, many chat platforms, same set of fun and handy commands.

<a href="https://bot.fouko.xyz"><img alt="bot.fouko.xyz" src="https://img.shields.io/badge/site-bot.fouko.xyz-8b5cf6?style=for-the-badge&labelColor=0a0a0f"></a>
<a href="https://github.com/FoukoDev/FoukoApi"><img alt="FoukoApi" src="https://img.shields.io/badge/built_on-FoukoApi-fbbf24?style=for-the-badge&labelColor=0a0a0f"></a>
<a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-6366f1?style=for-the-badge&labelColor=0a0a0f"></a>

<a href="https://bot.fouko.xyz">bot.fouko.xyz</a> · <a href="https://fouko.xyz">fouko.xyz</a> · <a href="https://discord.gg/rx9nXt735R">Discord</a> · <a href="https://t.me/foukoo">Telegram</a>

</div>

---

## What is it

FoukoBot is a reference bot built on top of [FoukoApi](https://github.com/FoukoDev/FoukoApi). One Rust codebase — **Telegram and Discord at the same time**. Profile, XP, coins, linked accounts and language settings are shared across every platform the user connects.

Source is open and doubles as a living example of how to build on FoukoApi.

## Highlights

- 🧩 **One bot, multiple platforms.** Telegram + Discord out of the box.
- 🔗 **Account linking** with a 6-character code and an inline-button primary picker. One-time choice, reversible only via Unlink.
- 🏅 **Profile, XP, coins.** Per-user progression carries across linked platforms.
- 🌐 **Per-user language** (`/lang`, EN/RU) with inline keyboard — takes effect instantly.
- 🎨 **Pretty embeds** everywhere (real Discord embed, HTML on Telegram).
- 🛠 **20+ useful commands**: see the full table below.

## Commands

| Command                   | What it does                                                        |
|---------------------------|---------------------------------------------------------------------|
| `/start`, `/help`         | Nudge toward the full command list                                  |
| `/bot`                    | Detailed bot info card: stack, features, links                      |
| `/about`                  | Short bot info card                                                 |
| `/ping`                   | Health check                                                        |
| `/stats`                  | Uptime + build version + current platform                           |
| `/time` · `/time +10`     | Current time with optional UTC offset                               |
| `/coin`                   | Flip a coin (Heads / Tails, RU/EN labels)                           |
| `/roll NdM`               | Roll dice, e.g. `/roll 3d6`                                         |
| `/8ball <question>`       | Magic 8-ball                                                        |
| `/cat`                    | Random cat picture (Telegram / Discord)                             |
| `/joke`                   | A random programmer joke                                            |
| `/choose a, b, c`         | Pick one at random                                                  |
| `/reverse <text>`         | Reverse text                                                        |
| `/emoji`                  | Random cute emoji                                                   |
| `/qr <text>`              | Generate a QR code                                                  |
| `/shorten <url>`          | URL shortener (is.gd)                                               |
| `/menu`                   | Interactive menu with inline buttons                                |
| `/profile`                | Level, XP, coins, language, linked platforms                        |
| `/link` · `/link CODE`    | Cross-platform account linking (with unlink + one-shot primary)     |
| `/lang`                   | Switch language (inline picker)                                     |
| `/settings`               | Personal settings (DM only, read-only view)                         |
| `/weather <city>`         | Weather via OpenWeatherMap (needs `OPEN_WEATHER_KEY`)               |

## Account linking flow

1. On platform A: `/link` → bot replies with a **6-char code**, valid 5 minutes.
2. On platform B: `/link CODE` → both sides are linked, bot shows **two buttons** to pick the primary. *This choice is one-shot.*
3. From now on profile / XP / coins / language follow the primary.
4. Any side can run `/link` → **Unlink**. The primary keeps its profile, the other side starts with a fresh one.

## Running it yourself

Requires the latest stable Rust toolchain.

```bash
git clone https://github.com/FoukoDev/Fouko
cd Fouko
cargo run --release   # on first run writes a .env template, then exits
```

Fill in the freshly created `.env`, then `cargo run --release` again.

### Minimal `.env`

```dotenv
# Platforms — set at least one
TG_TOKEN=123456:AA...
DISCORD_TOKEN=your-discord-bot-token

# Storage — leave empty and a SQLite file is created next to the binary
# FOUKO_DB=sqlite:./foukobot.sqlite

# Optional: weather command
OPEN_WEATHER_KEY=
```

Any platform whose token is empty is silently skipped. See [`.env.example`](.env.example) for the full list with comments.

### Logging

The bot uses `tracing` with an env filter. Default is `info,foukoapi=info,foukobot=debug`. Override with `RUST_LOG`, for example:

```bash
RUST_LOG=debug cargo run --release
```

## Configuration in one table

| Variable           | Required? | What it is                                                          |
|--------------------|-----------|---------------------------------------------------------------------|
| `TG_TOKEN`         | One of    | Telegram bot token ([@BotFather](https://t.me/BotFather))           |
| `DISCORD_TOKEN`    | One of    | Discord bot token (Developer Portal → Bot → Reset Token)            |
| `FOUKO_DB`         | No        | Storage URL. `sqlite:/path`, `memory:`, or your own scheme          |
| `OPEN_WEATHER_KEY` | No        | OpenWeatherMap API key for `/weather`                                |
| `RUST_LOG`         | No        | Standard `tracing-subscriber` filter                                 |

## Tech stack

- **Rust** (latest stable), async via `tokio`
- [`FoukoApi`](https://github.com/FoukoDev/FoukoApi) — command router, embeds, keyboards, platform adapters, account linking, storage
- `teloxide` for Telegram, `serenity` for Discord
- `rusqlite` (bundled) for SQLite storage
- `reqwest` (rustls) for the `/weather`, `/qr`, `/shorten` commands
- `dotenvy`, `tracing`, `chrono`, `rand`

## Contributing

Issues and PRs welcome. Run `cargo fmt && cargo clippy --all-targets` before opening a PR.

## License

MIT — see [LICENSE](LICENSE).

---

<sub>Part of the <a href="https://fouko.xyz">Fouko</a> family.</sub>
