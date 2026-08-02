//! All user-facing copy lives here, keyed and translated once.
//!
//! Commands pull strings through [`Services::tr`](crate::commands) instead
//! of inlining a `match lang { ... }` every time. English is the safety
//! net: anything a language is missing falls back to it, so adding a new
//! locale is just filling in the rows you have time for.

use foukoapi::I18n;

/// Languages the bot offers in `/lang`.
pub const SUPPORTED: &[&str] = &["en", "ru", "uk", "de", "es"];

/// Build the full string catalogue.
pub fn catalogue() -> I18n {
    I18n::new()
        // -- shared ---------------------------------------------------------
        .add(
            "rate_limited",
            &[
                ("en", "Easy there - try again in {}."),
                ("ru", "Не так быстро - попробуй снова через {}."),
                ("uk", "Не так швидко - спробуй знову за {}."),
                ("de", "Immer mit der Ruhe - versuch es in {} erneut."),
                ("es", "Con calma - inténtalo de nuevo en {}."),
            ],
        )
        .add(
            "flood_notice",
            &[
                ("en", "You're sending commands too fast. Give it a few seconds and try again."),
                ("ru", "Слишком много команд подряд. Подожди пару секунд и попробуй снова."),
                ("uk", "Забагато команд поспіль. Зачекай кілька секунд і спробуй знову."),
                ("de", "Zu viele Befehle auf einmal. Warte ein paar Sekunden und versuch es erneut."),
                ("es", "Estás enviando comandos muy rápido. Espera unos segundos e inténtalo de nuevo."),
            ],
        )
        .add(
            "ach_unlocked",
            &[
                ("en", "🏅 Achievement unlocked"),
                ("ru", "🏅 Достижение получено"),
                ("uk", "🏅 Досягнення отримано"),
                ("de", "🏅 Erfolg freigeschaltet"),
                ("es", "🏅 Logro desbloqueado"),
            ],
        )
        .add(
            "not_your_button",
            &[
                ("en", "These buttons aren't yours - run the command yourself."),
                ("ru", "Эти кнопки не для тебя - вызови команду сам."),
                ("uk", "Ці кнопки не для тебе - виклич команду сам."),
                ("de", "Diese Knöpfe sind nicht für dich - ruf den Befehl selbst auf."),
                ("es", "Estos botones no son tuyos - usa el comando tú mismo."),
            ],
        )
        .add(
            "slow_down_title",
            &[
                ("en", "⏳ Slow Down"),
                ("ru", "⏳ Не так быстро"),
                ("uk", "⏳ Не так швидко"),
                ("de", "⏳ Langsamer"),
                ("es", "⏳ Más despacio"),
            ],
        )
        .add(
            "amount_invalid",
            &[
                ("en", "That amount doesn't look right - use a positive whole number."),
                ("ru", "Сумма не похожа на число - укажи целое положительное число."),
                ("uk", "Сума не схожа на число - вкажи ціле додатне число."),
                ("de", "Der Betrag sieht falsch aus - nutze eine positive ganze Zahl."),
                ("es", "Esa cantidad no parece válida - usa un número entero positivo."),
            ],
        )
        .add(
            "settings_dm_only",
            &[
                ("en", "Settings live in a private chat with the bot - open a DM."),
                ("ru", "Настройки доступны в личке с ботом - открой ЛС."),
                ("uk", "Налаштування доступні в приваті з ботом - відкрий ЛС."),
                ("de", "Einstellungen gibt es im privaten Chat mit dem Bot - öffne einen DM."),
                ("es", "Los ajustes están en el chat privado con el bot - abre un MD."),
            ],
        )
        .add(
            "profile_unknown",
            &[
                ("en", "That user hasn't used the bot yet."),
                ("ru", "Этот пользователь ещё не пользовался ботом."),
                ("uk", "Цей користувач ще не користувався ботом."),
                ("de", "Dieser Nutzer hat den Bot noch nicht verwendet."),
                ("es", "Ese usuario aún no ha usado el bot."),
            ],
        )
        .add(
            "joke_unavailable",
            &[
                ("en", "The joke service is unreachable right now - try again in a bit."),
                ("ru", "Сервис шуток сейчас недоступен - попробуй чуть позже."),
                ("uk", "Сервіс жартів зараз недоступний - спробуй трохи пізніше."),
                ("de", "Der Witz-Dienst ist gerade nicht erreichbar - versuch es gleich nochmal."),
                ("es", "El servicio de chistes no está disponible ahora - inténtalo en un rato."),
            ],
        )
        // -- /server, /avatar ----------------------------------------------
        .add(
            "server_dm",
            &[
                ("en", "This is a private chat - there's no server to describe."),
                ("ru", "Это личный чат - здесь нет сервера."),
                ("uk", "Це приватний чат - тут немає сервера."),
                ("de", "Das ist ein privater Chat - hier gibt es keinen Server."),
                ("es", "Este es un chat privado - no hay servidor que describir."),
            ],
        )
        .add(
            "server_unavailable",
            &[
                ("en", "Couldn't fetch info for this chat right now."),
                ("ru", "Не удалось получить информацию об этом чате."),
                ("uk", "Не вдалося отримати інформацію про цей чат."),
                ("de", "Konnte gerade keine Infos zu diesem Chat abrufen."),
                ("es", "No se pudo obtener la información de este chat ahora."),
            ],
        )
        .add(
            "avatar_unavailable",
            &[
                ("en", "No avatar available here."),
                ("ru", "Аватар здесь недоступен."),
                ("uk", "Аватар тут недоступний."),
                ("de", "Hier ist kein Avatar verfügbar."),
                ("es", "No hay avatar disponible aquí."),
            ],
        )
        // -- /8ball ---------------------------------------------------------
        .add(
            "8ball_title",
            &[
                ("en", "\u{1F3B1} Magic 8-Ball"),
                ("ru", "\u{1F3B1} Магический шар"),
                ("uk", "\u{1F3B1} Магічна куля"),
                ("de", "\u{1F3B1} Magische Kugel"),
                ("es", "\u{1F3B1} Bola mágica"),
            ],
        )
        .add(
            "8ball_prompt",
            &[
                ("en", "Ask a question: `/8ball will it rain tomorrow?`"),
                ("ru", "Задай вопрос: `/8ball завтра будет дождь?`"),
                ("uk", "Постав питання: `/8ball завтра буде дощ?`"),
                ("de", "Stell eine Frage: `/8ball regnet es morgen?`"),
                ("es", "Haz una pregunta: `/8ball ¿lloverá mañana?`"),
            ],
        )
        .add(
            "8ball_question",
            &[
                ("en", "Question"),
                ("ru", "Вопрос"),
                ("uk", "Питання"),
                ("de", "Frage"),
                ("es", "Pregunta"),
            ],
        )
        .add(
            "8ball_answer",
            &[
                ("en", "Answer"),
                ("ru", "Ответ"),
                ("uk", "Відповідь"),
                ("de", "Antwort"),
                ("es", "Respuesta"),
            ],
        )
        .add(
            "8ball_answers",
            &[
                (
                    "en",
                    "Yes.|No.|Probably.|Absolutely not.|Ask again later.|Signs point to yes.|Outlook not so good.|Without a doubt.|Very doubtful.|It is certain.|Cannot predict now.|Most likely.",
                ),
                (
                    "ru",
                    "Да.|Нет.|Возможно.|Точно нет.|Спроси позже.|Всё указывает на да.|Так себе перспективы.|Без сомнений.|Очень сомнительно.|Это точно.|Пока не ясно.|Скорее всего.",
                ),
                (
                    "uk",
                    "Так.|Ні.|Можливо.|Точно ні.|Спитай пізніше.|Усе вказує на так.|Так собі перспективи.|Без сумніву.|Дуже сумнівно.|Це точно.|Поки не ясно.|Найпевніше.",
                ),
                (
                    "de",
                    "Ja.|Nein.|Wahrscheinlich.|Auf keinen Fall.|Frag später nochmal.|Sieht nach Ja aus.|Nicht so rosig.|Ohne Zweifel.|Sehr zweifelhaft.|Ganz sicher.|Kann ich noch nicht sagen.|Höchstwahrscheinlich.",
                ),
                (
                    "es",
                    "Sí.|No.|Probablemente.|En absoluto.|Pregunta más tarde.|Todo apunta a que sí.|No pinta bien.|Sin duda.|Muy dudoso.|Es seguro.|No puedo predecirlo ahora.|Lo más seguro que sí.",
                ),
            ],
        )
        // -- /coin, /roll -----------------------------------------------------
        .add(
            "coin_title",
            &[
                ("en", "{} Coin Flip"),
                ("ru", "{} Подброс монеты"),
                ("uk", "{} Підкидання монети"),
                ("de", "{} Münzwurf"),
                ("es", "{} Lanzamiento de moneda"),
            ],
        )
        .add(
            "coin_heads",
            &[
                ("en", "Heads"),
                ("ru", "Орёл"),
                ("uk", "Орел"),
                ("de", "Kopf"),
                ("es", "Cara"),
            ],
        )
        .add(
            "coin_tails",
            &[
                ("en", "Tails"),
                ("ru", "Решка"),
                ("uk", "Решка"),
                ("de", "Zahl"),
                ("es", "Cruz"),
            ],
        )
        .add(
            "roll_title",
            &[
                ("en", "\u{1F3B2} Dice Roll - {}d{}"),
                ("ru", "\u{1F3B2} Бросок кубиков - {}d{}"),
                ("uk", "\u{1F3B2} Кидок кубиків - {}d{}"),
                ("de", "\u{1F3B2} Würfelwurf - {}d{}"),
                ("es", "\u{1F3B2} Tirada de dados - {}d{}"),
            ],
        )
        .add(
            "roll_total",
            &[
                ("en", "Total"),
                ("ru", "Сумма"),
                ("uk", "Сума"),
                ("de", "Summe"),
                ("es", "Total"),
            ],
        )
        .add(
            "roll_rolls",
            &[
                ("en", "Rolls"),
                ("ru", "Броски"),
                ("uk", "Кидки"),
                ("de", "Würfe"),
                ("es", "Tiradas"),
            ],
        )
        .add(
            "roll_bad_title",
            &[
                ("en", "\u{2753} Bad Dice Spec"),
                ("ru", "\u{2753} Неверный формат"),
                ("uk", "\u{2753} Невірний формат"),
                ("de", "\u{2753} Ungültige Würfelangabe"),
                ("es", "\u{2753} Formato de dados inválido"),
            ],
        )
        .add(
            "roll_usage",
            &[
                ("en", "Usage: `/roll NdM` where N is 1..=100 and M is 2..=1000.\nExample: `/roll 3d6`."),
                ("ru", "Формат: `/roll NdM`, где N от 1 до 100, а M от 2 до 1000.\nПример: `/roll 3d6`."),
                ("uk", "Формат: `/roll NdM`, де N від 1 до 100, а M від 2 до 1000.\nПриклад: `/roll 3d6`."),
                ("de", "Nutzung: `/roll NdM`, N von 1 bis 100, M von 2 bis 1000.\nBeispiel: `/roll 3d6`."),
                ("es", "Uso: `/roll NdM`, N de 1 a 100 y M de 2 a 1000.\nEjemplo: `/roll 3d6`."),
            ],
        )
        // -- /cat, /joke, /emoji ----------------------------------------------
        .add(
            "cat_title",
            &[
                ("en", "\u{1F431} Random Cat"),
                ("ru", "\u{1F431} Случайный кот"),
                ("uk", "\u{1F431} Випадковий кіт"),
                ("de", "\u{1F431} Zufällige Katze"),
                ("es", "\u{1F431} Gato aleatorio"),
            ],
        )
        .add(
            "cat_body",
            &[
                ("en", "A random fluffy friend for you."),
                ("ru", "Случайный пушистый друг для тебя."),
                ("uk", "Випадковий пухнастий друг для тебе."),
                ("de", "Ein flauschiger Freund für dich."),
                ("es", "Un amigo peludo al azar para ti."),
            ],
        )
        .add(
            "joke_title",
            &[
                ("en", "\u{1F3AD} Joke"),
                ("ru", "\u{1F3AD} Шутка"),
                ("uk", "\u{1F3AD} Жарт"),
                ("de", "\u{1F3AD} Witz"),
                ("es", "\u{1F3AD} Chiste"),
            ],
        )
        .add(
            "joke_napping",
            &[
                ("en", "The joke service is napping - try /joke later."),
                ("ru", "Сервис шуток прилёг вздремнуть - попробуй /joke позже."),
                ("uk", "Сервіс жартів приліг подрімати - спробуй /joke пізніше."),
                ("de", "Der Witz-Dienst macht ein Nickerchen - versuch /joke später."),
                ("es", "El servicio de chistes está echando la siesta - prueba /joke más tarde."),
            ],
        )
        .add(
            "emoji_title",
            &[
                ("en", "{} Random Emoji"),
                ("ru", "{} Случайный эмодзи"),
                ("uk", "{} Випадковий емодзі"),
                ("de", "{} Zufälliges Emoji"),
                ("es", "{} Emoji aleatorio"),
            ],
        )
        .add(
            "emoji_body",
            &[
                ("en", "A random cute emoji for you."),
                ("ru", "Случайный милый эмодзи для тебя."),
                ("uk", "Випадковий милий емодзі для тебе."),
                ("de", "Ein zufälliges süßes Emoji für dich."),
                ("es", "Un emoji bonito al azar para ti."),
            ],
        )
        // -- /choose, /reverse ------------------------------------------------
        .add(
            "choose_title",
            &[
                ("en", "\u{1F3B2} Choose"),
                ("ru", "\u{1F3B2} Выбор"),
                ("uk", "\u{1F3B2} Вибір"),
                ("de", "\u{1F3B2} Auswahl"),
                ("es", "\u{1F3B2} Elegir"),
            ],
        )
        .add(
            "choose_usage",
            &[
                ("en", "Give me a few options, comma- or space-separated.\nExample: `/choose pizza, sushi, burger`"),
                ("ru", "Дай несколько вариантов через запятую или пробел.\nПример: `/choose пицца, суши, бургер`"),
                ("uk", "Дай кілька варіантів через кому або пробіл.\nПриклад: `/choose піца, суші, бургер`"),
                ("de", "Gib mir ein paar Optionen, durch Komma oder Leerzeichen getrennt.\nBeispiel: `/choose Pizza, Sushi, Burger`"),
                ("es", "Dame varias opciones separadas por comas o espacios.\nEjemplo: `/choose pizza, sushi, hamburguesa`"),
            ],
        )
        .add(
            "choose_need_two",
            &[
                ("en", "Need at least two options to choose from."),
                ("ru", "Нужно хотя бы два варианта на выбор."),
                ("uk", "Потрібно щонайменше два варіанти на вибір."),
                ("de", "Es braucht mindestens zwei Optionen zur Auswahl."),
                ("es", "Hacen falta al menos dos opciones para elegir."),
            ],
        )
        .add(
            "choose_options",
            &[
                ("en", "Options"),
                ("ru", "Варианты"),
                ("uk", "Варіанти"),
                ("de", "Optionen"),
                ("es", "Opciones"),
            ],
        )
        .add(
            "choose_pick",
            &[
                ("en", "Pick"),
                ("ru", "Выбор"),
                ("uk", "Вибір"),
                ("de", "Wahl"),
                ("es", "Elección"),
            ],
        )
        .add(
            "reverse_title",
            &[
                ("en", "\u{1F501} Reverse"),
                ("ru", "\u{1F501} Наоборот"),
                ("uk", "\u{1F501} Навпаки"),
                ("de", "\u{1F501} Umkehren"),
                ("es", "\u{1F501} Invertir"),
            ],
        )
        .add(
            "reverse_usage",
            &[
                ("en", "Give me some text: `/reverse hello`"),
                ("ru", "Дай какой-нибудь текст: `/reverse привет`"),
                ("uk", "Дай якийсь текст: `/reverse привіт`"),
                ("de", "Gib mir einen Text: `/reverse hallo`"),
                ("es", "Dame algún texto: `/reverse hola`"),
            ],
        )
        .add(
            "reverse_input",
            &[
                ("en", "Input"),
                ("ru", "Ввод"),
                ("uk", "Ввід"),
                ("de", "Eingabe"),
                ("es", "Entrada"),
            ],
        )
        .add(
            "reverse_output",
            &[
                ("en", "Output"),
                ("ru", "Результат"),
                ("uk", "Результат"),
                ("de", "Ausgabe"),
                ("es", "Salida"),
            ],
        )
        // -- /menu --------------------------------------------------------------
        .add(
            "menu_title",
            &[
                ("en", "\u{1F4CB} Menu"),
                ("ru", "\u{1F4CB} Меню"),
                ("uk", "\u{1F4CB} Меню"),
                ("de", "\u{1F4CB} Menü"),
                ("es", "\u{1F4CB} Menú"),
            ],
        )
        .add(
            "menu_body",
            &[
                ("en", "Pick something to try:"),
                ("ru", "Выбери, что попробовать:"),
                ("uk", "Обери, що спробувати:"),
                ("de", "Such dir etwas aus:"),
                ("es", "Elige algo para probar:"),
            ],
        )
        .add(
            "menu_footer",
            &[
                ("en", "Pick another action below"),
                ("ru", "Выбери ещё что-нибудь ниже"),
                ("uk", "Обери ще щось нижче"),
                ("de", "Wähl unten eine weitere Aktion"),
                ("es", "Elige otra acción abajo"),
            ],
        )
        .add(
            "menu_coin_result",
            &[
                ("en", "Result: **{}**"),
                ("ru", "Результат: **{}**"),
                ("uk", "Результат: **{}**"),
                ("de", "Ergebnis: **{}**"),
                ("es", "Resultado: **{}**"),
            ],
        )
        .add(
            "menu_roll_title",
            &[
                ("en", "\u{1F3B2} Dice Roll"),
                ("ru", "\u{1F3B2} Бросок кубика"),
                ("uk", "\u{1F3B2} Кидок кубика"),
                ("de", "\u{1F3B2} Würfelwurf"),
                ("es", "\u{1F3B2} Tirada de dados"),
            ],
        )
        .add(
            "menu_time_title",
            &[
                ("en", "\u{1F550} Time (UTC)"),
                ("ru", "\u{1F550} Время (UTC)"),
                ("uk", "\u{1F550} Час (UTC)"),
                ("de", "\u{1F550} Zeit (UTC)"),
                ("es", "\u{1F550} Hora (UTC)"),
            ],
        )
        .add(
            "menu_unknown_action",
            &[
                ("en", "Unknown action"),
                ("ru", "Неизвестное действие"),
                ("uk", "Невідома дія"),
                ("de", "Unbekannte Aktion"),
                ("es", "Acción desconocida"),
            ],
        )
        .add(
            "menu_rps_body",
            &[
                ("en", "You: **{}** · Bot: **{}**\n**{}**\nFull game: /rps"),
                ("ru", "Ты: **{}** · Бот: **{}**\n**{}**\nПолная игра: /rps"),
                ("uk", "Ти: **{}** · Бот: **{}**\n**{}**\nПовна гра: /rps"),
                ("de", "Du: **{}** · Bot: **{}**\n**{}**\nGanzes Spiel: /rps"),
                ("es", "Tú: **{}** · Bot: **{}**\n**{}**\nJuego completo: /rps"),
            ],
        )
        // -- /rps -----------------------------------------------------------------
        .add(
            "rps_title",
            &[
                ("en", "\u{1FAA8} Rock Paper Scissors"),
                ("ru", "\u{1FAA8} Камень, ножницы, бумага"),
                ("uk", "\u{1FAA8} Камінь, ножиці, папір"),
                ("de", "\u{1FAA8} Schere, Stein, Papier"),
                ("es", "\u{1FAA8} Piedra, papel o tijera"),
            ],
        )
        .add(
            "rps_prompt",
            &[
                ("en", "Make your move:"),
                ("ru", "Твой ход:"),
                ("uk", "Твій хід:"),
                ("de", "Dein Zug:"),
                ("es", "Haz tu jugada:"),
            ],
        )
        .add(
            "rps_rock",
            &[
                ("en", "Rock"),
                ("ru", "Камень"),
                ("uk", "Камінь"),
                ("de", "Stein"),
                ("es", "Piedra"),
            ],
        )
        .add(
            "rps_paper",
            &[
                ("en", "Paper"),
                ("ru", "Бумага"),
                ("uk", "Папір"),
                ("de", "Papier"),
                ("es", "Papel"),
            ],
        )
        .add(
            "rps_scissors",
            &[
                ("en", "Scissors"),
                ("ru", "Ножницы"),
                ("uk", "Ножиці"),
                ("de", "Schere"),
                ("es", "Tijera"),
            ],
        )
        .add(
            "rps_you",
            &[
                ("en", "You"),
                ("ru", "Ты"),
                ("uk", "Ти"),
                ("de", "Du"),
                ("es", "Tú"),
            ],
        )
        .add(
            "rps_bot",
            &[
                ("en", "Bot"),
                ("ru", "Бот"),
                ("uk", "Бот"),
                ("de", "Bot"),
                ("es", "Bot"),
            ],
        )
        .add(
            "rps_win",
            &[
                ("en", "You win! \u{1F389}"),
                ("ru", "Ты выиграл! \u{1F389}"),
                ("uk", "Ти виграв! \u{1F389}"),
                ("de", "Du gewinnst! \u{1F389}"),
                ("es", "¡Ganaste! \u{1F389}"),
            ],
        )
        .add(
            "rps_lose",
            &[
                ("en", "You lose \u{1F480}"),
                ("ru", "Ты проиграл \u{1F480}"),
                ("uk", "Ти програв \u{1F480}"),
                ("de", "Du verlierst \u{1F480}"),
                ("es", "Perdiste \u{1F480}"),
            ],
        )
        .add(
            "rps_draw",
            &[
                ("en", "Draw \u{1F91D}"),
                ("ru", "Ничья \u{1F91D}"),
                ("uk", "Нічия \u{1F91D}"),
                ("de", "Unentschieden \u{1F91D}"),
                ("es", "Empate \u{1F91D}"),
            ],
        )
        // -- /daily ---------------------------------------------------------
        .add(
            "daily_title",
            &[
                ("en", "🎁 Daily Reward"),
                ("ru", "🎁 Ежедневная награда"),
                ("uk", "🎁 Щоденна нагорода"),
                ("de", "🎁 Tägliche Belohnung"),
                ("es", "🎁 Recompensa diaria"),
            ],
        )
        .add(
            "daily_too_soon_title",
            &[
                ("en", "⏳ Too Soon"),
                ("ru", "⏳ Рано"),
                ("uk", "⏳ Зарано"),
                ("de", "⏳ Zu früh"),
                ("es", "⏳ Demasiado pronto"),
            ],
        )
        .add(
            "daily_too_soon_body",
            &[
                ("en", "Next reward in {}."),
                ("ru", "Следующая награда через {}."),
                ("uk", "Наступна нагорода через {}."),
                ("de", "Nächste Belohnung in {}."),
                ("es", "Próxima recompensa en {}."),
            ],
        )
        .add(
            "daily_streak",
            &[
                ("en", "Streak"),
                ("ru", "Стрик"),
                ("uk", "Серія"),
                ("de", "Serie"),
                ("es", "Racha"),
            ],
        )
        .add(
            "daily_reward",
            &[
                ("en", "Reward"),
                ("ru", "Награда"),
                ("uk", "Нагорода"),
                ("de", "Belohnung"),
                ("es", "Recompensa"),
            ],
        )
        .add(
            "daily_footer",
            &[
                ("en", "Come back tomorrow to keep the streak."),
                ("ru", "Возвращайся завтра, чтобы не потерять стрик."),
                ("uk", "Повертайся завтра, щоб не втратити серію."),
                ("de", "Komm morgen wieder, um die Serie zu halten."),
                ("es", "Vuelve mañana para mantener la racha."),
            ],
        )
        .add(
            "daily_footer_weekly",
            &[
                ("en", "Weekly bonus! Keep it up."),
                ("ru", "Недельный бонус! Так держать."),
                ("uk", "Тижневий бонус! Так тримати."),
                ("de", "Wochenbonus! Weiter so."),
                ("es", "¡Bono semanal! Sigue así."),
            ],
        )
        // -- shared field labels -------------------------------------------
        .add(
            "coins",
            &[
                ("en", "Coins"),
                ("ru", "Монеты"),
                ("uk", "Монети"),
                ("de", "Münzen"),
                ("es", "Monedas"),
            ],
        )
        // -- /profile ---------------------------------------------------------
        .add(
            "econ_profile_title",
            &[
                ("en", "\u{1F464} Profile"),
                ("ru", "\u{1F464} Профиль"),
                ("uk", "\u{1F464} Профіль"),
                ("de", "\u{1F464} Profil"),
                ("es", "\u{1F464} Perfil"),
            ],
        )
        .add(
            "econ_profile_level",
            &[
                ("en", "Level"),
                ("ru", "Уровень"),
                ("uk", "Рівень"),
                ("de", "Level"),
                ("es", "Nivel"),
            ],
        )
        .add(
            "econ_profile_xp",
            &[
                ("en", "XP"),
                ("ru", "Опыт"),
                ("uk", "Досвід"),
                ("de", "XP"),
                ("es", "XP"),
            ],
        )
        .add(
            "econ_profile_lang",
            &[
                ("en", "Language"),
                ("ru", "Язык"),
                ("uk", "Мова"),
                ("de", "Sprache"),
                ("es", "Idioma"),
            ],
        )
        .add(
            "econ_profile_platforms",
            &[
                ("en", "Platforms"),
                ("ru", "Платформы"),
                ("uk", "Платформи"),
                ("de", "Plattformen"),
                ("es", "Plataformas"),
            ],
        )
        .add(
            "econ_profile_next",
            &[
                ("en", "to level {}"),
                ("ru", "до уровня {}"),
                ("uk", "до рівня {}"),
                ("de", "bis Level {}"),
                ("es", "para el nivel {}"),
            ],
        )
        .add(
            "econ_profile_badges",
            &[
                ("en", "Badges"),
                ("ru", "Достижения"),
                ("uk", "Досягнення"),
                ("de", "Abzeichen"),
                ("es", "Insignias"),
            ],
        )
        .add(
            "econ_profile_account",
            &[
                ("en", "\u{1F511} Account"),
                ("ru", "\u{1F511} Аккаунт"),
                ("uk", "\u{1F511} Акаунт"),
                ("de", "\u{1F511} Konto"),
                ("es", "\u{1F511} Cuenta"),
            ],
        )
        .add(
            "econ_profile_id",
            &[
                ("en", "{} ID"),
                ("ru", "{} ID"),
                ("uk", "{} ID"),
                ("de", "{}-ID"),
                ("es", "ID de {}"),
            ],
        )
        // -- /achievements ----------------------------------------------------
        .add(
            "ach_list_title",
            &[
                ("en", "\u{1F3C5} Achievements"),
                ("ru", "\u{1F3C5} Достижения"),
                ("uk", "\u{1F3C5} Досягнення"),
                ("de", "\u{1F3C5} Erfolge"),
                ("es", "\u{1F3C5} Logros"),
            ],
        )
        .add(
            "ach_list_title_other",
            &[
                ("en", "\u{1F3C5} Achievements - {}"),
                ("ru", "\u{1F3C5} Достижения - {}"),
                ("uk", "\u{1F3C5} Досягнення - {}"),
                ("de", "\u{1F3C5} Erfolge - {}"),
                ("es", "\u{1F3C5} Logros - {}"),
            ],
        )
        .add(
            "ach_first_daily",
            &[
                ("en", "Early Bird"),
                ("ru", "Ранняя пташка"),
                ("uk", "Рання пташка"),
                ("de", "Früher Vogel"),
                ("es", "Madrugador"),
            ],
        )
        .add(
            "ach_streak_7",
            &[
                ("en", "Week Streak"),
                ("ru", "Неделя подряд"),
                ("uk", "Тиждень поспіль"),
                ("de", "Wochenserie"),
                ("es", "Racha semanal"),
            ],
        )
        .add(
            "ach_high_roller",
            &[
                ("en", "High Roller"),
                ("ru", "Крупная игра"),
                ("uk", "Велика гра"),
                ("de", "Zocker"),
                ("es", "Gran apostador"),
            ],
        )
        .add(
            "ach_big_spender",
            &[
                ("en", "Big Spender"),
                ("ru", "Транжира"),
                ("uk", "Марнотратник"),
                ("de", "Verschwender"),
                ("es", "Derrochador"),
            ],
        )
        // -- /shop + /buy -------------------------------------------------------
        .add(
            "shop_title",
            &[
                ("en", "\u{1F6D2} Shop"),
                ("ru", "\u{1F6D2} Магазин"),
                ("uk", "\u{1F6D2} Крамниця"),
                ("de", "\u{1F6D2} Shop"),
                ("es", "\u{1F6D2} Tienda"),
            ],
        )
        .add(
            "shop_balance",
            &[
                ("en", "Balance"),
                ("ru", "Баланс"),
                ("uk", "Баланс"),
                ("de", "Guthaben"),
                ("es", "Saldo"),
            ],
        )
        .add(
            "shop_footer",
            &[
                ("en", "Tap an item, or /buy <id>"),
                ("ru", "Нажми на товар или /buy <id>"),
                ("uk", "Натисни на товар або /buy <id>"),
                ("de", "Tippe einen Artikel an oder /buy <id>"),
                ("es", "Toca un artículo o /buy <id>"),
            ],
        )
        .add(
            "buy_title",
            &[
                ("en", "\u{1F6D2} Buy"),
                ("ru", "\u{1F6D2} Покупка"),
                ("uk", "\u{1F6D2} Покупка"),
                ("de", "\u{1F6D2} Kaufen"),
                ("es", "\u{1F6D2} Comprar"),
            ],
        )
        .add(
            "buy_usage",
            &[
                ("en", "Name an item id: `/buy title_legend`. See /shop for the list."),
                ("ru", "Укажи id товара: `/buy title_legend`. Список: /shop"),
                ("uk", "Вкажи id товару: `/buy title_legend`. Список: /shop"),
                ("de", "Gib eine Artikel-Id an: `/buy title_legend`. Liste: /shop"),
                ("es", "Indica el id del artículo: `/buy title_legend`. Lista: /shop"),
            ],
        )
        .add(
            "shop_unknown_title",
            &[
                ("en", "\u{2753} Unknown Item"),
                ("ru", "\u{2753} Неизвестный товар"),
                ("uk", "\u{2753} Невідомий товар"),
                ("de", "\u{2753} Unbekannter Artikel"),
                ("es", "\u{2753} Artículo desconocido"),
            ],
        )
        .add(
            "shop_unknown_body",
            &[
                ("en", "No such item. Check /shop."),
                ("ru", "Нет такого товара. Загляни в /shop."),
                ("uk", "Немає такого товару. Заглянь у /shop."),
                ("de", "So einen Artikel gibt es nicht. Schau in /shop."),
                ("es", "No existe ese artículo. Mira /shop."),
            ],
        )
        .add(
            "shop_not_enough_title",
            &[
                ("en", "\u{1FA99} Not Enough Coins"),
                ("ru", "\u{1FA99} Не хватает монет"),
                ("uk", "\u{1FA99} Не вистачає монет"),
                ("de", "\u{1FA99} Zu wenig Münzen"),
                ("es", "\u{1FA99} Faltan monedas"),
            ],
        )
        .add(
            "shop_not_enough_body",
            &[
                ("en", "Costs {}, you have {}."),
                ("ru", "Нужно {}, а у тебя {}."),
                ("uk", "Потрібно {}, а в тебе {}."),
                ("de", "Kostet {}, du hast {}."),
                ("es", "Cuesta {}, tienes {}."),
            ],
        )
        .add(
            "shop_purchased_title",
            &[
                ("en", "\u{2705} Purchased"),
                ("ru", "\u{2705} Куплено"),
                ("uk", "\u{2705} Куплено"),
                ("de", "\u{2705} Gekauft"),
                ("es", "\u{2705} Comprado"),
            ],
        )
        .add(
            "shop_purchased_body",
            &[
                ("en", "**{}** is yours."),
                ("ru", "**{}** теперь твой."),
                ("uk", "**{}** тепер твій."),
                ("de", "**{}** gehört jetzt dir."),
                ("es", "**{}** ya es tuyo."),
            ],
        )
        .add(
            "shop_purchased_footer",
            &[
                ("en", "Check it out on /profile."),
                ("ru", "Загляни в /profile."),
                ("uk", "Заглянь у /profile."),
                ("de", "Schau in /profile vorbei."),
                ("es", "Échale un vistazo en /profile."),
            ],
        )
        .add(
            "shop_item_title_novice",
            &[
                ("en", "Title: Novice"),
                ("ru", "Титул: Новичок"),
                ("uk", "Титул: Новачок"),
                ("de", "Titel: Neuling"),
                ("es", "Título: Novato"),
            ],
        )
        .add(
            "shop_item_title_regular",
            &[
                ("en", "Title: Regular"),
                ("ru", "Титул: Завсегдатай"),
                ("uk", "Титул: Завсідник"),
                ("de", "Titel: Stammgast"),
                ("es", "Título: Habitual"),
            ],
        )
        .add(
            "shop_item_title_legend",
            &[
                ("en", "Title: Legend"),
                ("ru", "Титул: Легенда"),
                ("uk", "Титул: Легенда"),
                ("de", "Titel: Legende"),
                ("es", "Título: Leyenda"),
            ],
        )
        .add(
            "shop_item_color_teal",
            &[
                ("en", "Profile colour: Teal"),
                ("ru", "Цвет профиля: Бирюза"),
                ("uk", "Колір профілю: Бірюза"),
                ("de", "Profilfarbe: Türkis"),
                ("es", "Color de perfil: Turquesa"),
            ],
        )
        .add(
            "shop_item_color_gold",
            &[
                ("en", "Profile colour: Gold"),
                ("ru", "Цвет профиля: Золото"),
                ("uk", "Колір профілю: Золото"),
                ("de", "Profilfarbe: Gold"),
                ("es", "Color de perfil: Oro"),
            ],
        )
        .add(
            "shop_item_color_crimson",
            &[
                ("en", "Profile colour: Crimson"),
                ("ru", "Цвет профиля: Багрянец"),
                ("uk", "Колір профілю: Багрянець"),
                ("de", "Profilfarbe: Karmesin"),
                ("es", "Color de perfil: Carmesí"),
            ],
        )
        // -- /leaderboard + /rank ------------------------------------------
        .add(
            "lb_title",
            &[
                ("en", "\u{1F3C6} Leaderboard"),
                ("ru", "\u{1F3C6} Рейтинг"),
                ("uk", "\u{1F3C6} Рейтинг"),
                ("de", "\u{1F3C6} Bestenliste"),
                ("es", "\u{1F3C6} Clasificación"),
            ],
        )
        .add(
            "lb_footer_by_xp",
            &[
                ("en", "/leaderboard - by XP"),
                ("ru", "/leaderboard - по опыту"),
                ("uk", "/leaderboard - за досвідом"),
                ("de", "/leaderboard - nach XP"),
                ("es", "/leaderboard - por XP"),
            ],
        )
        .add(
            "lb_footer_by_coins",
            &[
                ("en", "/leaderboard coins - by coins"),
                ("ru", "/leaderboard coins - по монетам"),
                ("uk", "/leaderboard coins - за монетами"),
                ("de", "/leaderboard coins - nach Münzen"),
                ("es", "/leaderboard coins - por monedas"),
            ],
        )
        .add(
            "lb_empty",
            &[
                ("en", "Nothing here yet. Chat a bit to earn some XP."),
                ("ru", "Пока пусто. Пообщайся в чате, чтобы заработать опыт."),
                (
                    "uk",
                    "Поки порожньо. Спілкуйся в чаті, щоб заробити досвід.",
                ),
                ("de", "Noch nichts hier. Schreib etwas, um XP zu sammeln."),
                ("es", "Aún no hay nada. Chatea un poco para ganar XP."),
            ],
        )
        .add(
            "lb_xp_title",
            &[
                ("en", "🏆 XP Leaderboard"),
                ("ru", "🏆 Топ по опыту"),
                ("uk", "🏆 Топ за досвідом"),
                ("de", "🏆 XP-Bestenliste"),
                ("es", "🏆 Clasificación de XP"),
            ],
        )
        .add(
            "lb_coins_title",
            &[
                ("en", "🏆 Coin Leaderboard"),
                ("ru", "🏆 Топ по монетам"),
                ("uk", "🏆 Топ за монетами"),
                ("de", "🏆 Münz-Bestenliste"),
                ("es", "🏆 Clasificación de monedas"),
            ],
        )
        .add(
            "rank_title",
            &[
                ("en", "📈 Your Rank"),
                ("ru", "📈 Твоё место"),
                ("uk", "📈 Твоє місце"),
                ("de", "📈 Dein Rang"),
                ("es", "📈 Tu posición"),
            ],
        )
        .add(
            "rank_none",
            &[
                ("en", "You're not ranked yet. Chat a bit first."),
                ("ru", "Ты ещё не в рейтинге. Пообщайся в чате."),
                ("uk", "Ти ще не в рейтингу. Спершу спілкуйся в чаті."),
                ("de", "Du bist noch nicht platziert. Schreib erst etwas."),
                ("es", "Aún no estás clasificado. Chatea un poco primero."),
            ],
        )
        .add(
            "rank_by_xp",
            &[
                ("en", "By XP"),
                ("ru", "По опыту"),
                ("uk", "За досвідом"),
                ("de", "Nach XP"),
                ("es", "Por XP"),
            ],
        )
        .add(
            "rank_by_coins",
            &[
                ("en", "By Coins"),
                ("ru", "По монетам"),
                ("uk", "За монетами"),
                ("de", "Nach Münzen"),
                ("es", "Por monedas"),
            ],
        )
        // -- /gamble --------------------------------------------------------
        .add(
            "gamble_title",
            &[
                ("en", "\u{1F3B0} Gamble"),
                ("ru", "\u{1F3B0} Ставка"),
                ("uk", "\u{1F3B0} Ставка"),
                ("de", "\u{1F3B0} Glücksspiel"),
                ("es", "\u{1F3B0} Apuesta"),
            ],
        )
        .add(
            "gamble_slow_title",
            &[
                ("en", "\u{1F3B0} Slow Down"),
                ("ru", "\u{1F3B0} Не так быстро"),
                ("uk", "\u{1F3B0} Не так швидко"),
                ("de", "\u{1F3B0} Langsamer"),
                ("es", "\u{1F3B0} Más despacio"),
            ],
        )
        .add(
            "gamble_not_enough_title",
            &[
                ("en", "\u{1FA99} Not Enough"),
                ("ru", "\u{1FA99} Не хватает"),
                ("uk", "\u{1FA99} Не вистачає"),
                ("de", "\u{1FA99} Zu wenig"),
                ("es", "\u{1FA99} No alcanza"),
            ],
        )
        .add(
            "gamble_win_title",
            &[
                ("en", "🎉 You Won!"),
                ("ru", "🎉 Выигрыш!"),
                ("uk", "🎉 Виграш!"),
                ("de", "🎉 Gewonnen!"),
                ("es", "🎉 ¡Ganaste!"),
            ],
        )
        .add(
            "gamble_lose_title",
            &[
                ("en", "💀 You Lost"),
                ("ru", "💀 Мимо"),
                ("uk", "💀 Повз"),
                ("de", "💀 Verloren"),
                ("es", "💀 Perdiste"),
            ],
        )
        .add(
            "gamble_prompt",
            &[
                ("en", "Bet some coins: `/gamble 20`."),
                ("ru", "Ставка монетами: `/gamble 20`."),
                ("uk", "Ставка монетами: `/gamble 20`."),
                ("de", "Setze Münzen: `/gamble 20`."),
                ("es", "Apuesta monedas: `/gamble 20`."),
            ],
        )
        .add(
            "gamble_wait",
            &[
                ("en", "Wait {}."),
                ("ru", "Подожди {}."),
                ("uk", "Зачекай {}."),
                ("de", "Warte {}."),
                ("es", "Espera {}."),
            ],
        )
        .add(
            "gamble_short",
            &[
                ("en", "You only have {}."),
                ("ru", "У тебя только {}."),
                ("uk", "У тебе лише {}."),
                ("de", "Du hast nur {}."),
                ("es", "Solo tienes {}."),
            ],
        )
        .add(
            "gamble_max",
            &[
                ("en", "Max bet is {} coins."),
                ("ru", "Максимальная ставка - {} монет."),
                ("uk", "Максимальна ставка - {} монет."),
                ("de", "Höchsteinsatz sind {} Münzen."),
                ("es", "La apuesta máxima es {} monedas."),
            ],
        )
        .add(
            "gamble_win_body",
            &[
                ("en", "+{} coins. Now: {}."),
                ("ru", "+{} монет. Теперь: {}."),
                ("uk", "+{} монет. Тепер: {}."),
                ("de", "+{} Münzen. Jetzt: {}."),
                ("es", "+{} monedas. Ahora: {}."),
            ],
        )
        .add(
            "gamble_lose_body",
            &[
                ("en", "-{} coins. Left: {}."),
                ("ru", "-{} монет. Осталось: {}."),
                ("uk", "-{} монет. Залишилось: {}."),
                ("de", "-{} Münzen. Übrig: {}."),
                ("es", "-{} monedas. Quedan: {}."),
            ],
        )
        // -- /give ----------------------------------------------------------
        .add(
            "give_title",
            &[
                ("en", "\u{1FA99} Give"),
                ("ru", "\u{1FA99} Перевод"),
                ("uk", "\u{1FA99} Переказ"),
                ("de", "\u{1FA99} Überweisung"),
                ("es", "\u{1FA99} Envío"),
            ],
        )
        .add(
            "give_sent_title",
            &[
                ("en", "🪙 Coins Sent"),
                ("ru", "🪙 Перевод отправлен"),
                ("uk", "🪙 Переказ надіслано"),
                ("de", "🪙 Münzen gesendet"),
                ("es", "🪙 Monedas enviadas"),
            ],
        )
        .add(
            "give_sent_body",
            &[
                ("en", "Sent **{}**. You have {} left."),
                ("ru", "Отправлено **{}**. Осталось: {}."),
                ("uk", "Надіслано **{}**. Залишилось: {}."),
                ("de", "**{}** gesendet. Es bleiben {}."),
                ("es", "Enviadas **{}**. Te quedan {}."),
            ],
        )
        .add(
            "give_usage",
            &[
                (
                    "en",
                    "Usage: `/give <id> <amount>`. Works within the same platform.",
                ),
                (
                    "ru",
                    "Формат: `/give <id> <сумма>`. Работает в пределах одной платформы.",
                ),
                (
                    "uk",
                    "Формат: `/give <id> <сума>`. Працює в межах однієї платформи.",
                ),
                (
                    "de",
                    "Nutzung: `/give <id> <Betrag>`. Nur innerhalb einer Plattform.",
                ),
                (
                    "es",
                    "Uso: `/give <id> <cantidad>`. Funciona dentro de la misma plataforma.",
                ),
            ],
        )
        .add(
            "give_self",
            &[
                ("en", "You can't send coins to yourself."),
                ("ru", "Себе переводить нельзя."),
                ("uk", "Собі переказувати не можна."),
                ("de", "Du kannst dir selbst keine Münzen senden."),
                ("es", "No puedes enviarte monedas a ti mismo."),
            ],
        )
        .add(
            "give_failed",
            &[
                ("en", "Not enough coins for that transfer."),
                ("ru", "Недостаточно монет для перевода."),
                ("uk", "Недостатньо монет для переказу."),
                ("de", "Nicht genug Münzen für diese Überweisung."),
                (
                    "es",
                    "No tienes monedas suficientes para esa transferencia.",
                ),
            ],
        )
        .add(
            "give_unknown",
            &[
                ("en", "That user hasn't used the bot yet, so there's nowhere to send to."),
                ("ru", "Этот пользователь ещё не пользовался ботом - отправлять некуда."),
                ("uk", "Цей користувач ще не користувався ботом - надсилати нікуди."),
                ("de", "Dieser Nutzer hat den Bot noch nicht verwendet - es gibt kein Ziel."),
                ("es", "Ese usuario aún no ha usado el bot, así que no hay adónde enviar."),
            ],
        )
        // -- /poll ----------------------------------------------------------
        .add(
            "poll_usage",
            &[
                ("en", "Usage: `/poll Question? | Option A | Option B`\nUp to 5 options."),
                ("ru", "Формат: `/poll Вопрос? | Вариант А | Вариант Б`\nДо 5 вариантов."),
                ("uk", "Формат: `/poll Питання? | Варіант А | Варіант Б`\nДо 5 варіантів."),
                ("de", "Nutzung: `/poll Frage? | Option A | Option B`\nBis zu 5 Optionen."),
                ("es", "Uso: `/poll ¿Pregunta? | Opción A | Opción B`\nHasta 5 opciones."),
            ],
        )
        .add(
            "poll_footer",
            &[
                ("en", "Tap a number to vote"),
                ("ru", "Нажми на цифру, чтобы проголосовать"),
                ("uk", "Натисни на цифру, щоб проголосувати"),
                ("de", "Tippe eine Zahl an, um abzustimmen"),
                ("es", "Toca un número para votar"),
            ],
        )
        .add(
            "poll_already",
            &[
                ("en", "You already voted for that option."),
                ("ru", "Ты уже проголосовал за этот вариант."),
                ("uk", "Ти вже проголосував за цей варіант."),
                ("de", "Du hast bereits für diese Option gestimmt."),
                ("es", "Ya votaste por esa opción."),
            ],
        )
        .add(
            "poll_gone",
            &[
                ("en", "This poll is no longer available."),
                ("ru", "Этот опрос больше недоступен."),
                ("uk", "Це опитування більше недоступне."),
                ("de", "Diese Umfrage ist nicht mehr verfügbar."),
                ("es", "Esta encuesta ya no está disponible."),
            ],
        )
        .add(
            "poll_title",
            &[
                ("en", "\u{1F4CA} Poll"),
                ("ru", "\u{1F4CA} Опрос"),
                ("uk", "\u{1F4CA} Опитування"),
                ("de", "\u{1F4CA} Umfrage"),
                ("es", "\u{1F4CA} Encuesta"),
            ],
        )
        .add(
            "poll_too_many",
            &[
                ("en", "Too many options - a poll takes up to {}."),
                ("ru", "Слишком много вариантов - в опросе их может быть до {}."),
                ("uk", "Забагато варіантів - в опитуванні їх може бути до {}."),
                ("de", "Zu viele Optionen - eine Umfrage erlaubt bis zu {}."),
                ("es", "Demasiadas opciones - una encuesta admite hasta {}."),
            ],
        )
        .add(
            "poll_vote_one",
            &[
                ("en", "1 vote"),
                ("ru", "1 голос"),
                ("uk", "1 голос"),
                ("de", "1 Stimme"),
                ("es", "1 voto"),
            ],
        )
        .add(
            "poll_votes",
            &[
                ("en", "{} votes"),
                ("ru", "{} голосов"),
                ("uk", "{} голосів"),
                ("de", "{} Stimmen"),
                ("es", "{} votos"),
            ],
        )
        // -- /remind --------------------------------------------------------
        .add(
            "remind_title",
            &[
                ("en", "\u{23F0} Remind"),
                ("ru", "\u{23F0} Напоминание"),
                ("uk", "\u{23F0} Нагадування"),
                ("de", "\u{23F0} Erinnerung"),
                ("es", "\u{23F0} Recordatorio"),
            ],
        )
        .add(
            "remind_usage",
            &[
                ("en", "Usage: `/remind 10m call mom`. Units: s, m, h."),
                ("ru", "Формат: `/remind 10m позвонить маме`. Единицы: s, m, h."),
                ("uk", "Формат: `/remind 10m зателефонувати мамі`. Одиниці: s, m, h."),
                ("de", "Nutzung: `/remind 10m Mama anrufen`. Einheiten: s, m, h."),
                ("es", "Uso: `/remind 10m llamar a mamá`. Unidades: s, m, h."),
            ],
        )
        .add(
            "remind_range",
            &[
                ("en", "Delay must be between 1 second and 24 hours."),
                ("ru", "Задержка должна быть от 1 секунды до 24 часов."),
                ("uk", "Затримка має бути від 1 секунди до 24 годин."),
                ("de", "Die Verzögerung muss zwischen 1 Sekunde und 24 Stunden liegen."),
                ("es", "El retraso debe estar entre 1 segundo y 24 horas."),
            ],
        )
        .add(
            "remind_set_title",
            &[
                ("en", "⏰ Reminder Set"),
                ("ru", "⏰ Напоминание поставлено"),
                ("uk", "⏰ Нагадування встановлено"),
                ("de", "⏰ Erinnerung gesetzt"),
                ("es", "⏰ Recordatorio creado"),
            ],
        )
        .add(
            "remind_set_body",
            &[
                ("en", "I'll ping you in {}."),
                ("ru", "Напомню через {}."),
                ("uk", "Нагадаю через {}."),
                ("de", "Ich melde mich in {}."),
                ("es", "Te aviso en {}."),
            ],
        )
        .add(
            "remind_ping_title",
            &[
                ("en", "⏰ Reminder"),
                ("ru", "⏰ Напоминание"),
                ("uk", "⏰ Нагадування"),
                ("de", "⏰ Erinnerung"),
                ("es", "⏰ Recordatorio"),
            ],
        )
        .add(
            "remind_none",
            &[
                ("en", "No pending reminders. Set one: `/remind 10m text`."),
                ("ru", "Нет активных напоминаний. Поставь: `/remind 10m текст`."),
                ("uk", "Немає активних нагадувань. Постав: `/remind 10m текст`."),
                ("de", "Keine anstehenden Erinnerungen. Setze eine: `/remind 10m Text`."),
                ("es", "No hay recordatorios pendientes. Crea uno: `/remind 10m texto`."),
            ],
        )
        .add(
            "remind_menu_hint",
            &[
                ("en", "Tap a number to delete that reminder"),
                ("ru", "Нажми на номер, чтобы удалить напоминание"),
                ("uk", "Натисни на номер, щоб видалити нагадування"),
                ("de", "Tippe eine Nummer an, um die Erinnerung zu löschen"),
                ("es", "Toca un número para borrar ese recordatorio"),
            ],
        )
        // -- /time ------------------------------------------------------------
        .add(
            "time_title",
            &[
                ("en", "\u{1F550} Time"),
                ("ru", "\u{1F550} Время"),
                ("uk", "\u{1F550} Час"),
                ("de", "\u{1F550} Zeit"),
                ("es", "\u{1F550} Hora"),
            ],
        )
        .add(
            "time_now",
            &[
                ("en", "Now"),
                ("ru", "Сейчас"),
                ("uk", "Зараз"),
                ("de", "Jetzt"),
                ("es", "Ahora"),
            ],
        )
        .add(
            "time_tz",
            &[
                ("en", "Timezone"),
                ("ru", "Часовой пояс"),
                ("uk", "Часовий пояс"),
                ("de", "Zeitzone"),
                ("es", "Zona horaria"),
            ],
        )
        .add(
            "time_bad_offset",
            &[
                ("en", "couldn't parse the offset. examples: `/time +10`, `/time -5:30`, `/time 0`"),
                ("ru", "не понял сдвиг. примеры: `/time +10`, `/time -5:30`, `/time 0`"),
                ("uk", "не зрозумів зсув. приклади: `/time +10`, `/time -5:30`, `/time 0`"),
                ("de", "konnte den Versatz nicht lesen. Beispiele: `/time +10`, `/time -5:30`, `/time 0`"),
                ("es", "no entendí el desfase. ejemplos: `/time +10`, `/time -5:30`, `/time 0`"),
            ],
        )
        // -- /qr ----------------------------------------------------------------
        .add(
            "qr_title",
            &[
                ("en", "\u{1F4F1} QR Code"),
                ("ru", "\u{1F4F1} QR-код"),
                ("uk", "\u{1F4F1} QR-код"),
                ("de", "\u{1F4F1} QR-Code"),
                ("es", "\u{1F4F1} Código QR"),
            ],
        )
        .add(
            "qr_usage",
            &[
                ("en", "Give me something to encode: `/qr https://example.com`"),
                ("ru", "Дай что-нибудь закодировать: `/qr https://example.com`"),
                ("uk", "Дай щось закодувати: `/qr https://example.com`"),
                ("de", "Gib mir etwas zum Kodieren: `/qr https://example.com`"),
                ("es", "Dame algo que codificar: `/qr https://example.com`"),
            ],
        )
        .add(
            "qr_too_long",
            &[
                ("en", "Text is too long for a QR code (max 800 chars)."),
                ("ru", "Текст слишком длинный для QR-кода (максимум 800 символов)."),
                ("uk", "Текст задовгий для QR-коду (максимум 800 символів)."),
                ("de", "Der Text ist zu lang für einen QR-Code (max. 800 Zeichen)."),
                ("es", "El texto es demasiado largo para un código QR (máx. 800 caracteres)."),
            ],
        )
        .add(
            "qr_encoded",
            &[
                ("en", "Encoded: `{}`"),
                ("ru", "Закодировано: `{}`"),
                ("uk", "Закодовано: `{}`"),
                ("de", "Kodiert: `{}`"),
                ("es", "Codificado: `{}`"),
            ],
        )
        // -- /shorten -------------------------------------------------------------
        .add(
            "shorten_title",
            &[
                ("en", "\u{1F517} Shorten"),
                ("ru", "\u{1F517} Сократить"),
                ("uk", "\u{1F517} Скоротити"),
                ("de", "\u{1F517} Kürzen"),
                ("es", "\u{1F517} Acortar"),
            ],
        )
        .add(
            "shorten_usage",
            &[
                ("en", "Usage: `/shorten https://example.com/long/path`"),
                ("ru", "Формат: `/shorten https://example.com/long/path`"),
                ("uk", "Формат: `/shorten https://example.com/long/path`"),
                ("de", "Nutzung: `/shorten https://example.com/long/path`"),
                ("es", "Uso: `/shorten https://example.com/long/path`"),
            ],
        )
        .add(
            "shorten_bad_url",
            &[
                ("en", "URL must start with `http://` or `https://`."),
                ("ru", "URL должен начинаться с `http://` или `https://`."),
                ("uk", "URL має починатися з `http://` або `https://`."),
                ("de", "Die URL muss mit `http://` oder `https://` beginnen."),
                ("es", "La URL debe empezar por `http://` o `https://`."),
            ],
        )
        .add(
            "shorten_done_title",
            &[
                ("en", "\u{1F517} Shortened"),
                ("ru", "\u{1F517} Сокращено"),
                ("uk", "\u{1F517} Скорочено"),
                ("de", "\u{1F517} Gekürzt"),
                ("es", "\u{1F517} Acortado"),
            ],
        )
        .add(
            "shorten_original",
            &[
                ("en", "Original"),
                ("ru", "Оригинал"),
                ("uk", "Оригінал"),
                ("de", "Original"),
                ("es", "Original"),
            ],
        )
        .add(
            "shorten_short",
            &[
                ("en", "Short"),
                ("ru", "Короткая"),
                ("uk", "Коротке"),
                ("de", "Kurz"),
                ("es", "Corta"),
            ],
        )
        .add(
            "shorten_failed_title",
            &[
                ("en", "\u{274C} Shorten Failed"),
                ("ru", "\u{274C} Не сократилось"),
                ("uk", "\u{274C} Не скоротилося"),
                ("de", "\u{274C} Kürzen fehlgeschlagen"),
                ("es", "\u{274C} No se pudo acortar"),
            ],
        )
        .add(
            "shorten_failed_body",
            &[
                ("en", "No shortening provider is reachable right now. Try again later."),
                ("ru", "Ни один сервис сокращения сейчас не отвечает. Попробуй позже."),
                ("uk", "Жоден сервіс скорочення зараз не відповідає. Спробуй пізніше."),
                ("de", "Gerade ist kein Kürzungsdienst erreichbar. Versuch es später erneut."),
                ("es", "Ningún servicio de acortamiento responde ahora. Inténtalo más tarde."),
            ],
        )
        // -- /weather -------------------------------------------------------------
        .add(
            "weather_title",
            &[
                ("en", "\u{1F324}\u{FE0F} Weather"),
                ("ru", "\u{1F324}\u{FE0F} Погода"),
                ("uk", "\u{1F324}\u{FE0F} Погода"),
                ("de", "\u{1F324}\u{FE0F} Wetter"),
                ("es", "\u{1F324}\u{FE0F} Tiempo"),
            ],
        )
        .add(
            "weather_usage",
            &[
                ("en", "Give me a city name: `/weather Berlin`"),
                ("ru", "Напиши название города: `/weather Berlin`"),
                ("uk", "Напиши назву міста: `/weather Berlin`"),
                ("de", "Gib mir einen Stadtnamen: `/weather Berlin`"),
                ("es", "Dame el nombre de una ciudad: `/weather Berlin`"),
            ],
        )
        .add(
            "weather_failed_title",
            &[
                ("en", "\u{1F324}\u{FE0F} Weather Failed"),
                ("ru", "\u{1F324}\u{FE0F} Не получилось"),
                ("uk", "\u{1F324}\u{FE0F} Не вийшло"),
                ("de", "\u{1F324}\u{FE0F} Wetter fehlgeschlagen"),
                ("es", "\u{1F324}\u{FE0F} Fallo del tiempo"),
            ],
        )
        .add(
            "weather_failed_body",
            &[
                ("en", "Weather: {}"),
                ("ru", "Погода: {}"),
                ("uk", "Погода: {}"),
                ("de", "Wetter: {}"),
                ("es", "Tiempo: {}"),
            ],
        )
        .add(
            "weather_err_city",
            &[
                ("en", "couldn't find that city"),
                ("ru", "не нашёл такой город"),
                ("uk", "не знайшов такого міста"),
                ("de", "diese Stadt wurde nicht gefunden"),
                ("es", "no encontré esa ciudad"),
            ],
        )
        .add(
            "weather_err_http",
            &[
                ("en", "the weather provider answered HTTP {}"),
                ("ru", "погодный сервис ответил HTTP {}"),
                ("uk", "погодний сервіс відповів HTTP {}"),
                ("de", "der Wetterdienst antwortete mit HTTP {}"),
                ("es", "el servicio del tiempo respondió HTTP {}"),
            ],
        )
        .add(
            "weather_err_network",
            &[
                ("en", "the weather provider is unreachable"),
                ("ru", "погодный сервис недоступен"),
                ("uk", "погодний сервіс недоступний"),
                ("de", "der Wetterdienst ist nicht erreichbar"),
                ("es", "el servicio del tiempo no está disponible"),
            ],
        )
        .add(
            "weather_err_json",
            &[
                ("en", "the weather provider sent an odd response"),
                ("ru", "погодный сервис прислал странный ответ"),
                ("uk", "погодний сервіс надіслав дивну відповідь"),
                ("de", "der Wetterdienst schickte eine seltsame Antwort"),
                ("es", "el servicio del tiempo envió una respuesta rara"),
            ],
        )
        .add(
            "weather_label_temp",
            &[
                ("en", "Temperature"),
                ("ru", "Температура"),
                ("uk", "Температура"),
                ("de", "Temperatur"),
                ("es", "Temperatura"),
            ],
        )
        .add(
            "weather_label_feels",
            &[
                ("en", "Feels Like"),
                ("ru", "Ощущается"),
                ("uk", "Відчувається"),
                ("de", "Gefühlt"),
                ("es", "Sensación"),
            ],
        )
        .add(
            "weather_label_humidity",
            &[
                ("en", "Humidity"),
                ("ru", "Влажность"),
                ("uk", "Вологість"),
                ("de", "Luftfeuchtigkeit"),
                ("es", "Humedad"),
            ],
        )
        .add(
            "weather_label_wind",
            &[
                ("en", "Wind"),
                ("ru", "Ветер"),
                ("uk", "Вітер"),
                ("de", "Wind"),
                ("es", "Viento"),
            ],
        )
        .add(
            "weather_wind_unit",
            &[
                ("en", "m/s"),
                ("ru", "м/с"),
                ("uk", "м/с"),
                ("de", "m/s"),
                ("es", "m/s"),
            ],
        )
        .add(
            "weather_label_local_time",
            &[
                ("en", "Local Time"),
                ("ru", "Местное время"),
                ("uk", "Місцевий час"),
                ("de", "Ortszeit"),
                ("es", "Hora local"),
            ],
        )
        .add(
            "weather_cond_clear",
            &[
                ("en", "clear sky"),
                ("ru", "ясно"),
                ("uk", "ясно"),
                ("de", "klarer Himmel"),
                ("es", "cielo despejado"),
            ],
        )
        .add(
            "weather_cond_partly",
            &[
                ("en", "partly cloudy"),
                ("ru", "переменная облачность"),
                ("uk", "мінлива хмарність"),
                ("de", "teils bewölkt"),
                ("es", "parcialmente nublado"),
            ],
        )
        .add(
            "weather_cond_overcast",
            &[
                ("en", "overcast"),
                ("ru", "пасмурно"),
                ("uk", "похмуро"),
                ("de", "bedeckt"),
                ("es", "nublado"),
            ],
        )
        .add(
            "weather_cond_fog",
            &[
                ("en", "fog"),
                ("ru", "туман"),
                ("uk", "туман"),
                ("de", "Nebel"),
                ("es", "niebla"),
            ],
        )
        .add(
            "weather_cond_drizzle",
            &[
                ("en", "drizzle"),
                ("ru", "морось"),
                ("uk", "мряка"),
                ("de", "Nieselregen"),
                ("es", "llovizna"),
            ],
        )
        .add(
            "weather_cond_rain",
            &[
                ("en", "rain"),
                ("ru", "дождь"),
                ("uk", "дощ"),
                ("de", "Regen"),
                ("es", "lluvia"),
            ],
        )
        .add(
            "weather_cond_snow",
            &[
                ("en", "snow"),
                ("ru", "снег"),
                ("uk", "сніг"),
                ("de", "Schnee"),
                ("es", "nieve"),
            ],
        )
        .add(
            "weather_cond_thunder",
            &[
                ("en", "thunderstorm"),
                ("ru", "гроза"),
                ("uk", "гроза"),
                ("de", "Gewitter"),
                ("es", "tormenta"),
            ],
        )
        .add(
            "weather_cond_generic",
            &[
                ("en", "weather"),
                ("ru", "погода"),
                ("uk", "погода"),
                ("de", "Wetter"),
                ("es", "tiempo"),
            ],
        )
        // -- /calc ------------------------------------------------------------
        .add(
            "calc_title",
            &[
                ("en", "\u{1F9EE} Calc"),
                ("ru", "\u{1F9EE} Калькулятор"),
                ("uk", "\u{1F9EE} Калькулятор"),
                ("de", "\u{1F9EE} Rechner"),
                ("es", "\u{1F9EE} Calculadora"),
            ],
        )
        .add(
            "calc_usage",
            &[
                ("en", "Give me an expression: `/calc 2*(3+4)`"),
                ("ru", "Дай выражение: `/calc 2*(3+4)`"),
                ("uk", "Дай вираз: `/calc 2*(3+4)`"),
                ("de", "Gib mir einen Ausdruck: `/calc 2*(3+4)`"),
                ("es", "Dame una expresión: `/calc 2*(3+4)`"),
            ],
        )
        .add(
            "calc_expression",
            &[
                ("en", "Expression"),
                ("ru", "Выражение"),
                ("uk", "Вираз"),
                ("de", "Ausdruck"),
                ("es", "Expresión"),
            ],
        )
        .add(
            "calc_result",
            &[
                ("en", "Result"),
                ("ru", "Результат"),
                ("uk", "Результат"),
                ("de", "Ergebnis"),
                ("es", "Resultado"),
            ],
        )
        .add(
            "calc_bad_expr",
            &[
                ("en", "Couldn't parse that. Allowed: numbers, `+ - * / %` and parentheses."),
                ("ru", "Не смог разобрать. Разрешены: числа, `+ - * / %` и скобки."),
                ("uk", "Не зміг розібрати. Дозволено: числа, `+ - * / %` і дужки."),
                ("de", "Konnte das nicht lesen. Erlaubt: Zahlen, `+ - * / %` und Klammern."),
                ("es", "No pude interpretarlo. Permitido: números, `+ - * / %` y paréntesis."),
            ],
        )
        // -- /help, /ping, /info ------------------------------------------------
        .add(
            "help_title",
            &[
                ("en", "\u{1F44B} FoukoBot"),
                ("ru", "\u{1F44B} FoukoBot"),
                ("uk", "\u{1F44B} FoukoBot"),
                ("de", "\u{1F44B} FoukoBot"),
                ("es", "\u{1F44B} FoukoBot"),
            ],
        )
        .add(
            "help_intro",
            &[
                ("en", "One bot, every chat platform. Type `/help` to see every command."),
                ("ru", "Один бот - много платформ. Напиши `/help`, чтобы увидеть все команды."),
                ("uk", "Один бот - багато платформ. Напиши `/help`, щоб побачити всі команди."),
                ("de", "Ein Bot, jede Chat-Plattform. Tipp `/help` für alle Befehle."),
                ("es", "Un bot, todas las plataformas de chat. Escribe `/help` para ver todos los comandos."),
            ],
        )
        .add(
            "help_try",
            &[
                ("en", "Try"),
                ("ru", "Попробуй"),
                ("uk", "Спробуй"),
                ("de", "Probier mal"),
                ("es", "Prueba"),
            ],
        )
        .add(
            "ping_title",
            &[
                ("en", "\u{1F3D3} Pong"),
                ("ru", "\u{1F3D3} Понг"),
                ("uk", "\u{1F3D3} Понг"),
                ("de", "\u{1F3D3} Pong"),
                ("es", "\u{1F3D3} Pong"),
            ],
        )
        .add(
            "ping_body",
            &[
                ("en", "Bot is alive and well."),
                ("ru", "Бот жив и здоров."),
                ("uk", "Бот живий і здоровий."),
                ("de", "Der Bot ist quicklebendig."),
                ("es", "El bot está vivito y coleando."),
            ],
        )
        .add(
            "info_tagline",
            &[
                ("en", "One bot, Telegram and Discord at the same time."),
                ("ru", "Один бот - сразу в Telegram и Discord."),
                ("uk", "Один бот - одразу в Telegram і Discord."),
                ("de", "Ein Bot, gleichzeitig auf Telegram und Discord."),
                ("es", "Un bot, en Telegram y Discord a la vez."),
            ],
        )
        .add(
            "info_about_label",
            &[
                ("en", "About"),
                ("ru", "О боте"),
                ("uk", "Про бота"),
                ("de", "Über"),
                ("es", "Acerca de"),
            ],
        )
        .add(
            "info_about_text",
            &[
                ("en", "Open-source bot written in Rust. Levels, coins, cross-platform account linking, achievements and pretty embeds - all in one place."),
                ("ru", "Открытый проект на Rust. Уровни, монеты, связывание аккаунтов между платформами, достижения и красивые эмбеды - всё в одном боте."),
                ("uk", "Відкритий проєкт на Rust. Рівні, монети, зв'язування акаунтів між платформами, досягнення та гарні ембеди - все в одному боті."),
                ("de", "Open-Source-Bot in Rust. Level, Münzen, plattformübergreifende Kontoverknüpfung, Erfolge und hübsche Embeds - alles an einem Ort."),
                ("es", "Bot de código abierto escrito en Rust. Niveles, monedas, vinculación de cuentas entre plataformas, logros y embeds bonitos - todo en un solo sitio."),
            ],
        )
        .add(
            "info_stack_label",
            &[
                ("en", "Stack"),
                ("ru", "Стек"),
                ("uk", "Стек"),
                ("de", "Stack"),
                ("es", "Stack"),
            ],
        )
        .add(
            "info_stack_text",
            &[
                ("en", "`Rust` · `tokio` · `teloxide` · `serenity` · `SQLite`\nCore: [FoukoApi](https://api.fouko.xyz) - a tiny framework made for bots like this."),
                ("ru", "`Rust` · `tokio` · `teloxide` · `serenity` · `SQLite`\nЯдро: [FoukoApi](https://api.fouko.xyz) - мини-фреймворк для таких ботов."),
                ("uk", "`Rust` · `tokio` · `teloxide` · `serenity` · `SQLite`\nЯдро: [FoukoApi](https://api.fouko.xyz) - міні-фреймворк для таких ботів."),
                ("de", "`Rust` · `tokio` · `teloxide` · `serenity` · `SQLite`\nKern: [FoukoApi](https://api.fouko.xyz) - ein kleines Framework für solche Bots."),
                ("es", "`Rust` · `tokio` · `teloxide` · `serenity` · `SQLite`\nNúcleo: [FoukoApi](https://api.fouko.xyz) - un mini framework para bots como este."),
            ],
        )
        .add(
            "info_features_label",
            &[
                ("en", "Features"),
                ("ru", "Возможности"),
                ("uk", "Можливості"),
                ("de", "Funktionen"),
                ("es", "Funciones"),
            ],
        )
        .add(
            "info_features_text",
            &[
                ("en", "• Synced profile & XP between Telegram and Discord\n• Economy: /daily, shop, /gamble, /leaderboard\n• Code-based account linking (/link)\n• On-the-fly language switch (/lang)\n• Weather, time, dice, QR, polls and more"),
                ("ru", "• Синхронные профиль и XP между Telegram и Discord\n• Экономика: /daily, магазин, /gamble, /leaderboard\n• Связывание аккаунтов по коду (/link)\n• Переключение языка на лету (/lang)\n• Погода, время, кубики, QR, опросы и прочее"),
                ("uk", "• Синхронні профіль і XP між Telegram і Discord\n• Економіка: /daily, крамниця, /gamble, /leaderboard\n• Зв'язування акаунтів за кодом (/link)\n• Перемикання мови на льоту (/lang)\n• Погода, час, кубики, QR, опитування та інше"),
                ("de", "• Synchrones Profil & XP zwischen Telegram und Discord\n• Wirtschaft: /daily, Shop, /gamble, /leaderboard\n• Kontoverknüpfung per Code (/link)\n• Sprachwechsel im Handumdrehen (/lang)\n• Wetter, Zeit, Würfel, QR, Umfragen und mehr"),
                ("es", "• Perfil y XP sincronizados entre Telegram y Discord\n• Economía: /daily, tienda, /gamble, /leaderboard\n• Vinculación de cuentas por código (/link)\n• Cambio de idioma al vuelo (/lang)\n• Tiempo, hora, dados, QR, encuestas y más"),
            ],
        )
        .add(
            "info_links_label",
            &[
                ("en", "Links"),
                ("ru", "Ссылки"),
                ("uk", "Посилання"),
                ("de", "Links"),
                ("es", "Enlaces"),
            ],
        )
        .add(
            "info_links_text",
            &[
                ("en", "[Website](https://bot.fouko.xyz) · [API](https://api.fouko.xyz) · [GitHub](https://github.com/FoukoDev)"),
                ("ru", "[Сайт](https://bot.fouko.xyz) · [API](https://api.fouko.xyz) · [GitHub](https://github.com/FoukoDev)"),
                ("uk", "[Сайт](https://bot.fouko.xyz) · [API](https://api.fouko.xyz) · [GitHub](https://github.com/FoukoDev)"),
                ("de", "[Website](https://bot.fouko.xyz) · [API](https://api.fouko.xyz) · [GitHub](https://github.com/FoukoDev)"),
                ("es", "[Sitio web](https://bot.fouko.xyz) · [API](https://api.fouko.xyz) · [GitHub](https://github.com/FoukoDev)"),
            ],
        )
        .add(
            "info_footer",
            &[
                ("en", "/help - full command list"),
                ("ru", "/help - полный список команд"),
                ("uk", "/help - повний список команд"),
                ("de", "/help - vollständige Befehlsliste"),
                ("es", "/help - lista completa de comandos"),
            ],
        )
        .add(
            "info_version",
            &[
                ("en", "Version"),
                ("ru", "Версия"),
                ("uk", "Версія"),
                ("de", "Version"),
                ("es", "Versión"),
            ],
        )
        .add(
            "info_uptime",
            &[
                ("en", "Uptime"),
                ("ru", "Аптайм"),
                ("uk", "Аптайм"),
                ("de", "Laufzeit"),
                ("es", "Tiempo activo"),
            ],
        )
        .add(
            "info_players",
            &[
                ("en", "Players"),
                ("ru", "Игроков"),
                ("uk", "Гравців"),
                ("de", "Spieler"),
                ("es", "Jugadores"),
            ],
        )
        // -- /server, /avatar (labels) ------------------------------------------
        .add(
            "srv_title",
            &[
                ("en", "\u{1F3E0} Server"),
                ("ru", "\u{1F3E0} Сервер"),
                ("uk", "\u{1F3E0} Сервер"),
                ("de", "\u{1F3E0} Server"),
                ("es", "\u{1F3E0} Servidor"),
            ],
        )
        .add(
            "srv_title_discord",
            &[
                ("en", "\u{1F3F0} Server Info"),
                ("ru", "\u{1F3F0} О сервере"),
                ("uk", "\u{1F3F0} Про сервер"),
                ("de", "\u{1F3F0} Server-Info"),
                ("es", "\u{1F3F0} Info del servidor"),
            ],
        )
        .add(
            "srv_title_chat",
            &[
                ("en", "\u{1F465} Chat Info"),
                ("ru", "\u{1F465} О чате"),
                ("uk", "\u{1F465} Про чат"),
                ("de", "\u{1F465} Chat-Info"),
                ("es", "\u{1F465} Info del chat"),
            ],
        )
        .add(
            "srv_name",
            &[
                ("en", "Name"),
                ("ru", "Название"),
                ("uk", "Назва"),
                ("de", "Name"),
                ("es", "Nombre"),
            ],
        )
        .add(
            "srv_members",
            &[
                ("en", "Members"),
                ("ru", "Участников"),
                ("uk", "Учасників"),
                ("de", "Mitglieder"),
                ("es", "Miembros"),
            ],
        )
        .add(
            "srv_about",
            &[
                ("en", "About"),
                ("ru", "Описание"),
                ("uk", "Опис"),
                ("de", "Beschreibung"),
                ("es", "Descripción"),
            ],
        )
        .add(
            "avatar_title",
            &[
                ("en", "\u{1F5BC}\u{FE0F} Avatar"),
                ("ru", "\u{1F5BC}\u{FE0F} Аватар"),
                ("uk", "\u{1F5BC}\u{FE0F} Аватар"),
                ("de", "\u{1F5BC}\u{FE0F} Avatar"),
                ("es", "\u{1F5BC}\u{FE0F} Avatar"),
            ],
        )
        .add(
            "avatar_banner",
            &[
                ("en", "Banner"),
                ("ru", "Баннер"),
                ("uk", "Банер"),
                ("de", "Banner"),
                ("es", "Banner"),
            ],
        )
        // -- /settings ------------------------------------------------------------
        .add(
            "settings_title",
            &[
                ("en", "\u{2699}\u{FE0F} Settings"),
                ("ru", "\u{2699}\u{FE0F} Настройки"),
                ("uk", "\u{2699}\u{FE0F} Налаштування"),
                ("de", "\u{2699}\u{FE0F} Einstellungen"),
                ("es", "\u{2699}\u{FE0F} Ajustes"),
            ],
        )
        .add(
            "settings_lang",
            &[
                ("en", "Language"),
                ("ru", "Язык"),
                ("uk", "Мова"),
                ("de", "Sprache"),
                ("es", "Idioma"),
            ],
        )
        .add(
            "settings_platform",
            &[
                ("en", "Primary Platform"),
                ("ru", "Основная платформа"),
                ("uk", "Основна платформа"),
                ("de", "Hauptplattform"),
                ("es", "Plataforma principal"),
            ],
        )
        .add(
            "settings_linked",
            &[
                ("en", "Linked Accounts"),
                ("ru", "Связанные аккаунты"),
                ("uk", "Зв'язані акаунти"),
                ("de", "Verknüpfte Konten"),
                ("es", "Cuentas vinculadas"),
            ],
        )
        .add(
            "settings_linked_none",
            &[
                ("en", "none"),
                ("ru", "нет"),
                ("uk", "немає"),
                ("de", "keine"),
                ("es", "ninguna"),
            ],
        )
        // -- /ai ------------------------------------------------------------
        .add("ai_disabled", &[
            ("en", "The AI feature isn't enabled on this bot (no encryption key set)."),
            ("ru", "Функция ИИ не включена на этом боте (не задан ключ шифрования)."),
            ("uk", "Функція ШІ не увімкнена на цьому боті (не задано ключ шифрування)."),
            ("de", "Die KI-Funktion ist auf diesem Bot nicht aktiviert (kein Schlüssel gesetzt)."),
            ("es", "La función de IA no está activada en este bot (sin clave de cifrado)."),
        ])
        .add("ai_dm_only", &[
            ("en", "Manage your AI only in a private chat with the bot."),
            ("ru", "Управляй ИИ только в личке с ботом."),
            ("uk", "Керуй ШІ лише в приваті з ботом."),
            ("de", "Verwalte deine KI nur im privaten Chat mit dem Bot."),
            ("es", "Gestiona tu IA solo en un chat privado con el bot."),
        ])
        .add("ai_dm_only_title", &[
            ("en", "\u{1F512} DM only"),
            ("ru", "\u{1F512} Только в ЛС"),
            ("uk", "\u{1F512} Лише в ЛС"),
            ("de", "\u{1F512} Nur im DM"),
            ("es", "\u{1F512} Solo en MD"),
        ])
        .add("ai_host_usage", &[
            ("en", "Usage: `/ai host add <name> <url> [key]` or `/ai host del <name>`."),
            ("ru", "Формат: `/ai host add <имя> <url> [ключ]` или `/ai host del <имя>`."),
            ("uk", "Формат: `/ai host add <ім'я> <url> [ключ]` або `/ai host del <ім'я>`."),
            ("de", "Nutzung: `/ai host add <Name> <URL> [Key]` oder `/ai host del <Name>`."),
            ("es", "Uso: `/ai host add <nombre> <url> [clave]` o `/ai host del <nombre>`."),
        ])
        .add("ai_host_add_usage", &[
            ("en", "Usage: `/ai host add <name> <url> [key]`."),
            ("ru", "Формат: `/ai host add <имя> <url> [ключ]`."),
            ("uk", "Формат: `/ai host add <ім'я> <url> [ключ]`."),
            ("de", "Nutzung: `/ai host add <Name> <URL> [Key]`."),
            ("es", "Uso: `/ai host add <nombre> <url> [clave]`."),
        ])
        .add("ai_host_del_usage", &[
            ("en", "Usage: `/ai host del <name>`."),
            ("ru", "Формат: `/ai host del <имя>`."),
            ("uk", "Формат: `/ai host del <ім'я>`."),
            ("de", "Nutzung: `/ai host del <Name>`."),
            ("es", "Uso: `/ai host del <nombre>`."),
        ])
        .add("ai_host_added", &[
            ("en", "Host added."), ("ru", "Хост добавлен."), ("uk", "Хост додано."),
            ("de", "Host hinzugefügt."), ("es", "Host añadido."),
        ])
        .add("ai_host_added_models", &[
            ("en", "Host added - found {} models on it. Create a chat: tap + Chat in /ai."),
            ("ru", "Хост добавлен - нашёл на нём {} моделей. Создай чат: кнопка + Чат в /ai."),
            ("uk", "Хост додано - знайшов на ньому {} моделей. Створи чат: кнопка + Чат в /ai."),
            ("de", "Host hinzugefügt - {} Modelle darauf gefunden. Erstelle einen Chat: + Chat in /ai."),
            ("es", "Host añadido - encontré {} modelos en él. Crea un chat: + Chat en /ai."),
        ])
        .add("ai_refresh_none", &[
            ("en", "The host didn't return a model list - add models by hand: `/ai model add <host> <model>`."),
            ("ru", "Хост не вернул список моделей - добавь вручную: `/ai model add <хост> <модель>`."),
            ("uk", "Хост не повернув список моделей - додай вручну: `/ai model add <хост> <модель>`."),
            ("de", "Der Host lieferte keine Modellliste - füge sie manuell hinzu: `/ai model add <Host> <Modell>`."),
            ("es", "El host no devolvió una lista de modelos - añádelos a mano: `/ai model add <host> <modelo>`."),
        ])
        .add("ai_host_added_key", &[
            ("en", "Host added - your key is stored encrypted. Delete the message you just sent so the key doesn't linger in your chat history."),
            ("ru", "Хост добавлен - ключ хранится в зашифрованном виде. Удали своё сообщение, чтобы ключ не оставался в истории чата."),
            ("uk", "Хост додано - ключ зберігається зашифрованим. Видали своє повідомлення, щоб ключ не лишався в історії чату."),
            ("de", "Host hinzugefügt - dein Schlüssel wird verschlüsselt gespeichert. Lösche deine Nachricht, damit der Schlüssel nicht im Verlauf bleibt."),
            ("es", "Host añadido - tu clave se guarda cifrada. Borra el mensaje que enviaste para que la clave no quede en el historial."),
        ])
        .add("ai_too_fast", &[
            ("en", "Slow down a moment - one AI request every few seconds."),
            ("ru", "Помедленнее - один запрос к ИИ раз в несколько секунд."),
            ("uk", "Повільніше - один запит до ШІ раз на кілька секунд."),
            ("de", "Etwas langsamer - eine KI-Anfrage alle paar Sekunden."),
            ("es", "Más despacio - una petición de IA cada pocos segundos."),
        ])
        .add("ai_too_long", &[
            ("en", "That message is too long for the AI - trim it down a bit."),
            ("ru", "Сообщение слишком длинное для ИИ - сократи немного."),
            ("uk", "Повідомлення надто довге для ШІ - скороти трохи."),
            ("de", "Die Nachricht ist zu lang für die KI - kürze sie etwas."),
            ("es", "Ese mensaje es demasiado largo para la IA - recórtalo un poco."),
        ])
        .add("ai_open_in_dm", &[
            ("en", "Open a DM with me to set up and use your AI."),
            ("ru", "Открой личку со мной, чтобы настроить и использовать ИИ."),
            ("uk", "Відкрий приват зі мною, щоб налаштувати та використовувати ШІ."),
            ("de", "Öffne einen DM mit mir, um deine KI einzurichten und zu nutzen."),
            ("es", "Abre un chat privado conmigo para configurar y usar tu IA."),
        ])
        // -- /ai setup wizard -------------------------------------------------
        .add("ai_wiz_host_name", &[
            ("en", "Name the new host (one word, e.g. `local`):"),
            ("ru", "Название нового хоста (одно слово, например `local`):"),
            ("uk", "Назва нового хоста (одне слово, наприклад `local`):"),
            ("de", "Name des neuen Hosts (ein Wort, z. B. `local`):"),
            ("es", "Nombre del nuevo host (una palabra, p. ej. `local`):"),
        ])
        .add("ai_wiz_host_url", &[
            ("en", "Now its URL (e.g. `http://127.0.0.1:11434`):"),
            ("ru", "Теперь его URL (например `http://127.0.0.1:11434`):"),
            ("uk", "Тепер його URL (наприклад `http://127.0.0.1:11434`):"),
            ("de", "Jetzt die URL (z. B. `http://127.0.0.1:11434`):"),
            ("es", "Ahora su URL (p. ej. `http://127.0.0.1:11434`):"),
        ])
        .add("ai_wiz_host_key", &[
            ("en", "API key, or `-` if the host doesn't need one:"),
            ("ru", "API-ключ, или `-` если хосту он не нужен:"),
            ("uk", "API-ключ, або `-` якщо хосту він не потрібен:"),
            ("de", "API-Schlüssel, oder `-` falls keiner nötig ist:"),
            ("es", "Clave API, o `-` si el host no la necesita:"),
        ])
        .add("ai_wiz_no_spaces", &[
            ("en", "No spaces, please - try a single word."),
            ("ru", "Без пробелов - попробуй одним словом."),
            ("uk", "Без пробілів - спробуй одним словом."),
            ("de", "Bitte ohne Leerzeichen - ein einzelnes Wort."),
            ("es", "Sin espacios - prueba con una sola palabra."),
        ])
        .add("ai_wiz_bad_url", &[
            ("en", "That doesn't look like a URL - it should start with `http://` or `https://`."),
            ("ru", "Не похоже на URL - он должен начинаться с `http://` или `https://`."),
            ("uk", "Не схоже на URL - він має починатися з `http://` або `https://`."),
            ("de", "Das sieht nicht nach einer URL aus - sie muss mit `http://` oder `https://` beginnen."),
            ("es", "Eso no parece una URL - debe empezar por `http://` o `https://`."),
        ])
        .add("ai_wiz_pick_host", &[
            ("en", "Which host should the new chat use?"),
            ("ru", "Какой хост использовать для нового чата?"),
            ("uk", "Який хост використовувати для нового чату?"),
            ("de", "Welchen Host soll der neue Chat verwenden?"),
            ("es", "¿Qué host debe usar el nuevo chat?"),
        ])
        .add("ai_wiz_pick_model", &[
            ("en", "And which model?"),
            ("ru", "А какую модель?"),
            ("uk", "А яку модель?"),
            ("de", "Und welches Modell?"),
            ("es", "¿Y qué modelo?"),
        ])
        .add("ai_wiz_chat_name", &[
            ("en", "Almost done - name this chat:"),
            ("ru", "Почти готово - назови этот чат:"),
            ("uk", "Майже готово - назви цей чат:"),
            ("de", "Fast fertig - gib dem Chat einen Namen:"),
            ("es", "Casi listo - ponle nombre a este chat:"),
        ])
        .add("ai_wiz_no_hosts", &[
            ("en", "No hosts yet - add one first."),
            ("ru", "Хостов ещё нет - сначала добавь один."),
            ("uk", "Хостів ще немає - спершу додай один."),
            ("de", "Noch keine Hosts - füge zuerst einen hinzu."),
            ("es", "Aún no hay hosts - añade uno primero."),
        ])
        .add("ai_wiz_no_models", &[
            ("en", "That host has no models - add one with `/ai model add <host> <model>`."),
            ("ru", "У этого хоста нет моделей - добавь: `/ai model add <хост> <модель>`."),
            ("uk", "У цього хоста немає моделей - додай: `/ai model add <хост> <модель>`."),
            ("de", "Dieser Host hat keine Modelle - füge eines mit `/ai model add <Host> <Modell>` hinzu."),
            ("es", "Ese host no tiene modelos - añade uno con `/ai model add <host> <modelo>`."),
        ])
        .add("ai_host_removed", &[
            ("en", "Host removed."), ("ru", "Хост удалён."), ("uk", "Хост видалено."),
            ("de", "Host entfernt."), ("es", "Host eliminado."),
        ])
        .add("ai_host_exists", &[
            ("en", "A host with that name already exists."),
            ("ru", "Хост с таким именем уже есть."),
            ("uk", "Хост з таким ім'ям вже існує."),
            ("de", "Ein Host mit diesem Namen existiert bereits."),
            ("es", "Ya existe un host con ese nombre."),
        ])
        .add("ai_host_missing", &[
            ("en", "No such host."), ("ru", "Нет такого хоста."), ("uk", "Немає такого хоста."),
            ("de", "Kein solcher Host."), ("es", "No existe ese host."),
        ])
        .add("ai_model_usage", &[
            ("en", "Usage: `/ai model add <host> <model>` or `/ai model del <host> <model>`."),
            ("ru", "Формат: `/ai model add <хост> <модель>` или `/ai model del <хост> <модель>`."),
            ("uk", "Формат: `/ai model add <хост> <модель>` або `/ai model del <хост> <модель>`."),
            ("de", "Nutzung: `/ai model add <Host> <Modell>` oder `/ai model del <Host> <Modell>`."),
            ("es", "Uso: `/ai model add <host> <modelo>` o `/ai model del <host> <modelo>`."),
        ])
        .add("ai_model_added", &[
            ("en", "Model added."), ("ru", "Модель добавлена."), ("uk", "Модель додано."),
            ("de", "Modell hinzugefügt."), ("es", "Modelo añadido."),
        ])
        .add("ai_model_removed", &[
            ("en", "Model removed."), ("ru", "Модель удалена."), ("uk", "Модель видалено."),
            ("de", "Modell entfernt."), ("es", "Modelo eliminado."),
        ])
        .add("ai_model_missing", &[
            ("en", "That model isn't registered on the host. Add it with `/ai model add`."),
            ("ru", "Эта модель не добавлена на хост. Добавь: `/ai model add`."),
            ("uk", "Ця модель не додана на хост. Додай: `/ai model add`."),
            ("de", "Dieses Modell ist nicht am Host registriert. Füge es mit `/ai model add` hinzu."),
            ("es", "Ese modelo no está registrado en el host. Añádelo con `/ai model add`."),
        ])
        .add("ai_chat_usage", &[
            ("en", "Usage: `/ai chat new <name> <host> <model>` or `/ai chat del <name>`."),
            ("ru", "Формат: `/ai chat new <имя> <хост> <модель>` или `/ai chat del <имя>`."),
            ("uk", "Формат: `/ai chat new <ім'я> <хост> <модель>` або `/ai chat del <ім'я>`."),
            ("de", "Nutzung: `/ai chat new <Name> <Host> <Modell>` oder `/ai chat del <Name>`."),
            ("es", "Uso: `/ai chat new <nombre> <host> <modelo>` o `/ai chat del <nombre>`."),
        ])
        .add("ai_chat_new_usage", &[
            ("en", "Usage: `/ai chat new <name> <host> <model>`."),
            ("ru", "Формат: `/ai chat new <имя> <хост> <модель>`."),
            ("uk", "Формат: `/ai chat new <ім'я> <хост> <модель>`."),
            ("de", "Nutzung: `/ai chat new <Name> <Host> <Modell>`."),
            ("es", "Uso: `/ai chat new <nombre> <host> <modelo>`."),
        ])
        .add("ai_chat_del_usage", &[
            ("en", "Usage: `/ai chat del <name>`."),
            ("ru", "Формат: `/ai chat del <имя>`."),
            ("uk", "Формат: `/ai chat del <ім'я>`."),
            ("de", "Nutzung: `/ai chat del <Name>`."),
            ("es", "Uso: `/ai chat del <nombre>`."),
        ])
        .add("ai_chat_created", &[
            ("en", "Chat created and set active."),
            ("ru", "Чат создан и выбран активным."),
            ("uk", "Чат створено та зроблено активним."),
            ("de", "Chat erstellt und aktiviert."),
            ("es", "Chat creado y activado."),
        ])
        .add("ai_chat_removed", &[
            ("en", "Chat removed."), ("ru", "Чат удалён."), ("uk", "Чат видалено."),
            ("de", "Chat entfernt."), ("es", "Chat eliminado."),
        ])
        .add("ai_chat_missing", &[
            ("en", "No such chat."), ("ru", "Нет такого чата."), ("uk", "Немає такого чату."),
            ("de", "Kein solcher Chat."), ("es", "No existe ese chat."),
        ])
        .add("ai_chat_active", &[
            ("en", "Active chat switched."),
            ("ru", "Активный чат переключён."),
            ("uk", "Активний чат перемкнено."),
            ("de", "Aktiver Chat gewechselt."),
            ("es", "Chat activo cambiado."),
        ])
        .add("ai_use_usage", &[
            ("en", "Usage: `/ai use <chat name>`."),
            ("ru", "Формат: `/ai use <имя чата>`."),
            ("uk", "Формат: `/ai use <ім'я чату>`."),
            ("de", "Nutzung: `/ai use <Chat-Name>`."),
            ("es", "Uso: `/ai use <nombre del chat>`."),
        ])
        .add("ai_no_active", &[
            ("en", "No active chat. Create one with `/ai chat new` or pick with `/ai use`."),
            ("ru", "Нет активного чата. Создай `/ai chat new` или выбери `/ai use`."),
            ("uk", "Немає активного чату. Створи `/ai chat new` або обери `/ai use`."),
            ("de", "Kein aktiver Chat. Erstelle einen mit `/ai chat new` oder wähle mit `/ai use`."),
            ("es", "No hay chat activo. Crea uno con `/ai chat new` o elige con `/ai use`."),
        ])
        .add("ai_prompt_set", &[
            ("en", "System prompt updated."),
            ("ru", "Системный промпт обновлён."),
            ("uk", "Системний промпт оновлено."),
            ("de", "System-Prompt aktualisiert."),
            ("es", "Prompt del sistema actualizado."),
        ])
        .add("ai_history_cleared", &[
            ("en", "History cleared."), ("ru", "История очищена."), ("uk", "Історію очищено."),
            ("de", "Verlauf gelöscht."), ("es", "Historial borrado."),
        ])
        .add("ai_say_usage", &[
            ("en", "Type a message after `/ai say`, or just write in this DM."),
            ("ru", "Напиши сообщение после `/ai say` или просто пиши в этом ЛС."),
            ("uk", "Напиши повідомлення після `/ai say` або просто пиши в цьому ЛС."),
            ("de", "Schreib eine Nachricht nach `/ai say` oder einfach hier im DM."),
            ("es", "Escribe un mensaje tras `/ai say`, o simplemente escribe en este chat."),
        ])
        .add("ai_error", &[
            ("en", "AI request failed: {}"),
            ("ru", "Запрос к ИИ не удался: {}"),
            ("uk", "Запит до ШІ не вдався: {}"),
            ("de", "KI-Anfrage fehlgeschlagen: {}"),
            ("es", "La solicitud a la IA falló: {}"),
        ])
        .add("ai_store_error", &[
            ("en", "Couldn't read your AI settings right now - nothing was changed. Try again later."),
            ("ru", "Не удалось прочитать настройки ИИ - ничего не изменено. Попробуй позже."),
            ("uk", "Не вдалося прочитати налаштування ШІ - нічого не змінено. Спробуй пізніше."),
            ("de", "Deine KI-Einstellungen konnten gerade nicht gelesen werden - nichts wurde geändert. Versuch es später erneut."),
            ("es", "No se pudieron leer tus ajustes de IA - no se cambió nada. Inténtalo más tarde."),
        ])
        .add("ai_host_revoked", &[
            ("en", "That host is no longer available to you (access changed or removed)."),
            ("ru", "Этот хост тебе больше недоступен (доступ изменён или отозван)."),
            ("uk", "Цей хост тобі більше недоступний (доступ змінено або відкликано)."),
            ("de", "Dieser Host ist für dich nicht mehr verfügbar (Zugriff geändert/entzogen)."),
            ("es", "Ese host ya no está disponible para ti (acceso cambiado o revocado)."),
        ])
        .add("ai_share_usage", &[
            ("en", "Usage: `/ai share <user> <host> [model1,model2]`. User is `platform:id`."),
            ("ru", "Формат: `/ai share <юзер> <хост> [модель1,модель2]`. Юзер: `platform:id`."),
            ("uk", "Формат: `/ai share <юзер> <хост> [модель1,модель2]`. Юзер: `platform:id`."),
            ("de", "Nutzung: `/ai share <User> <Host> [Modell1,Modell2]`. User ist `platform:id`."),
            ("es", "Uso: `/ai share <usuario> <host> [modelo1,modelo2]`. Usuario: `platform:id`."),
        ])
        .add("ai_share_self", &[
            ("en", "You can't share with yourself."),
            ("ru", "Нельзя поделиться с самим собой."),
            ("uk", "Не можна поділитися із собою."),
            ("de", "Du kannst nicht mit dir selbst teilen."),
            ("es", "No puedes compartir contigo mismo."),
        ])
        .add("ai_share_no_models", &[
            ("en", "None of those models exist on that host."),
            ("ru", "Ни одной из этих моделей нет на этом хосте."),
            ("uk", "Жодної з цих моделей немає на цьому хості."),
            ("de", "Keines dieser Modelle existiert auf dem Host."),
            ("es", "Ninguno de esos modelos existe en ese host."),
        ])
        .add("ai_share_sent", &[
            ("en", "Invite sent. It expires in 24h if unanswered."),
            ("ru", "Приглашение отправлено. Истечёт через 24ч без ответа."),
            ("uk", "Запрошення надіслано. Спливе за 24 год без відповіді."),
            ("de", "Einladung gesendet. Läuft nach 24h ohne Antwort ab."),
            ("es", "Invitación enviada. Caduca en 24h si no responden."),
        ])
        .add("ai_unshare_usage", &[
            ("en", "Usage: `/ai unshare <user>`."),
            ("ru", "Формат: `/ai unshare <юзер>`."),
            ("uk", "Формат: `/ai unshare <юзер>`."),
            ("de", "Nutzung: `/ai unshare <User>`."),
            ("es", "Uso: `/ai unshare <usuario>`."),
        ])
        .add("ai_unshare_done", &[
            ("en", "Access revoked."), ("ru", "Доступ отозван."), ("uk", "Доступ відкликано."),
            ("de", "Zugriff entzogen."), ("es", "Acceso revocado."),
        ])
        .add("ai_shared_none", &[
            ("en", "You haven't shared any hosts yet."),
            ("ru", "Ты ещё ни с кем не делился хостами."),
            ("uk", "Ти ще ні з ким не ділився хостами."),
            ("de", "Du hast noch keine Hosts geteilt."),
            ("es", "Aún no has compartido ningún host."),
        ])
        .add("ai_invite_gone", &[
            ("en", "That invite is no longer available."),
            ("ru", "Это приглашение больше недоступно."),
            ("uk", "Це запрошення більше недоступне."),
            ("de", "Diese Einladung ist nicht mehr verfügbar."),
            ("es", "Esa invitación ya no está disponible."),
        ])
        .add("ai_invite_expired", &[
            ("en", "That invite has expired."),
            ("ru", "Приглашение истекло."),
            ("uk", "Запрошення спливло."),
            ("de", "Diese Einladung ist abgelaufen."),
            ("es", "Esa invitación ha caducado."),
        ])
        .add("ai_invite_accepted", &[
            ("en", "Access accepted. Create a chat on the shared host with `/ai chat new`."),
            ("ru", "Доступ принят. Создай чат на общем хосте: `/ai chat new`."),
            ("uk", "Доступ прийнято. Створи чат на спільному хості: `/ai chat new`."),
            ("de", "Zugriff akzeptiert. Erstelle einen Chat auf dem geteilten Host mit `/ai chat new`."),
            ("es", "Acceso aceptado. Crea un chat en el host compartido con `/ai chat new`."),
        ])
        .add("ai_invite_declined", &[
            ("en", "Invite declined."), ("ru", "Приглашение отклонено."),
            ("uk", "Запрошення відхилено."), ("de", "Einladung abgelehnt."),
            ("es", "Invitación rechazada."),
        ])
        .add("ai_models_count", &[
            ("en", "({} models)"),
            ("ru", "({} моделей)"),
            ("uk", "({} моделей)"),
            ("de", "({} Modelle)"),
            ("es", "({} modelos)"),
        ])
        .add("ai_family_title", &[
            ("en", "\u{1F46A} Family Access"),
            ("ru", "\u{1F46A} Семейный доступ"),
            ("uk", "\u{1F46A} Сімейний доступ"),
            ("de", "\u{1F46A} Familienzugang"),
            ("es", "\u{1F46A} Acceso familiar"),
        ])
        .add("ai_family_invite_title", &[
            ("en", "\u{1F46A} AI family access"),
            ("ru", "\u{1F46A} Семейный доступ к ИИ"),
            ("uk", "\u{1F46A} Сімейний доступ до ШІ"),
            ("de", "\u{1F46A} KI-Familienzugang"),
            ("es", "\u{1F46A} Acceso familiar a la IA"),
        ])
        .add("ai_family_invite_dm", &[
            ("en", "**{}** wants to share an AI host with you. Accept to use it."),
            ("ru", "**{}** хочет поделиться с тобой хостом ИИ. Прими, чтобы пользоваться."),
            ("uk", "**{}** хоче поділитися з тобою хостом ШІ. Прийми, щоб користуватися."),
            ("de", "**{}** möchte einen KI-Host mit dir teilen. Nimm an, um ihn zu nutzen."),
            ("es", "**{}** quiere compartir un host de IA contigo. Acepta para usarlo."),
        ])
        .add("ai_invite_accept_btn", &[
            ("en", "\u{2705} Accept"),
            ("ru", "\u{2705} Принять"),
            ("uk", "\u{2705} Прийняти"),
            ("de", "\u{2705} Annehmen"),
            ("es", "\u{2705} Aceptar"),
        ])
        .add("ai_invite_decline_btn", &[
            ("en", "\u{274C} Decline"),
            ("ru", "\u{274C} Отклонить"),
            ("uk", "\u{274C} Відхилити"),
            ("de", "\u{274C} Ablehnen"),
            ("es", "\u{274C} Rechazar"),
        ])
        .add("ai_family_timeout_title", &[
            ("en", "\u{23F3} AI family access"),
            ("ru", "\u{23F3} Семейный доступ к ИИ"),
            ("uk", "\u{23F3} Сімейний доступ до ШІ"),
            ("de", "\u{23F3} KI-Familienzugang"),
            ("es", "\u{23F3} Acceso familiar a la IA"),
        ])
        .add("ai_family_timeout_body", &[
            ("en", "Your invite to **{}** went unanswered for 24h and was cancelled."),
            ("ru", "Твоё приглашение для **{}** осталось без ответа 24ч и было отменено."),
            ("uk", "Твоє запрошення для **{}** лишилося без відповіді 24 год і було скасовано."),
            ("de", "Deine Einladung an **{}** blieb 24h unbeantwortet und wurde storniert."),
            ("es", "Tu invitación a **{}** quedó sin respuesta durante 24h y fue cancelada."),
        ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_string_covers_every_language() {
        let gaps = catalogue().missing(SUPPORTED);
        assert!(
            gaps.is_empty(),
            "these strings are missing translations: {gaps:?}"
        );
    }
}
