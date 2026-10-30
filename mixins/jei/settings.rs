// The mod's settings page: one button, for now.
fn jei_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::jei as m;
    if mod_id != m::ID {
        return;
    }
    let on = m::debug(game.native_setting(m::ID, "debug").as_deref());
    buttons.push(m::debug_button(on));
}
