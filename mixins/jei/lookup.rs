// Another mod asks for a stack's recipes, uses, or a key over it (the quest
// book's items): the recipe screen opens as for JEI's own list.
fn jei_lookup(
    game: &mut crate::game::Game<'_>,
    stack: &crate::game::native_hooks::NativeStack,
    lookup: &crate::game::native_hooks::NativeRecipeLookup,
    handled: &mut bool,
) {
    if *handled {
        return;
    }
    crate::game::jei_glue::lookup(game, stack, lookup, handled);
}
