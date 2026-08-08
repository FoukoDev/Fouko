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

- One bot, Telegram and Discord at once, from a single codebase.
- Account linking: a 6-character code ties identities together, an inline-button picker locks in the primary once. Reversible only via Unlink.
- Profile, XP, coins - progression carries across linked platforms, with `/daily` streaks, achievements, a leaderboard and a small shop.
- **Bring-your-own AI.** `/ai` plugs into Ollama, LiteLLM, LM Studio or OpenRouter - models are discovered automatically, everything is stored encrypted, and family sharing lends a host without exposing your keys. It draws, renders short videos, speaks - and the model can call generation itself, so "create a cat picture" in chat just works. Send a photo and a vision model answers, streaming the reply live.
- **Telegram Mini App.** The whole `/ai` in a web interface - streaming chats with a stop button, model switching, hosts and settings, signed in through Telegram only. Set `WEBAPP_URL`, or let the bot get a free tunnel URL itself with `WEBAPP_TUNNEL=cloudflared` (no domain needed).
- Five languages: English, Russian, Ukrainian, German, Spanish. Every command answers in all five, switched instantly with `/lang`
- Owner alerts: set `OWNER_TG_ID` / `OWNER_DISCORD_ID` and the bot DMs you about crash restarts and update floods.
- Nice to look at - real embeds on Discord, HTML rendering on Telegram, a colorful startup banner in the terminal.

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
| `/cat`                    | Random cat picture                                                  |
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
| `/profile [@user]`        | Level, XP, coins, streak, title, badges, linked platforms           |
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
| `/ai draw <prompt>`       | Generate an image with an image model on your hosts                |
| `/ai video <prompt>`      | Render a short video with a Sora-style model                       |
| `/ai speak <text>`        | Text to speech with a TTS model                                     |
| `/ai models`              | List your models tagged with what they can do                       |
| `/ai gen`                 | Pin a model per generation type (image/video/audio), or `auto`      |

Your own `/profile` card also shows an Account section with your user id on the current platform - handy when filling in `OWNER_*` or linking accounts.

## Account linking flow

1. On platform A: `/link` -> bot replies with a **6-char code**, valid 5 minutes.
2. On platform B: `/link CODE` -> both sides are linked, bot shows **two buttons** to pick the primary. *This choice is one-shot.*
3. From now on profile / XP / coins / language follow the primary.
4. Any side can run `/link` -> **Unlink**. The primary keeps its profile, the other side starts with a fresh one.

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

# Storage - leave empty and a SQLite file is created next to the .env
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

### Running in Docker

The repo ships a `Dockerfile` and `docker-compose.yml` that build the bot
from source. Put your `.env` next to them, prepare a data directory and go:

```bash
mkdir -p data && sudo chown 10001 data   # container runs as uid 10001
docker compose up -d --build
docker compose logs -f                   # follow the logs
```

The SQLite database and rotating logs live in `./data` on the host; the
container pins `FOUKO_DB` and `FOUKO_LOG_DIR` there, everything else comes
from your `.env`.

### Enabling the Mini App

Set `WEBAPP_URL` to the public https URL of the app and put a reverse proxy
in front of the built-in server (it listens on `WEBAPP_BIND`, default
`127.0.0.1:8990`). Caddy example:

```
ai.example.com {
    reverse_proxy 127.0.0.1:8990
}
```

The bot publishes the app as the menu button next to the message box in
private chats automatically. To also get the "Open app" button on the
bot's profile, set the same URL once in [@BotFather](https://t.me/BotFather):
`/mybots` -> your bot -> `Bot Settings` -> `Configure Mini App` (Telegram
offers no API for that part, it's a one-time manual step).

#### No domain? Use a tunnel

Leave `WEBAPP_URL` empty and set `WEBAPP_TUNNEL=cloudflared` instead. On
start the bot runs a free [Cloudflare quick tunnel](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/do-more-with-tunnels/trycloudflare/)
and picks up its `https://<random>.trycloudflare.com` URL automatically -
no domain, no reverse proxy, no account. The `cloudflared` binary must be
installed (`pacman -S cloudflared` / `apt install cloudflared` /
`brew install cloudflared`).

Caveats: the URL is random and changes on every restart, so Mini App
buttons in old messages stop working (the menu button is re-published on
start and stays fresh). Quick tunnels have no uptime guarantee - fine for
trying things out, not for production. For a permanent setup, get a domain
and set `WEBAPP_URL`.

## Configuration in one table

| Variable               | Required? | What it is                                                          |
|------------------------|-----------|---------------------------------------------------------------------|
| `TG_TOKEN`             | One of    | Telegram bot token ([@BotFather](https://t.me/BotFather))           |
| `DISCORD_TOKEN`        | One of    | Discord bot token (Developer Portal -> Bot -> Reset Token)            |
| `FOUKO_DB`             | No        | Storage URL. `sqlite:/path`, `memory:`, or your own scheme          |
| `FOUKO_SECRET`         | No        | Passphrase encrypting the `/ai` feature's data; unset disables `/ai` |
| `WEBAPP_URL`           | No        | Public https URL of the Telegram Mini App (behind a reverse proxy); unset disables it |
| `WEBAPP_TUNNEL`        | No        | `cloudflared` - get a free quick-tunnel URL automatically when `WEBAPP_URL` is empty |
| `WEBAPP_BIND`          | No        | Address the Mini App server listens on (default `127.0.0.1:8990`)   |
| `OWNER_TG_ID`          | No        | Your Telegram user id for owner alerts (see `/profile` -> Account)   |
| `OWNER_DISCORD_ID`     | No        | Your Discord user id for owner alerts                               |
| `OWNER_ALERT_PLATFORM` | No        | Preferred alert platform: `telegram` or `discord` (the other is the fallback) |
| `FLOOD_ALERT_PER_MIN`  | No        | Updates per minute that trigger a flood alert (default `300`)       |
| `PRESENCE_KIND`        | No        | Discord activity type: `playing`, `streaming`, `listening`, `watching`, `competing` or `custom` |
| `PRESENCE_TEXT`        | No        | The activity text; empty = no presence (quote values with spaces)   |
| `PRESENCE_URL`         | No        | Stream link, `streaming` only (twitch/youtube light up the purple dot) |
| `PRESENCE_STATE`       | No        | Smaller detail line under the activity name on the profile card     |
| `PRESENCE_STATUS`      | No        | The online dot: `online`, `idle`, `dnd` or `invisible`              |
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
