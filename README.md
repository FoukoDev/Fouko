<div align="center">

<img src="https://fouko.xyz/assets/brand/logoViolet.png" alt="FoukoBot" width="96" height="96">

# FoukoBot

**The reference bot for [FoukoApi](https://github.com/FoukoDev/FoukoApi).**
One codebase, many chat platforms, same set of fun and handy commands.

<a href="https://bot.fouko.xyz"><img alt="bot.fouko.xyz" src="https://img.shields.io/badge/site-bot.fouko.xyz-8b5cf6?style=for-the-badge&labelColor=0a0a0f"></a>
<a href="https://github.com/FoukoDev/FoukoApi"><img alt="FoukoApi" src="https://img.shields.io/badge/built_on-FoukoApi-fbbf24?style=for-the-badge&labelColor=0a0a0f"></a>
<a href="https://github.com/FoukoDev/Fouko/actions"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/FoukoDev/Fouko/ci.yml?style=for-the-badge&labelColor=0a0a0f"></a>
<a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-6366f1?style=for-the-badge&labelColor=0a0a0f"></a>

<a href="https://bot.fouko.xyz">bot.fouko.xyz</a> · <a href="https://fouko.xyz">fouko.xyz</a> · <a href="https://discord.gg/rx9nXt735R">Discord</a> · <a href="https://t.me/foukoo">Telegram</a>

</div>

---

## What is it

FoukoBot is a reference bot built on top of [FoukoApi](https://github.com/FoukoDev/FoukoApi). One Rust codebase - **Telegram and Discord at the same time**. Profile, XP, coins, linked accounts and language settings are shared across every platform the user connects.

Source is open and doubles as a living example of how to build on FoukoApi.

## Highlights

- 🧩 **One bot, multiple platforms.** Telegram + Discord out of the box.
- 🔗 **Account linking** with a 6-character code and an inline-button primary picker. One-time choice, reversible only via Unlink.
- 🏅 **Profile, XP, coins.** Per-user progression carries across linked platforms, with a `/daily` streak, a leaderboard and a small shop.
- 🌐 **Per-user language** (`/lang`) with inline keyboard - English, Russian, Ukrainian, German and Spanish. Full localisation: every command answers in all five, switched instantly.
- 🤖 **Bring-your-own AI** (`/ai`): plug in Ollama, LiteLLM, LM Studio or OpenRouter - models are discovered automatically, everything is stored encrypted, and family sharing lets you lend a host without exposing your keys.
- 🚨 **Owner alerts**: set `OWNER_TG_ID` / `OWNER_DISCORD_ID` and the bot DMs you about crash restarts and update floods.
- 🎛 **Colourful startup banner** with a summary of platforms, storage, AI, owner config and log paths.
- 🎨 **Pretty embeds** everywhere (real Discord embed, HTML on Telegram).
- 🛠 **30+ commands**: see the full table below.

## Commands

| Command                   | What it does                                                        |
|---------------------------|---------------------------------------------------------------------|
| `/start`, `/help`         | Nudge toward the full command list                                  |
| `/info`                   | About the bot: stack, features, version, uptime                     |
| `/server`                 | Info about the current server or chat                               |
| `/avatar [@user]`         | Show an avatar and banner (Discord only - Telegram has no shareable avatar URL) |
| `/ping`                   | Health check                                                        |
| `/time` · `/time +10`     | Current time with optional UTC offset                               |
| `/coin`                   | Flip a coin                                                         |
| `/roll NdM`               | Roll dice, e.g. `/roll 3d6`                                         |
| `/8ball <question>`       | Magic 8-ball                                                        |
| `/rps`                    | Rock paper scissors on buttons                                      |
| `/cat`                    | Random cat picture (Telegram / Discord)                             |
| `/joke`                   | A joke - pick a theme with buttons                                  |
| `/choose a, b, c`         | Pick one at random                                                  |
| `/reverse <text>`         | Reverse text                                                        |
| `/emoji`                  | Random cute emoji                                                   |
| `/calc <expr>`            | Evaluate an arithmetic expression                                   |
| `/qr <text>`              | Generate a QR code                                                  |
| `/shorten <url>`          | URL shortener                                                       |
| `/poll Q? \| A \| B`      | Start a poll with vote buttons                                      |
| `/remind 10m <text>`      | Set a reminder (bare `/remind` lists pending ones)                  |
| `/menu`                   | Interactive menu with inline buttons                                |
| `/profile [@user]`        | Level, XP, coins, streak, title, badges, linked platforms; your own card has an Account section with your user id (handy for `/link` and `OWNER_*` setup) |
| `/daily`                  | Daily reward with a streak bonus                                    |
| `/leaderboard [coins]`    | Top players by XP or coins                                          |
| `/rank [@user]`           | Position on the leaderboard                                         |
| `/achievements [@user]`   | All badges, earned and not yet                                      |
| `/shop` · `/buy <id>`     | Spend coins on titles and profile colours                           |
| `/give <id> <amount>`     | Send coins to another user                                          |
| `/gamble <amount>`        | Bet coins on a coin flip                                            |
| `/link` · `/link CODE`    | Cross-platform account linking (with unlink + one-shot primary)     |
| `/lang`                   | Switch language (inline picker)                                     |
| `/settings`               | Personal settings hub with buttons (DM only)                        |
| `/weather <city>`         | Current weather for a city (no key needed)                          |
| `/ai`                     | Your private AI: add a host, pick a model, chat (needs `FOUKO_SECRET`) |

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
# Platforms - set at least one
TG_TOKEN=123456:AA...
DISCORD_TOKEN=your-discord-bot-token

# Storage - leave empty and a SQLite file is created next to the binary
# FOUKO_DB=sqlite:./foukobot.sqlite

```

Any platform whose token is empty is silently skipped. See [`.env.example`](.env.example) for the full list with comments.

### Logging

The bot logs to stdout **and** to a rotating file (a new one each day) under
`logs/`. Point it elsewhere with `FOUKO_LOG_DIR`. Verbosity follows the
standard `tracing` env filter (default `info,serenity=warn,tracing=warn`):

```bash
RUST_LOG=debug cargo run --release
```

### Running as a service

The bot supervises itself - if a connection drops it reconnects with a
backoff, and it stops cleanly on `Ctrl+C` or `SIGTERM`. That makes it a good
fit for a systemd unit that also restarts it across reboots and crashes:

```ini
# /etc/systemd/system/foukobot.service
[Unit]
Description=FoukoBot
After=network-online.target
Wants=network-online.target

[Service]
WorkingDirectory=/opt/foukobot
ExecStart=/opt/foukobot/foukobot
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable --now foukobot
journalctl -u foukobot -f   # follow the logs
```

## Configuration in one table

| Variable               | Required? | What it is                                                          |
|------------------------|-----------|---------------------------------------------------------------------|
| `TG_TOKEN`             | One of    | Telegram bot token ([@BotFather](https://t.me/BotFather))           |
| `DISCORD_TOKEN`        | One of    | Discord bot token (Developer Portal → Bot → Reset Token)            |
| `FOUKO_DB`             | No        | Storage URL. `sqlite:/path`, `memory:`, or your own scheme          |
| `FOUKO_SECRET`         | No        | Passphrase encrypting the `/ai` feature's data; unset disables `/ai` |
| `OWNER_TG_ID`          | No        | Your Telegram user id for owner alerts (see `/profile` → Account)   |
| `OWNER_DISCORD_ID`     | No        | Your Discord user id for owner alerts                               |
| `OWNER_ALERT_PLATFORM` | No        | Preferred alert platform: `telegram` or `discord` (the other is the fallback) |
| `FLOOD_ALERT_PER_MIN`  | No        | Updates per minute that trigger a flood alert (default `300`)       |
| `RUST_LOG`             | No        | Standard `tracing-subscriber` filter                                 |
| `FOUKO_LOG_DIR`        | No        | Directory for rotating log files (default `logs`)                    |

## Tech stack

- **Rust** (latest stable), async via `tokio`
- [`FoukoApi`](https://github.com/FoukoDev/FoukoApi) - command router, embeds, keyboards, platform adapters, account linking, storage
- `teloxide` for Telegram, `serenity` for Discord
- `rusqlite` (bundled) for SQLite storage
- `reqwest` (rustls) for the `/weather`, `/qr`, `/shorten` commands
- `dotenvy`, `tracing`, `chrono`, `rand`

## Contributing

Issues and PRs welcome. Run `cargo fmt && cargo clippy --all-targets` before opening a PR.

## License

MIT - see [LICENSE](LICENSE).

---

<sub>Part of the <a href="https://fouko.xyz">Fouko</a> family.</sub>
