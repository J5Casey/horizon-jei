//! JEI for Horizon: an item and recipe viewer, a port of Just Enough Items,
//! as a native mod on the arismoko/horizon mod API (`horizon-mod-api`, lib
//! `mod_api`).
//!
//! Every item lists beside container screens (`panel_items`, drawn in JEI
//! 1.12's classic look by `panel_style`); R and U over a stack open JEI's
//! recipe screen over the open screen, where "+" fills the crafting grid
//! (`transfer_recipe`, the recipe's id from `recipe_book_ids`) and marks in
//! red what the server says it lacks (`transfer_recipe_check`). Bookmarks,
//! tooltip hints, `/jei` and the settings on its page in Options > Mods.
//!
//! It is also a library: other mods add recipe categories and open the
//! viewer through [`api`] (`docs/category-api.md`).
#![deny(unsafe_code)]

mod catalog;
pub mod engine;
mod glue;

/// What other mods call: recipe categories and lookups. See
/// `docs/category-api.md`.
pub use engine::api;

use engine as jei;
use mod_api::{
    ClientHost, CommandEvent, Flow, GuiRenderPostEvent, Hooks, ItemTooltipEvent, KeyRegistration,
    PanelEventEvent, RecipesReceivedEvent, RegisterClientCommandsEvent, Registrar,
    RegistrationContext, RenderFramePostEvent, RenderTooltipPreEvent, ScreenClosingEvent,
    ScreenKeyPressedPreEvent, ScreenMouseButtonPressedPreEvent, ScreenMouseButtonReleasedPreEvent,
    ScreenMouseDraggedPreEvent, ScreenMouseScrolledPreEvent, ScreenRenderPostEvent, SettingSpec,
};

#[derive(Default)]
struct Jei;

fn flow(taken: bool) -> Flow {
    if taken { Flow::Cancel } else { Flow::Continue }
}

fn toggle(key: &str, default: bool) -> SettingSpec {
    SettingSpec::toggle(key, format!("jei.setting.{key}"), default)
        .tooltip(format!("jei.setting.{key}.tooltip"))
}

#[mod_api::horizon_mod]
impl Hooks for Jei {
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        let settings = [
            toggle("give", false),
            SettingSpec::choice(
                "language",
                "jei.setting.language",
                ["jei.language.en", "jei.language.es"],
                0,
            ),
            toggle("hints", true),
            toggle("centred_search", true),
        ];
        for setting in settings {
            if let Err(e) = r.register_setting(setting) {
                r.registration_refusal(format!("JEI's setting: {e:?}"));
            }
        }
        for [id, _, key] in jei::KEYS {
            r.register_key_mapping(KeyRegistration::new(id, key).label(&format!("jei.key.{id}")));
        }
    }

    fn render_frame_post(&self, host: &mut dyn ClientHost, _event: &mut RenderFramePostEvent) {
        glue::frame(host);
    }

    fn screen_render_post(&self, host: &dyn ClientHost, event: &mut ScreenRenderPostEvent<'_>) {
        glue::screen_render_post(host, event.draw, event.screen);
    }

    fn gui_render_post(&self, host: &dyn ClientHost, event: &mut GuiRenderPostEvent<'_>) {
        glue::gui_render_post(host, event.batch);
    }

    fn screen_mouse_button_pressed_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenMouseButtonPressedPreEvent<'_>,
    ) -> Flow {
        flow(glue::screen_mouse_pressed(
            host,
            event.screen,
            event.button,
            event.modifiers,
        ))
    }

    fn screen_mouse_button_released_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenMouseButtonReleasedPreEvent<'_>,
    ) -> Flow {
        flow(glue::screen_mouse_released(
            host,
            event.screen,
            event.button,
        ))
    }

    fn screen_mouse_dragged_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenMouseDraggedPreEvent<'_>,
    ) -> Flow {
        flow(glue::screen_mouse_dragged(host, event.screen, event.button))
    }

    fn screen_mouse_scrolled_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenMouseScrolledPreEvent<'_>,
    ) -> Flow {
        flow(glue::screen_mouse_scrolled(
            host,
            event.screen,
            event.scroll,
        ))
    }

    fn screen_key_pressed_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenKeyPressedPreEvent<'_>,
    ) -> Flow {
        flow(glue::screen_key_pressed(host, event.screen, event.key))
    }

    fn screen_closing(&self, host: &dyn ClientHost, _event: &mut ScreenClosingEvent<'_>) {
        glue::screen_closing(host);
    }

    fn render_tooltip_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut RenderTooltipPreEvent<'_>,
    ) -> Flow {
        let at = [event.place.x as f32, event.place.y as f32];
        flow(glue::tooltip_pre(host, at))
    }

    fn item_tooltip(&self, host: &dyn ClientHost, event: &mut ItemTooltipEvent<'_>) {
        glue::tooltip(host, event.stack, event.lines);
    }

    fn panel_event(&self, host: &mut dyn ClientHost, event: &mut PanelEventEvent<'_>) {
        if event.mod_id == jei::ID {
            glue::panel_event(host, event.event);
        }
    }

    fn register_client_commands(
        &self,
        _host: &mut dyn ClientHost,
        event: &mut RegisterClientCommandsEvent<'_>,
    ) {
        event.commands.push(mod_api::Command::new("jei"));
        event
            .commands
            .push(mod_api::Command::new("jei <args:text>"));
    }

    fn command(&self, host: &mut dyn ClientHost, event: &mut CommandEvent<'_>) -> Flow {
        flow(glue::command(host, event.command))
    }

    // No `datapack_sync`: with the server's recipes synced, the window's
    // recipe book numbers them unlike the server, and the "+" (which finds its
    // recipe by that number) moves the wrong recipe or none. The viewer shows
    // the window's own recipes until that is fixed in Horizon; the "+" still
    // moves what the server's recipe of that id takes.

    fn recipes_received(&self, host: &mut dyn ClientHost, _event: &mut RecipesReceivedEvent<'_>) {
        if host.on(jei::ID) {
            glue::recipes_received();
        }
    }
}
