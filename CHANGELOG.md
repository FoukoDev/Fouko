# Changelog

All notable changes to FoukoBot are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1-alpha.1] - 2026-08-02

### Added
- **Economy & profile.** Earn XP by chatting, level up, and collect coins.
  `/profile` shows your level, XP, coins, streak, title and badges.
- **Achievements.** Earn badges (first daily, week streak, high roller,
  big spender) that show up on your profile.
- `/daily` - claim a daily reward with a growing streak and a weekly bonus.
- `/leaderboard` - see the top players by XP, or `/leaderboard coins` by balance.
- `/rank` - check your own position on both boards.
- `/achievements` - browse every badge, including the ones you haven't
  earned yet, yours or anyone else's.
- `/shop` and `/buy` - spend coins on titles and profile colours.
- `/give` - send coins to another user.
- `/gamble` - bet coins on a coin flip.
- `/rps` - play rock paper scissors on buttons.
- `/poll` - start a poll with live vote buttons.
- `/remind` - set a reminder, e.g. `/remind 10m stretch`. Reminders
  survive a restart, arrive in your DM (falling back to the original chat
  with a mention), and bare `/remind` opens a list of pending ones with
  delete buttons.
- `/calc` - evaluate an arithmetic expression.
- `/info` - one card with the bot's stack, features, version and uptime.
- `/server` - info about the current server or chat.
- `/avatar` - show your avatar (Discord).
- `/ai` - your own private AI. Add a host (Ollama, LiteLLM, LM Studio,
  OpenRouter, ...) and its models are picked up automatically; name a
  chat, set its system prompt and talk to it. Everything (hosts, keys,
  prompts, history) is stored encrypted and follows your `/link` across
  platforms.
- Family access for `/ai`: share a host and chosen models with another
  user by invite. They can use it (even on another platform via `/link`)
  but never see your keys, prompts or history - and you can change or
  revoke their access at any time.
- `/profile` now has an Account section: your user id on the current
  platform (handy for `/link` and owner setup) and your display name.
  Only shown on your own profile - other people's cards never expose ids.
- Owner alerts: set `OWNER_TG_ID` / `OWNER_DISCORD_ID` in `.env` and the
  bot DMs you when it restarts after a crash (with the time it went
  down), when it hits repeated errors, or when it sees a suspicious
  flood of updates (`FLOOD_ALERT_PER_MIN`, default 300). Pick the
  preferred platform with `OWNER_ALERT_PLATFORM`; the other one is the
  fallback. Alerts are throttled so a flaky network doesn't spam you.
- Owner verification at startup: the bot resolves the configured
  `OWNER_*` ids to display names and logs them, so you can see whose id
  you pasted. On first setup (or when the ids change) the owner also
  gets a one-time "alerts armed" DM confirming everything is wired up.
- Colorful startup banner: ASCII-art logo plus a summary of platforms,
  storage, AI, owner config, flood threshold and log paths, followed by a
  green "online" line once the adapters connect and a checkmark when the
  owner ids are verified. Respects `NO_COLOR` and non-terminal output.
- More languages: `/lang` now offers English, Russian, Ukrainian, German
  and Spanish - and every command answers in all five, from the profile
  and shop to weather, polls, the menu and reminders. Reminders and AI
  family-access invites arrive in the recipient's language, not the
  sender's.

### Changed
- Commands work everywhere now, including Discord servers (not just DMs).
- `/daily` shows the coins that were actually minted, so the number on
  the card always matches your balance.
- `/gamble` and `/give` tell you when the amount doesn't parse (negative,
  garbage, too large) instead of showing generic usage text.
- `/poll` rejects more than 5 options with a clear message instead of
  silently dropping the extras.
- The menu's joke button is rate-limited like the `/joke` command.
- `/weather` no longer needs an API key - it just works out of the box,
  and it now shows the city's local time too.
- `/menu` grew more buttons: 8-ball, joke, emoji and rock paper scissors.
- `/joke` pulls fresh jokes from an online service now, follows your
  language where it can (Russian included), and lets you pick a theme
  with buttons.
- `/profile`, `/rank`, `/achievements` and `/avatar` accept a user
  argument, so you can look at someone else's card (avatar included); on
  Discord you pick the person from the native member list.
- Replying to one of the bot's messages talks to your AI chat, even in a
  group.
- `/settings` is button-driven now - switch language with a tap.
- `/help` is now a tidy embed with commands grouped into sections
  (General, Economy, Fun, Tools, Account).
- `/shop` and the `/ai` menu are now button-driven (typed commands still
  work as a fallback).
- `/ai` shows a typing indicator while the model thinks, rate-limits
  requests, and splits long answers across messages instead of clipping
  them. Adding a host with a key now reminds you to delete your message so
  the key doesn't linger in chat history.
- The bot keeps itself running: it restarts automatically if it loses its
  connection, stops cleanly on shutdown, and won't reply to a backlog of
  old messages after being offline - it just answers what's current.
- Logs are written to a rotating daily file under `logs/` as well as the
  console, so a self-hosted instance keeps a history.
- The terminal log is clean now: short `HH:MM:SS` timestamps, no module
  paths, and the Discord library's connection chatter is hidden by
  default (warnings still show). The file log stays fully detailed, and
  `RUST_LOG` still overrides everything.
- `/about`, `/stats` and `/bot` were merged into a single `/info`.

### Fixed
- Flood protection: spamming the same command no longer overloads the bot,
  and it politely asks you to slow down instead. Those "slow down" and
  "not your button" notices now delete themselves after a few seconds so
  they don't clutter the chat.
- Poll votes can no longer be lost when two people tap at the same moment,
  and the "poll gone" / "already voted" notices clean themselves up too.
- A mistyped `/gamble` bet no longer burns the cooldown.
- Reminders: a corrupt record is dropped instead of firing at a random
  moment, restored timers are capped at 24h, and the group-chat fallback
  on Telegram now mentions you so the ping isn't missed.
- A transient storage error can no longer wipe your saved AI hosts or
  chats - the bot aborts the change and asks you to retry.
- `/ai share` accepts a model list with spaces after commas, and the
  model picker in the chat wizard selects the right model beyond the
  first row.
- `/ai host add` validates the URL like the setup wizard does.
- `/qr` counts characters, not bytes, so the 800-char limit is honest for
  non-Latin text.
- `/reverse` keeps emoji and accented characters intact.
- Reminder durations with absurdly large numbers no longer overflow.
- `/settings` buttons respect the DM-only rule like the command itself.
- `/shorten` now falls back across providers, so a single one being down
  no longer breaks it.
- `/ai` no longer mistakes a button tap for a chat message, and running it
  in a group no longer spills your settings into the channel.
- The leaderboard shows display names instead of raw ids.
- AI replies come through clean, without noisy headers.

## [0.1.0-alpha.1] - 2026-05-03

Initial release.

[0.1.1-alpha.1]: https://github.com/FoukoDev/Fouko/releases/tag/v0.1.1-alpha.1
[0.1.0-alpha.1]: https://github.com/FoukoDev/Fouko/releases/tag/v0.1.0-alpha.1
