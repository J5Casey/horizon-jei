// The mod's settings page: "Click gives items", for now.
fn jei_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::jei as m;
    if mod_id != m::ID {
        return;
    }
    let on = m::give(game.native_setting(m::ID, "give").as_deref());
    buttons.push(m::give_button(on));
}
