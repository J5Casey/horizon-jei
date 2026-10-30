// A click on the settings page's button: turn debug on or off, and keep it.
fn jei_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::jei as m;
    if mod_id != m::ID || button != 0 {
        return;
    }
    let on = m::debug(game.native_setting(m::ID, "debug").as_deref());
    game.native_set_setting(m::ID, "debug", if on { "false" } else { "true" });
}
