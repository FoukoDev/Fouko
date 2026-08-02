//! Account commands: `/settings` - a small settings hub with buttons.

use super::economy::primary_id;
use super::Services;
use super::{COLOR_ACCENT, COLOR_OK, COLOR_WARN};
use foukoapi::{util::capitalize, Button, Ctx, Embed, Keyboard, Reply, Result};

/// `/settings` - language, linking and AI, all on buttons. Language
/// buttons switch in place; the rest point at the matching commands.
pub(crate) async fn settings_cmd(ctx: Ctx, svc: Services) -> Result<()> {
    // Button presses: `settings:lang:<code>` switches the language right
    // here and re-renders the card. Same DM-only rule as the command - the
    // card should never be drawn into a group chat.
    if let Some(data) = ctx.callback_data() {
        if !ctx.is_dm() {
            let em = Embed::new()
                .description(svc.tr(&ctx, "settings_dm_only").await)
                .color(COLOR_WARN);
            return ctx.reply_with(Reply::embed(em)).await;
        }
        if let Some(code) = data.strip_prefix("settings:lang:") {
            let supported = crate::strings::SUPPORTED;
            if supported.contains(&code) {
                svc.accounts
                    .set_lang(ctx.platform(), ctx.user_id(), code)
                    .await?;
            }
            return render(&ctx, &svc, true).await;
        }
        return Ok(());
    }

    if !ctx.is_dm() {
        let em = Embed::new()
            .description(svc.tr(&ctx, "settings_dm_only").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }

    render(&ctx, &svc, false).await
}

/// Draw (or redraw, after a button press) the settings card.
async fn render(ctx: &Ctx, svc: &Services, edit: bool) -> Result<()> {
    let lang = svc.lang(ctx).await;

    let partner = svc
        .accounts
        .partner_for(ctx.platform(), ctx.user_id())
        .await?;
    let primary = primary_id(ctx, svc).await;
    let primary_platform = primary.split(':').next().unwrap_or(&primary).to_owned();

    let linked = match &partner {
        Some(p) => {
            let me = ctx.platform().to_string();
            let other = p.split(':').next().unwrap_or(p);
            format!("{} \u{2194} {}", capitalize(&me), capitalize(other))
        }
        None => svc.tr(ctx, "settings_linked_none").await,
    };

    let em = Embed::new()
        .title(svc.tr(ctx, "settings_title").await)
        .field_inline(svc.tr(ctx, "settings_lang").await, lang.to_uppercase())
        .field_inline(
            svc.tr(ctx, "settings_platform").await,
            capitalize(&primary_platform),
        )
        .field(svc.tr(ctx, "settings_linked").await, linked)
        .color(if edit { COLOR_OK } else { COLOR_ACCENT });

    // Language buttons: one per supported language, the current one marked.
    let mut kb = Keyboard::new();
    for chunk in crate::strings::SUPPORTED.chunks(3) {
        let row: Vec<Button> = chunk
            .iter()
            .map(|code| {
                let mark = if *code == lang { "\u{2B50} " } else { "" };
                Button::callback(
                    format!("{mark}{}", code.to_uppercase()),
                    format!("settings:lang:{code}"),
                )
            })
            .collect();
        kb = kb.row(row);
    }

    if edit {
        ctx.edit_reply(Reply::embed(em).keyboard(kb)).await
    } else {
        ctx.reply_with(Reply::embed(em).keyboard(kb)).await
    }
}
