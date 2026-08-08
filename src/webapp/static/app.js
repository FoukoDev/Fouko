"use strict";

// -- i18n ---------------------------------------------------------------------

var STR = {
  en: {
    open_in_tg: "Open this app from Telegram.",
    nav_chat: "Chat", nav_chats: "Chats", nav_hosts: "Hosts", nav_settings: "Settings",
    chats_heading: "Your chats", chats_new: "New chat",
    hosts_heading: "Your hosts", hosts_new: "Add host",
    send: "Send",
    chat_name_ph: "Chat name", host_name_ph: "Host name",
    host_url_ph: "URL (http://... or https://...)", host_key_ph: "API key (optional)",
    model_ph: "Model (pick or type)",
    create: "Create", add: "Add",
    use: "Switch to", history: "History", prompt: "Edit prompt",
    change_model: "Change model",
    model_title: "Model",
    model_free_ph: "Or type a model name",
    clear: "Clear history", del: "Delete", cancel: "Cancel", save: "Save",
    refresh: "Refresh", insecure_on: "Trust cert", insecure_off: "Strict TLS",
    active: "active", shared: "shared", insecure_badge: "insecure",
    has_prompt: "prompt",
    models_count: "{n} models",
    no_chats: "No chats yet.",
    no_chats_btn: "Create the first chat",
    no_hosts: "No hosts yet.",
    no_hosts_btn: "Add the first host",
    no_hosts_for_chat: "Add a host first - a chat needs one.",
    no_models: "No models on this host yet.",
    no_active_chat: "No active chat",
    no_active_chat_sub: "Tap here to pick or create one",
    confirm_del_chat: "Delete this chat and its history?",
    confirm_del_host: "Delete this host?",
    prompt_title: "System prompt",
    prompt_ph: "Empty to remove",
    history_title: "History",
    history_empty: "History is empty.",
    role_user: "You", role_assistant: "Assistant",
    add_model_ph: "Model name",
    tag: "Tag", untag: "Untag", cap_none: "text",
    sec_generation: "Generation", sec_assistant: "Assistant",
    sec_appearance: "Appearance", sec_info: "About",
    gen_image: "Images", gen_video: "Video", gen_audio: "Speech",
    gen_image_hint: "Model used when you ask for a picture.",
    gen_video_hint: "Model used for video generation.",
    gen_audio_hint: "Model used to voice text.",
    gen_auto: "Auto", gen_auto_now: "Auto (now: {m})", gen_none: "Auto (nothing available)",
    tools_label: "AI calls generation itself",
    tools_hint: "The model may draw, render video or speak on its own during a chat.",
    voice_label: "Voice",
    voice_hint: "Voice used for speech synthesis.",
    voice_default: "Server default",
    theme_label: "Theme",
    theme_auto: "Auto (Telegram)", theme_dark: "Dark", theme_light: "Light",
    lang_label: "Language",
    lang_auto: "Auto",
    info_version: "Bot version",
    info_chats: "Chats",
    info_hosts: "Hosts",
    video_queued: "Video is rendering, the bot will DM it to you.",
    speech_queued: "Audio is on its way, the bot will DM it to you.",
    host_added: "Host added, {n} models found.",
    host_added_nodiscover: "Host added, but model discovery failed: {e}",
    refreshed: "Model list updated: {n} models.",
    refresh_none: "The host returned no models; the stored list was kept.",
    saved: "Saved",
    done: "Done.",
    err_generic: "Request failed.",
    err_conn: "Connection lost, the answer may be incomplete.",
    stopped: "Stopped.",
    session_expired: "Session expired. Close and reopen the app from Telegram.",
    thinking: "..."
  },
  ru: {
    open_in_tg: "Открой это приложение из Telegram.",
    nav_chat: "Чат", nav_chats: "Чаты", nav_hosts: "Хосты", nav_settings: "Настройки",
    chats_heading: "Твои чаты", chats_new: "Новый чат",
    hosts_heading: "Твои хосты", hosts_new: "Добавить хост",
    send: "Отправить",
    chat_name_ph: "Имя чата", host_name_ph: "Имя хоста",
    host_url_ph: "URL (http://... или https://...)", host_key_ph: "API-ключ (необязательно)",
    model_ph: "Модель (выбери или впиши)",
    create: "Создать", add: "Добавить",
    use: "Переключиться", history: "История", prompt: "Сменить промпт",
    change_model: "Сменить модель",
    model_title: "Модель",
    model_free_ph: "Или впиши имя модели",
    clear: "Очистить историю", del: "Удалить", cancel: "Отмена", save: "Сохранить",
    refresh: "Обновить", insecure_on: "Доверять серт.", insecure_off: "Строгий TLS",
    active: "активный", shared: "общий", insecure_badge: "insecure",
    has_prompt: "промпт",
    models_count: "{n} моделей",
    no_chats: "Чатов пока нет.",
    no_chats_btn: "Создать первый чат",
    no_hosts: "Хостов пока нет.",
    no_hosts_btn: "Добавь первый хост",
    no_hosts_for_chat: "Сначала добавь хост - без него чат не создать.",
    no_models: "На этом хосте пока нет моделей.",
    no_active_chat: "Нет активного чата",
    no_active_chat_sub: "Нажми сюда, чтобы выбрать или создать",
    confirm_del_chat: "Удалить чат и его историю?",
    confirm_del_host: "Удалить хост?",
    prompt_title: "Системный промпт",
    prompt_ph: "Пусто = убрать",
    history_title: "История",
    history_empty: "История пуста.",
    role_user: "Ты", role_assistant: "Ассистент",
    add_model_ph: "Имя модели",
    tag: "Тег", untag: "Снять тег", cap_none: "текст",
    sec_generation: "Генерация", sec_assistant: "Ассистент",
    sec_appearance: "Внешний вид", sec_info: "Информация",
    gen_image: "Картинки", gen_video: "Видео", gen_audio: "Озвучка",
    gen_image_hint: "Модель, которая рисует картинки по запросу.",
    gen_video_hint: "Модель для генерации видео.",
    gen_audio_hint: "Модель, которая озвучивает текст.",
    gen_auto: "Авто", gen_auto_now: "Авто (сейчас: {m})", gen_none: "Авто (нет доступных)",
    tools_label: "ИИ сам вызывает генерацию",
    tools_hint: "Модель может сама рисовать, делать видео и озвучку прямо в чате.",
    voice_label: "Голос",
    voice_hint: "Голос для синтеза речи.",
    voice_default: "По умолчанию",
    theme_label: "Тема",
    theme_auto: "Авто (Telegram)", theme_dark: "Тёмная", theme_light: "Светлая",
    lang_label: "Язык",
    lang_auto: "Авто",
    info_version: "Версия бота",
    info_chats: "Чатов",
    info_hosts: "Хостов",
    video_queued: "Видео готовится, бот пришлёт его в ЛС.",
    speech_queued: "Аудио готовится, бот пришлёт его в ЛС.",
    host_added: "Хост добавлен, найдено моделей: {n}.",
    host_added_nodiscover: "Хост добавлен, но список моделей не получен: {e}",
    refreshed: "Список моделей обновлён: {n}.",
    refresh_none: "Хост вернул пустой список; сохранённый оставлен.",
    saved: "Сохранено",
    done: "Готово.",
    err_generic: "Запрос не удался.",
    err_conn: "Соединение оборвалось, ответ мог прийти не целиком.",
    stopped: "Остановлено.",
    session_expired: "Сессия истекла. Закрой и открой приложение из Telegram заново.",
    thinking: "..."
  }
};

var LANG = "en";
function t(key) {
  var d = STR[LANG] || STR.en;
  return d[key] || STR.en[key] || key;
}
function tf(key, args) {
  var s = t(key);
  Object.keys(args).forEach(function (k) {
    s = s.replace("{" + k + "}", args[k]);
  });
  return s;
}

// Known OpenAI voices; proxies may have others, this is just the picker.
var VOICES = ["alloy", "ash", "ballad", "coral", "echo", "fable", "onyx", "nova", "sage", "shimmer", "verse"];

// -- Telegram bootstrap ---------------------------------------------------------

var tg = window.Telegram && window.Telegram.WebApp;
var initData = tg ? tg.initData : "";

if (tg) {
  tg.ready();
  tg.expand();
}

var $ = function (id) { return document.getElementById(id); };

// Local prefs survive reloads; server state does not cover them.
function pref(key) {
  try { return localStorage.getItem(key); } catch (e) { return null; }
}
function setPref(key, val) {
  try {
    if (val) localStorage.setItem(key, val);
    else localStorage.removeItem(key);
  } catch (e) { /* private mode */ }
}

function applyTheme(mode) {
  document.body.classList.toggle("theme-dark", mode === "dark");
  document.body.classList.toggle("theme-light", mode === "light");
}

applyTheme(pref("theme") || "auto");

if (!initData) {
  // Opened outside Telegram: no auth possible, do nothing else.
  $("not-telegram").classList.remove("hidden");
} else {
  boot();
}

// -- state ---------------------------------------------------------------------

var appState = null; // last /api/state payload
var activeChatName = null;
var sending = false;
var streamAbort = null; // AbortController while a reply is streaming
var expandedHost = null;

function detectLang() {
  var saved = pref("lang");
  if (saved === "ru" || saved === "en") return saved;
  var code = (tg.initDataUnsafe && tg.initDataUnsafe.user && tg.initDataUnsafe.user.language_code) || "";
  return code.slice(0, 2) === "ru" ? "ru" : "en";
}

function boot() {
  LANG = detectLang();
  applyStrings();
  $("app").classList.remove("hidden");
  showSkeleton();
  loadState().then(function () {
    hideSkeleton();
    // Server language only applies when the user has no manual override.
    if (appState && appState.lang && !pref("lang")) {
      LANG = appState.lang === "ru" ? "ru" : "en";
      applyStrings();
      renderAll();
    }
    if (activeChatName) loadHistory(activeChatName);
  });
  wireEvents();
}

// Pulsing gray blocks in the chat log until the first state arrives.
function showSkeleton() {
  var log = $("chat-log");
  var sk = el("div", "skeleton");
  sk.id = "skeleton";
  for (var i = 0; i < 4; i++) {
    sk.appendChild(el("div", "sk-line" + (i % 2 ? " short" : "")));
  }
  log.appendChild(sk);
}

function hideSkeleton() {
  var sk = $("skeleton");
  if (sk) sk.remove();
}

function applyStrings() {
  $("nt-text").textContent = t("open_in_tg");
  $("nav-chat").querySelector(".label").textContent = t("nav_chat");
  $("nav-chats").querySelector(".label").textContent = t("nav_chats");
  $("nav-hosts").querySelector(".label").textContent = t("nav_hosts");
  $("nav-settings").querySelector(".label").textContent = t("nav_settings");
  $("chats-heading").textContent = t("chats_heading");
  $("chats-new-heading").textContent = t("chats_new");
  $("hosts-heading").textContent = t("hosts_heading");
  $("hosts-new-heading").textContent = t("hosts_new");
  $("new-chat-name").placeholder = t("chat_name_ph");
  $("new-chat-model").placeholder = t("model_ph");
  $("new-chat-btn").textContent = t("create");
  $("new-host-name").placeholder = t("host_name_ph");
  $("new-host-url").placeholder = t("host_url_ph");
  $("new-host-key").placeholder = t("host_key_ph");
  $("new-host-btn").textContent = t("add");
  $("sec-generation").textContent = t("sec_generation");
  $("sec-assistant").textContent = t("sec_assistant");
  $("sec-appearance").textContent = t("sec_appearance");
  $("sec-info").textContent = t("sec_info");
  $("gen-image-label").textContent = t("gen_image");
  $("gen-video-label").textContent = t("gen_video");
  $("gen-audio-label").textContent = t("gen_audio");
  $("gen-image-hint").textContent = t("gen_image_hint");
  $("gen-video-hint").textContent = t("gen_video_hint");
  $("gen-audio-hint").textContent = t("gen_audio_hint");
  $("tools-label").textContent = t("tools_label");
  $("tools-hint").textContent = t("tools_hint");
  $("voice-label").textContent = t("voice_label");
  $("voice-hint").textContent = t("voice_hint");
  $("theme-label").textContent = t("theme_label");
  $("lang-label").textContent = t("lang_label");
  $("info-version-label").textContent = t("info_version");
  $("info-chats-label").textContent = t("info_chats");
  $("info-hosts-label").textContent = t("info_hosts");
  $("modal-cancel").textContent = t("cancel");
  $("modal-save").textContent = t("save");
}

// -- api helpers -----------------------------------------------------------------

function api(path, body) {
  return fetch(path, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-Init-Data": initData
    },
    body: JSON.stringify(body || {})
  }).then(function (resp) {
    if (resp.status === 401) {
      sessionExpired();
      throw new Error(t("session_expired"));
    }
    return resp.json().catch(function () { return {}; }).then(function (data) {
      if (!resp.ok) throw new Error(data.error || t("err_generic"));
      return data;
    });
  });
}

// Init data outlives its 12h validity window: nothing works anymore,
// so replace the UI with a clear "reopen the app" message.
function sessionExpired() {
  $("app").classList.add("hidden");
  $("nt-text").textContent = t("session_expired");
  $("not-telegram").classList.remove("hidden");
}

function toast(msg) {
  var el = $("toast");
  el.textContent = msg;
  el.classList.remove("hidden");
  clearTimeout(el._timer);
  el._timer = setTimeout(function () { el.classList.add("hidden"); }, 2000);
}

function ask(question, cb) {
  if (tg && tg.showConfirm) {
    tg.showConfirm(question, function (yes) { if (yes) cb(); });
  } else if (window.confirm(question)) {
    cb();
  }
}

function loadState() {
  return api("/api/state").then(function (data) {
    appState = data;
    var act = (data.chats || []).find(function (c) { return c.active; });
    activeChatName = act ? act.name : null;
    renderAll();
  }).catch(function (e) {
    toast(e.message);
  });
}

// -- small dom helpers -------------------------------------------------------------

function el(tag, cls, text) {
  var node = document.createElement(tag);
  if (cls) node.className = cls;
  if (text !== undefined) node.textContent = text;
  return node;
}

function btn(label, cls, onclick) {
  var b = el("button", cls, label);
  b.type = "button";
  b.addEventListener("click", onclick);
  return b;
}

function badge(text, kind) {
  return el("span", "badge" + (kind ? " " + kind : ""), text);
}

// -- action sheet -------------------------------------------------------------------

function openSheet(title, items) {
  var sheet = $("sheet");
  sheet.innerHTML = "";
  if (title) sheet.appendChild(el("div", "sheet-title", title));
  items.forEach(function (it) {
    var b = btn(it.label, it.danger ? "danger" : "", function () {
      closeSheet();
      it.action();
    });
    sheet.appendChild(b);
  });
  $("sheet-backdrop").classList.remove("hidden");
  sheet.classList.remove("hidden");
}

function closeSheet() {
  $("sheet-backdrop").classList.add("hidden");
  $("sheet").classList.add("hidden");
}

// -- modal --------------------------------------------------------------------------

// Opens the shared modal; onSave null hides the save button (viewer mode).
function openModal(title, bodyNode, onSave) {
  $("modal-title").textContent = title;
  var body = $("modal-body");
  body.innerHTML = "";
  body.appendChild(bodyNode);
  var save = $("modal-save");
  save.classList.toggle("hidden", !onSave);
  save.onclick = onSave ? function () { onSave(); } : null;
  $("modal").classList.remove("hidden");
}

function closeModal() {
  $("modal").classList.add("hidden");
}

// -- rendering --------------------------------------------------------------------

function renderAll() {
  renderChatHeader();
  renderChats();
  renderHosts();
  renderSettings();
}

function renderChatHeader() {
  var title = $("chat-title");
  var sub = $("chat-sub");
  if (!activeChatName) {
    title.textContent = t("no_active_chat");
    sub.textContent = t("no_active_chat_sub");
    return;
  }
  var chat = (appState.chats || []).find(function (c) { return c.name === activeChatName; });
  title.textContent = activeChatName;
  sub.textContent = chat ? (chat.host ? chat.host + " / " : "") + chat.model : "";
}

// Short message timestamp, TG style: "HH:MM" today, "DD.MM HH:MM" otherwise.
function fmtTime(ms) {
  var d = new Date(ms);
  var time = d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  var now = new Date();
  if (d.toDateString() === now.toDateString()) return time;
  var dd = String(d.getDate()).padStart(2, "0");
  var mm = String(d.getMonth() + 1).padStart(2, "0");
  return dd + "." + mm + " " + time;
}

// Append (or refresh) the small corner timestamp inside a bubble.
function stampMsg(div, ms) {
  var old = div.querySelector(".msg-time");
  if (old) old.remove();
  div.appendChild(el("span", "msg-time", fmtTime(ms)));
}

function addMsg(role, text, ts) {
  var log = $("chat-log");
  var div = el("div", "msg " + role, text);
  if (ts) stampMsg(div, ts);
  log.appendChild(div);
  log.scrollTop = log.scrollHeight;
  return div;
}

function addImage(dataUrl) {
  var log = $("chat-log");
  var div = el("div", "msg assistant");
  var img = document.createElement("img");
  img.src = dataUrl;
  div.appendChild(img);
  log.appendChild(div);
  log.scrollTop = log.scrollHeight;
}

function loadHistory(name) {
  api("/api/chat/history", { name: name }).then(function (data) {
    var log = $("chat-log");
    log.innerHTML = "";
    (data.messages || []).forEach(function (m) {
      if (m.role === "system") return;
      addMsg(m.role === "assistant" ? "assistant" : "user", m.content, m.ts ? m.ts * 1000 : 0);
    });
  }).catch(function (e) { toast(e.message); });
}

// -- chats tab ----------------------------------------------------------------------

function switchChat(name) {
  api("/api/chat/use", { name: name }).then(function () {
    activeChatName = name;
    return loadState();
  }).then(function () {
    loadHistory(name);
    showTab("chat");
  }).catch(function (e) { toast(e.message); });
}

function editPrompt(name) {
  // Pull the current prompt first so the editor is prefilled.
  api("/api/chat/history", { name: name }).then(function (data) {
    var ta = document.createElement("textarea");
    ta.value = data.prompt || "";
    ta.placeholder = t("prompt_ph");
    openModal(t("prompt_title"), ta, function () {
      api("/api/chat/prompt", { name: name, prompt: ta.value })
        .then(function () {
          closeModal();
          toast(t("saved"));
          return loadState();
        })
        .catch(function (e) { toast(e.message); });
    });
    ta.focus();
  }).catch(function (e) { toast(e.message); });
}

// Model picker modal: the chat host's models as a radio list, current
// one checked, plus a free-form input for anything not listed.
function openModelPicker(chatName) {
  var chat = (appState && appState.chats || []).find(function (c) { return c.name === chatName; });
  if (!chat) return;
  var host = (appState.hosts || []).find(function (h) { return h.name === chat.host; });
  var models = (host && host.models || []).map(function (m) { return m.name; });

  var wrap = el("div", "model-pick");
  var free = document.createElement("input");
  free.placeholder = t("model_free_ph");
  free.autocomplete = "off";

  function apply(model) {
    api("/api/chat/model", { name: chatName, model: model })
      .then(function () {
        closeModal();
        toast(t("saved"));
        return loadState();
      })
      .catch(function (e) { toast(e.message); });
  }

  models.forEach(function (m) {
    var row = el("label", "model-opt");
    var radio = document.createElement("input");
    radio.type = "radio";
    radio.name = "model-pick";
    radio.value = m;
    radio.checked = m === chat.model;
    radio.addEventListener("change", function () { apply(m); });
    row.appendChild(radio);
    row.appendChild(el("span", "model-opt-name", m));
    wrap.appendChild(row);
  });
  wrap.appendChild(free);

  openModal(t("model_title"), wrap, function () {
    var v = free.value.trim();
    if (v) {
      apply(v);
    } else {
      var checked = wrap.querySelector("input[type=radio]:checked");
      if (checked && checked.value !== chat.model) apply(checked.value);
      else closeModal();
    }
  });
}

function viewHistory(name) {
  api("/api/chat/history", { name: name }).then(function (data) {
    var wrap = el("div", "hist");
    var msgs = data.messages || [];
    if (!msgs.length) {
      wrap.appendChild(el("div", "hint", t("history_empty")));
    }
    msgs.forEach(function (m) {
      if (m.role === "system") return;
      wrap.appendChild(el("div", "h-role", m.role === "assistant" ? t("role_assistant") : t("role_user")));
      wrap.appendChild(el("div", "h-text", m.content));
    });
    openModal(t("history_title"), wrap, null);
  }).catch(function (e) { toast(e.message); });
}

function chatSheet(c) {
  openSheet(c.name, [
    { label: t("use"), action: function () { switchChat(c.name); } },
    { label: t("change_model"), action: function () { openModelPicker(c.name); } },
    { label: t("history"), action: function () { viewHistory(c.name); } },
    { label: t("prompt"), action: function () { editPrompt(c.name); } },
    { label: t("clear"), action: function () {
      api("/api/chat/clear", { name: c.name })
        .then(function () {
          toast(t("done"));
          if (c.name === activeChatName) $("chat-log").innerHTML = "";
        })
        .catch(function (e) { toast(e.message); });
    } },
    { label: t("del"), danger: true, action: function () {
      ask(t("confirm_del_chat"), function () {
        api("/api/chat/del", { name: c.name })
          .then(function () { toast(t("done")); return loadState(); })
          .catch(function (e) { toast(e.message); });
      });
    } }
  ]);
}

function kebabIcon() {
  var b = el("button", "kebab");
  b.type = "button";
  b.innerHTML = '<svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">' +
    '<circle cx="12" cy="5" r="1.7"/><circle cx="12" cy="12" r="1.7"/><circle cx="12" cy="19" r="1.7"/></svg>';
  return b;
}

function renderChats() {
  var list = $("chats-list");
  list.innerHTML = "";
  var chats = appState ? appState.chats || [] : [];
  if (!chats.length) {
    var empty = el("div", "card empty");
    empty.appendChild(el("div", "", t("no_chats")));
    empty.appendChild(btn(t("no_chats_btn"), "btn primary", function () {
      $("new-chat-name").focus();
    }));
    list.appendChild(empty);
  }
  chats.forEach(function (c) {
    var card = el("div", "card tappable");

    var row = el("div", "row");
    var main = el("div", "grow");
    main.style.flex = "1";
    main.style.minWidth = "0";
    main.appendChild(el("div", "name", c.name));
    main.appendChild(el("div", "sub", (c.host ? c.host + " / " : "") + c.model));
    row.appendChild(main);
    if (c.active) row.appendChild(badge(t("active"), "on"));
    if (c.has_prompt) row.appendChild(badge(t("has_prompt")));

    var kb = kebabIcon();
    kb.addEventListener("click", function (ev) {
      ev.stopPropagation();
      chatSheet(c);
    });
    row.appendChild(kb);
    card.appendChild(row);

    card.addEventListener("click", function () { switchChat(c.name); });
    list.appendChild(card);
  });

  // New-chat form: host select + a model input backed by a datalist.
  // Without hosts the form can't work - hint and disable it.
  var hostSel = $("new-chat-host");
  var prev = hostSel.value;
  hostSel.innerHTML = "";
  var hosts = appState ? appState.hosts || [] : [];
  var form = $("chat-new-form");
  var noHosts = !hosts.length;
  form.classList.toggle("disabled-form", noHosts);
  ["new-chat-name", "new-chat-host", "new-chat-model", "new-chat-btn"].forEach(function (id) {
    $(id).disabled = noHosts;
  });
  var hintEl = $("new-chat-hint");
  hintEl.textContent = noHosts ? t("no_hosts_for_chat") : "";
  hintEl.classList.toggle("hidden", !noHosts);
  hosts.forEach(function (h) {
    var opt = document.createElement("option");
    opt.value = h.name;
    opt.textContent = h.name;
    hostSel.appendChild(opt);
  });
  if (prev && hosts.some(function (h) { return h.name === prev; })) hostSel.value = prev;
  fillModelList();
}

function fillModelList() {
  var hosts = appState ? appState.hosts || [] : [];
  var hostName = $("new-chat-host").value;
  var host = hosts.find(function (h) { return h.name === hostName; });
  var dl = $("new-chat-models");
  dl.innerHTML = "";
  ((host && host.models) || []).forEach(function (m) {
    var opt = document.createElement("option");
    opt.value = m.name;
    dl.appendChild(opt);
  });
}

// -- hosts tab ----------------------------------------------------------------------

function renderHosts() {
  var list = $("hosts-list");
  list.innerHTML = "";
  var hosts = appState ? appState.hosts || [] : [];
  if (!hosts.length) {
    var empty = el("div", "card empty");
    empty.appendChild(el("div", "", t("no_hosts")));
    empty.appendChild(btn(t("no_hosts_btn"), "btn primary", function () {
      $("new-host-name").focus();
    }));
    list.appendChild(empty);
  }
  hosts.forEach(function (h) {
    var card = el("div", "card");

    var head = el("div", "row");
    head.style.cursor = "pointer";
    var main = el("div", "");
    main.style.flex = "1";
    main.style.minWidth = "0";
    main.appendChild(el("div", "name", h.name));
    if (h.url) main.appendChild(el("div", "sub", h.url));
    head.appendChild(main);
    head.appendChild(badge(tf("models_count", { n: (h.models || []).length })));
    if (h.insecure) head.appendChild(badge(t("insecure_badge"), "warn"));
    if (h.shared) head.appendChild(badge(t("shared"), "info"));
    head.addEventListener("click", function () {
      expandedHost = expandedHost === h.name ? null : h.name;
      renderHosts();
    });
    card.appendChild(head);

    if (expandedHost === h.name) {
      card.appendChild(hostBody(h));
    }
    list.appendChild(card);
  });
}

function hostBody(h) {
  var body = el("div", "host-body");
  var models = el("div", "models");
  if (!(h.models || []).length) {
    models.appendChild(el("div", "hint", t("no_models")));
  }
  (h.models || []).forEach(function (m) {
    var row = el("div", "model-row");
    row.appendChild(el("span", "mname", m.name));
    if (m.caps && m.caps.length) {
      m.caps.forEach(function (c) { row.appendChild(el("span", "chip", c)); });
    } else {
      row.appendChild(el("span", "chip", t("cap_none")));
    }

    if (!h.shared) {
      var capSel = document.createElement("select");
      ["image", "video", "audio"].forEach(function (c) {
        var opt = document.createElement("option");
        opt.value = c;
        opt.textContent = c;
        capSel.appendChild(opt);
      });
      row.appendChild(capSel);
      row.appendChild(btn(t("tag"), "btn ghost", function () {
        api("/api/model/tag", { host: h.name, model: m.name, cap: capSel.value })
          .then(function () { toast(t("saved")); return loadState(); })
          .catch(function (e) { toast(e.message); });
      }));
      row.appendChild(btn(t("untag"), "btn ghost", function () {
        api("/api/model/untag", { host: h.name, model: m.name })
          .then(function () { toast(t("saved")); return loadState(); })
          .catch(function (e) { toast(e.message); });
      }));
      row.appendChild(btn(t("del"), "btn danger", function () {
        api("/api/model/del", { host: h.name, model: m.name })
          .then(function () { toast(t("done")); return loadState(); })
          .catch(function (e) { toast(e.message); });
      }));
    }
    models.appendChild(row);
  });
  body.appendChild(models);

  if (!h.shared) {
    var addRow = el("div", "model-add");
    var input = document.createElement("input");
    input.placeholder = t("add_model_ph");
    addRow.appendChild(input);
    addRow.appendChild(btn(t("add"), "btn ghost", function () {
      var v = input.value.trim();
      if (!v) return;
      api("/api/model/add", { host: h.name, model: v })
        .then(function () { toast(t("done")); return loadState(); })
        .catch(function (e) { toast(e.message); });
    }));
    body.appendChild(addRow);

    var actions = el("div", "host-actions");
    actions.appendChild(btn(t("refresh"), "btn ghost", function () {
      api("/api/host/refresh", { name: h.name }).then(function (d) {
        toast(d.models ? tf("refreshed", { n: d.models }) : t("refresh_none"));
        return loadState();
      }).catch(function (e) { toast(e.message); });
    }));
    actions.appendChild(btn(h.insecure ? t("insecure_off") : t("insecure_on"), "btn ghost", function () {
      api("/api/host/insecure", { name: h.name, on: !h.insecure })
        .then(function () { toast(t("saved")); return loadState(); })
        .catch(function (e) { toast(e.message); });
    }));
    actions.appendChild(btn(t("del"), "btn danger", function () {
      ask(t("confirm_del_host"), function () {
        api("/api/host/del", { name: h.name })
          .then(function () { toast(t("done")); return loadState(); })
          .catch(function (e) { toast(e.message); });
      });
    }));
    body.appendChild(actions);
  }
  return body;
}

// -- settings tab ---------------------------------------------------------------------

function renderSettings() {
  if (!appState) return;
  ["image", "video", "audio"].forEach(function (cap) {
    var sel = $("gen-" + cap);
    sel.innerHTML = "";
    var cur = appState.gen && appState.gen[cap];

    var auto = document.createElement("option");
    auto.value = "auto";
    if (cur && !cur.pinned) {
      auto.textContent = tf("gen_auto_now", { m: cur.host + " / " + cur.model });
    } else if (!cur) {
      auto.textContent = t("gen_none");
    } else {
      auto.textContent = t("gen_auto");
    }
    sel.appendChild(auto);

    // Every model with this capability, across hosts.
    (appState.hosts || []).forEach(function (h) {
      (h.models || []).forEach(function (m) {
        if ((m.caps || []).indexOf(cap) < 0) return;
        var opt = document.createElement("option");
        opt.value = h.name + "\u0001" + m.name;
        opt.textContent = h.name + " / " + m.name;
        sel.appendChild(opt);
      });
    });

    if (cur && cur.pinned) {
      sel.value = cur.host + "\u0001" + cur.model;
    } else {
      sel.value = "auto";
    }
    sel.onchange = function () {
      var body;
      if (sel.value === "auto") {
        body = { cap: cap, model: "auto" };
      } else {
        var parts = sel.value.split("\u0001");
        body = { cap: cap, host: parts[0], model: parts[1] };
      }
      api("/api/gen/set", body)
        .then(function () { toast(t("saved")); return loadState(); })
        .catch(function (e) { toast(e.message); });
    };
  });

  var toggle = $("tools-toggle");
  toggle.checked = !!appState.tools;
  toggle.onchange = function () {
    api("/api/tools/set", { on: toggle.checked })
      .then(function () { toast(t("saved")); })
      .catch(function (e) { toast(e.message); });
  };

  var voiceSel = $("voice-select");
  voiceSel.innerHTML = "";
  var dflt = document.createElement("option");
  dflt.value = "";
  dflt.textContent = t("voice_default");
  voiceSel.appendChild(dflt);
  VOICES.forEach(function (v) {
    var opt = document.createElement("option");
    opt.value = v;
    opt.textContent = v;
    voiceSel.appendChild(opt);
  });
  voiceSel.value = appState.voice || "";
  voiceSel.onchange = function () {
    api("/api/voice/set", { voice: voiceSel.value })
      .then(function () { toast(t("saved")); })
      .catch(function (e) { toast(e.message); });
  };

  // Appearance: theme and language live in localStorage only.
  var themeSel = $("theme-select");
  themeSel.innerHTML = "";
  [["auto", t("theme_auto")], ["dark", t("theme_dark")], ["light", t("theme_light")]].forEach(function (p) {
    var opt = document.createElement("option");
    opt.value = p[0];
    opt.textContent = p[1];
    themeSel.appendChild(opt);
  });
  themeSel.value = pref("theme") || "auto";
  themeSel.onchange = function () {
    setPref("theme", themeSel.value === "auto" ? null : themeSel.value);
    applyTheme(themeSel.value);
    toast(t("saved"));
  };

  var langSel = $("lang-select");
  langSel.innerHTML = "";
  [["auto", t("lang_auto")], ["en", "English"], ["ru", "Русский"]].forEach(function (p) {
    var opt = document.createElement("option");
    opt.value = p[0];
    opt.textContent = p[1];
    langSel.appendChild(opt);
  });
  langSel.value = pref("lang") || "auto";
  langSel.onchange = function () {
    setPref("lang", langSel.value === "auto" ? null : langSel.value);
    LANG = detectLang();
    if (langSel.value === "auto" && appState && appState.lang) {
      LANG = appState.lang === "ru" ? "ru" : "en";
    }
    applyStrings();
    renderAll();
    toast(t("saved"));
  };

  $("info-version").textContent = appState.version || "?";
  $("info-chats").textContent = String((appState.chats || []).length);
  $("info-hosts").textContent = String((appState.hosts || []).length);
}

// -- chat streaming ---------------------------------------------------------------

function sendMessage(text) {
  if (!activeChatName) {
    toast(t("no_active_chat"));
    showTab("chats");
    return;
  }
  if (sending) return;
  sending = true;
  updateSendState();
  addMsg("user", text, Date.now());
  var bubble = addMsg("assistant", t("thinking"));
  bubble.classList.add("streaming");
  var settled = false; // done/error event seen; a later abort is harmless
  // The send button doubles as a stop button while streaming.
  streamAbort = new AbortController();

  fetch("/api/chat/send", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-Init-Data": initData
    },
    body: JSON.stringify({ chat: activeChatName, text: text }),
    signal: streamAbort.signal
  }).then(function (resp) {
    if (resp.status === 401) {
      sessionExpired();
      throw new Error(t("session_expired"));
    }
    if (!resp.ok) {
      return resp.json().catch(function () { return {}; }).then(function (d) {
        throw new Error(d.error || t("err_generic"));
      });
    }
    return readSse(resp, bubble, function () { settled = true; });
  }).catch(function (e) {
    if (settled) return;
    if (e.name === "AbortError") {
      // User pressed stop: keep whatever arrived, mark it visibly.
      if (!bubble.textContent || bubble.textContent === t("thinking")) {
        bubble.textContent = t("stopped");
      }
      stampMsg(bubble, Date.now());
      return;
    }
    // A drop mid-stream keeps the partial text; a pre-stream failure
    // replaces the placeholder with the error.
    if (bubble.textContent && bubble.textContent !== t("thinking")) {
      toast(t("err_conn"));
    } else {
      bubble.textContent = e.message;
    }
  }).finally(function () {
    bubble.classList.remove("streaming");
    streamAbort = null;
    sending = false;
    updateSendState();
  });
}

// Parse the SSE body of a fetch response. EventSource can't POST, so we
// read the stream by hand; the format is simple enough.
function readSse(resp, bubble, onSettle) {
  var reader = resp.body.getReader();
  var decoder = new TextDecoder();
  var buf = "";

  function handle(eventName, data) {
    if (eventName === "delta" || eventName === "done") {
      try { bubble.textContent = JSON.parse(data); } catch (e) { /* skip */ }
      if (eventName === "done") {
        onSettle();
        bubble.classList.remove("streaming");
        // The answer is final now, stamp it like a settled message.
        stampMsg(bubble, Date.now());
      }
      var log = $("chat-log");
      log.scrollTop = log.scrollHeight;
    } else if (eventName === "image") {
      addImage(data);
    } else if (eventName === "notice") {
      addMsg("notice", t(data));
    } else if (eventName === "error") {
      onSettle();
      bubble.classList.remove("streaming");
      try { bubble.textContent = JSON.parse(data); } catch (e) { bubble.textContent = data; }
    }
  }

  function pump() {
    return reader.read().then(function (res) {
      if (res.done) return;
      buf += decoder.decode(res.value, { stream: true });
      var blocks = buf.split("\n\n");
      buf = blocks.pop();
      blocks.forEach(function (block) {
        var name = "message";
        var data = [];
        block.split("\n").forEach(function (line) {
          if (line.indexOf("event:") === 0) name = line.slice(6).trim();
          else if (line.indexOf("data:") === 0) data.push(line.slice(5).trim());
        });
        if (data.length) handle(name, data.join("\n"));
      });
      return pump();
    });
  }
  return pump();
}

// -- events ----------------------------------------------------------------------

function showTab(name) {
  ["chat", "chats", "hosts", "settings"].forEach(function (n) {
    $("tab-" + n).classList.toggle("hidden", n !== name);
    $("nav-" + n).classList.toggle("active", n === name);
  });
}

function updateSendState() {
  var btn = $("chat-send");
  if (sending) {
    // Streaming: the button turns into a stop control.
    btn.disabled = false;
    $("send-icon").classList.add("hidden");
    $("stop-icon").classList.remove("hidden");
  } else {
    btn.disabled = !$("chat-text").value.trim();
    $("send-icon").classList.remove("hidden");
    $("stop-icon").classList.add("hidden");
  }
}

function autogrow(ta) {
  ta.style.height = "auto";
  ta.style.height = Math.min(ta.scrollHeight, 120) + "px";
}

function wireEvents() {
  document.querySelectorAll(".tabs button").forEach(function (b) {
    b.addEventListener("click", function () { showTab(b.dataset.tab); });
  });

  // The chat title jumps to the chat list; the model line under it opens
  // the model picker for the active chat.
  $("chat-title").addEventListener("click", function () { showTab("chats"); });
  $("chat-sub").addEventListener("click", function () {
    if (activeChatName) openModelPicker(activeChatName);
    else showTab("chats");
  });

  $("sheet-backdrop").addEventListener("click", closeSheet);
  $("modal-cancel").addEventListener("click", closeModal);
  $("modal").addEventListener("click", function (ev) {
    if (ev.target === $("modal")) closeModal();
  });

  var input = $("chat-text");

  $("chat-form").addEventListener("submit", function (ev) {
    ev.preventDefault();
    if (sending) {
      // The button is a stop control during a stream.
      if (streamAbort) streamAbort.abort();
      return;
    }
    var text = input.value.trim();
    if (!text) return;
    input.value = "";
    autogrow(input);
    updateSendState();
    sendMessage(text);
  });

  input.addEventListener("input", function () {
    autogrow(input);
    updateSendState();
  });

  // Enter sends, Shift+Enter makes a newline.
  input.addEventListener("keydown", function (ev) {
    if (ev.key === "Enter" && !ev.shiftKey) {
      ev.preventDefault();
      $("chat-form").dispatchEvent(new Event("submit", { cancelable: true }));
    }
  });

  // Mobile keyboard: shrink the layout to the visual viewport so the
  // input never hides behind the keyboard; fall back to a scroll nudge.
  if (window.visualViewport) {
    window.visualViewport.addEventListener("resize", function () {
      var full = window.innerHeight;
      var vh = window.visualViewport.height;
      // Only pin the height while the keyboard eats real space.
      document.getElementById("app").style.height =
        (full - vh > 80) ? vh + "px" : "";
      var log = $("chat-log");
      log.scrollTop = log.scrollHeight;
    });
  } else {
    input.addEventListener("focus", function () {
      setTimeout(function () {
        input.scrollIntoView({ block: "end" });
      }, 300);
    });
  }

  $("new-chat-host").addEventListener("change", fillModelList);

  $("chat-new-form").addEventListener("submit", function (ev) {
    ev.preventDefault();
    var name = $("new-chat-name").value.trim();
    var host = $("new-chat-host").value;
    var model = $("new-chat-model").value.trim();
    if (!name || !host || !model) return;
    api("/api/chat/new", { name: name, host: host, model: model })
      .then(function () {
        $("new-chat-name").value = "";
        $("new-chat-model").value = "";
        toast(t("done"));
        return loadState();
      })
      .then(function () { loadHistory(name); showTab("chat"); })
      .catch(function (e) { toast(e.message); });
  });

  $("host-new-form").addEventListener("submit", function (ev) {
    ev.preventDefault();
    var name = $("new-host-name").value.trim();
    var url = $("new-host-url").value.trim();
    var key = $("new-host-key").value.trim();
    if (!name || !url) return;
    api("/api/host/add", { name: name, url: url, key: key })
      .then(function (d) {
        $("new-host-name").value = "";
        $("new-host-url").value = "";
        $("new-host-key").value = "";
        if (d.discover_error) {
          toast(tf("host_added_nodiscover", { e: d.discover_error }));
        } else {
          toast(tf("host_added", { n: d.models || 0 }));
        }
        return loadState();
      })
      .catch(function (e) { toast(e.message); });
  });
}
