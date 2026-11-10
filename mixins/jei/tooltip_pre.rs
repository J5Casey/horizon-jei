// While JEI's recipe screen is open, the screen under it draws no item
// tooltips (only the item list beside it does).
fn jei_tooltip_pre(game: &crate::game::Game<'_>, x: f32, y: f32, cancel: &mut bool) {
    crate::game::jei_glue::tooltip_pre(game, [x, y], cancel);
}
