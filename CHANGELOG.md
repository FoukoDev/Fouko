# Changelog

All notable changes to FoukoBot are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2-alpha.1] - 2026-08-08

### Added
- `/ai draw <prompt>` - generate an image with any image model on your
  hosts (DALL-E, SDXL, Flux, ...). In a DM a plain "draw a cat" (or
  "нарисуй кота") is detected and routed to the image model automatically.
- `/ai video <prompt>` - generate a short video with a Sora-style video
  model on your hosts and get it as a native video. Rendering takes a
  couple of minutes; the bot says so and posts the result when it's done.
  In a DM a plain "make a video of ..." is detected automatically.
- `/ai speak <text>` - turn text into speech with a TTS model on your
  hosts and get it as an audio file.
- `/ai models` - list every model on your hosts (own and family-shared)
  tagged with what it can do: image, video or audio. Capabilities come
  from the host's metadata when available, or from the model name.
- Vision: send the bot a photo (with a caption or without) in your `/ai`
  chat and a vision-capable model (gpt-4o, llava, qwen-vl, ...) will
  answer about it. Models that can't see images politely say so instead
  of silently ignoring the picture. The photo itself is never saved to
  chat history - only your text is.
- `/ai speak voice:<name> <text>` - pick a voice for text-to-speech, and
  `/ai speak voices` lists the known ones. The last voice you chose is
  remembered and becomes your default.
- Docker: the repo now ships a `Dockerfile` and `docker-compose.yml`.
  Drop your `.env` next to them, run `docker compose up -d --build`, and
  the database and logs live in `./data` on the host.
- Release workflow: pushing a version tag builds the bot and attaches a
  ready-to-run Linux binary to the GitHub release.
- The model can call generation itself: say "create a cat picture" in
  your `/ai` chat and the AI decides to run `generate_image` (or
  `generate_video`, or `speak`) on its own, with the prompt it wrote.
  Works with any chat model - the chat model makes the call, your
  chosen generation model does the rendering. Turn it off with
  `/ai tools off`; hosts that don't support tool calling automatically
  fall back to the old phrase detection ("draw a cat" still works).
- `/ai gen` - pin a model per generation type: `/ai gen image <host>
  <model>` makes every image request use that model, same for `video`
  and `audio`; `/ai gen image auto` returns to automatic pick. Bare
  `/ai gen` shows the current choices.
- `/ai model tag <host> <model> image|video|audio` - tell the bot what
  a model can do when its name gives nothing away; `/ai model untag`
  goes back to guessing from the name.
- `/ai host insecure <name> on|off` - use a host with a self-signed
  certificate. When a host fails with a certificate error, the bot
  suggests this command right in the error message. In practice you
  rarely need it: the bot now adapts on its own (see Changed), and the
  command remains as a manual override.
- `/ai model check <host> <model>` - probe a model for canned answers:
  some dead upstreams keep returning one captured response to every
  request. The check sends tiny inputs a live backend must react to and
  reports a verdict (live, canned, unstable) with the evidence. Also
  available as a button in the host's management card.
- Management buttons in `/ai`. Every chat has a card: use it, change its
  model with buttons, or delete it with a confirmation step. Every host
  you own has a card too: refresh its models, toggle self-signed
  certificates, check a model, or delete the host. Generation models
  (`/ai gen`) are picked with buttons as well, and when creating a chat
  you can also just type any model name instead of picking from the
  list.
- Telegram Mini App: the whole `/ai` in a web interface - chats with
  live streaming answers, hosts, models and settings, all the same data
  the bot uses. Sign-in is Telegram-only via the signed `initData`, so
  nobody else can open your chats. Enable it by setting `WEBAPP_URL` in
  `.env` and putting the built-in server behind a reverse proxy; the
  bot then shows an "Open app" button in `/ai`, and Telegram gets an
  app button right next to the message input field, automatically.
- No domain? `WEBAPP_TUNNEL=cloudflared` in `.env` makes the bot start
  a free Cloudflare quick tunnel on its own and use the
  `https://<random>.trycloudflare.com` URL it gets - the Mini App works
  from a laptop behind NAT with zero setup beyond installing the
  `cloudflared` binary. The URL changes on every restart, so it's for
  trying things out, not production.
- Mini App: the send button turns into a stop button while an answer is
  streaming. Stopping keeps whatever text already arrived instead of
  throwing it away.
- Mini App: messages carry a timestamp now, TG-style ("14:32" today,
  date otherwise), and it's stored with the chat history - old entries
  without one still load fine.
- Mini App: switch the chat's model on the spot - tap the model line in
  the chat header or use "Change model" in the chat's menu. History
  survives the switch.
- Mini App redesign: card layout, accent gradients, a dark/light theme
  override (auto follows Telegram), interface language picker, and the
  bot version in Settings.

### Changed
- The OpenAI-compatible client (chat, images, video, speech, model
  discovery) moved into the framework as `foukoapi::genai`; the bot now
  uses it through the new `genai` feature. Behaviour is unchanged.
- AI answers now stream live: the bot posts a placeholder and keeps
  editing it as the model generates, so you watch the answer appear in
  real time instead of waiting for the whole thing.
- `/ai host add` and `/ai host refresh` now show the reason when a
  host's model list can't be fetched (wrong key, bad certificate, host
  down) instead of silently reporting "0 models".
- Hosts with self-signed certificates just work now: when a host's TLS
  setup changes, the bot adapts by itself instead of waiting for you to
  run `/ai host insecure`. The command stays as a manual override.
- The "Open app" button in `/ai` only shows on Telegram now. On Discord
  the Mini App has no way to sign you in, so the button there would
  open a page that can't do anything.
- Generation says what it's doing: "Drawing with <model> - hold on...",
  "Voicing it with <model>...", and the video notice warns it can take
  a couple of minutes. When the model calls generation itself, the
  status recycles the streaming placeholder (edit-in-place) instead of
  stacking a second notice under it - one status message, not two.

### Fixed
- The bot no longer drops its database into `target/`, and it finds the
  `.env` next to the binary when started from another directory (see
  the framework changelog). An existing database keeps working: a
  legacy file next to the binary is picked up with a log line
  suggesting where to move it.
- Panic alerts to the owner are throttled to one per 10 minutes: a dead
  bot token or a broken network used to restart the bot in a loop and
  spam your DMs with an alert on every lap.
- The model can no longer claim a video or audio job succeeded while it
  is still rendering in the background (Mini App tool calls): the tool
  result now tells it the job was queued and may still fail, so the
  answer says "being prepared" instead of a premature "done!"
- Owner verification no longer fails just because a platform was slow
  to connect. Discord's gateway takes a few seconds longer than
  Telegram, and the startup lookup used to give up on whichever was
  late; now each platform gets a short grace period before the check
  runs.
- A broken generation-model pin (host gone, model dropped, access
  revoked) is logged once and falls back to auto-pick quietly, instead
  of repeating the same warning on every single message.
- Mini App hardening pass: mutating api requests are
  rate-limited per user (reads are exempt, so a refresh never trips
  it), two answers can't stream into one chat at the same time and race
  the history write, an expired session shows a proper "reopen the app"
  screen instead of dead buttons, and a connection drop mid-answer
  keeps the partial text instead of blanking the bubble.

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

[0.1.2-alpha.1]: https://github.com/FoukoDev/Fouko/releases/tag/v0.1.2-alpha.1
[0.1.1-alpha.1]: https://github.com/FoukoDev/Fouko/releases/tag/v0.1.1-alpha.1
[0.1.0-alpha.1]: https://github.com/FoukoDev/Fouko/releases/tag/v0.1.0-alpha.1
