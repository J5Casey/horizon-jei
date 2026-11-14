//! JEI's library API: what other mods call to add recipe categories to the
//! viewer and to open it on a stack. See `docs/category-api.md`.
//!
//! A mod declares JEI as an optional dependency in its `horizon-mod.toml`
//! (`[dependencies] jei = { version = "^0.3", optional = true }`) and calls
//! these under `mod_api::if_installed!(host, jei, { .. })`, so it builds and
//! runs without JEI too. Every call is plain data in, nothing out: no Horizon
//! type crosses the boundary, so a JEI update that keeps these signatures
//! never breaks a caller.
//!
//! The calls only record what they are given. JEI reads it on its next
//! client tick: a changed category rebuilds the viewer's tabs and tooltip
//! counts, a lookup opens the recipe screen. They are safe from any thread
//! and any hook (a server hook in single player included), but a category
//! only shows on the client that published it, so publish from a client
//! hook (`recipes_received`, `client_tick_post`) on every client.

use std::sync::{Mutex, MutexGuard};

pub use super::{Category, Chance, ChanceRecipe, Stack};

/// This API's version: 1 is `set_category`, `remove_category`,
/// `categories`, `show_recipes` and `show_uses`. A later version only adds.
pub const API_VERSION: u32 = 1;

#[derive(Default)]
struct Registry {
    /// In the order they were first published: the tabs' order after the
    /// stations'.
    categories: Vec<Category>,
    /// Moves on every change, so the viewer knows to rebuild.
    version: u64,
    /// Lookups to run on JEI's next tick: a stack, and whether its uses.
    lookups: Vec<(Stack, bool)>,
}

static REGISTRY: Mutex<Registry> = Mutex::new(Registry {
    categories: Vec::new(),
    version: 0,
    lookups: Vec::new(),
});

fn registry() -> MutexGuard<'static, Registry> {
    REGISTRY
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Publishes a recipe category, or replaces the one with the same `id`
/// (keeping its place among the tabs). `id` is namespaced by the publishing
/// mod (`salvage:sifting`). A category with no recipes still shows its tab
/// for its catalysts' uses; `remove_category` takes it away.
pub fn set_category(category: Category) {
    let mut r = registry();
    match r.categories.iter_mut().find(|c| c.id == category.id) {
        Some(old) if *old == category => return,
        Some(old) => *old = category,
        None => r.categories.push(category),
    }
    r.version += 1;
}

/// Takes the category `id` away; nothing when there is none.
pub fn remove_category(id: &str) {
    let mut r = registry();
    let before = r.categories.len();
    r.categories.retain(|c| c.id != id);
    if r.categories.len() != before {
        r.version += 1;
    }
}

/// The categories published now, in tab order.
pub fn categories() -> Vec<Category> {
    registry().categories.clone()
}

/// Opens JEI's recipe screen on what makes `stack` (as R over it does), over
/// the open screen or, with none open, on a screen of JEI's own.
pub fn show_recipes(stack: Stack) {
    registry().lookups.push((stack, false));
}

/// Opens JEI's recipe screen on what `stack` is used in (as U does).
pub fn show_uses(stack: Stack) {
    registry().lookups.push((stack, true));
}

/// The categories, when they changed since `seen` (JEI's own read).
#[doc(hidden)]
pub fn changed_since(seen: Option<u64>) -> Option<(u64, Vec<Category>)> {
    let r = registry();
    (seen != Some(r.version)).then(|| (r.version, r.categories.clone()))
}

/// The lookups other mods asked for since the last call (JEI's own read).
#[doc(hidden)]
pub fn take_lookups() -> Vec<(Stack, bool)> {
    std::mem::take(&mut registry().lookups)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn category(id: &str, title: &str) -> Category {
        Category {
            id: id.to_owned(),
            title: title.to_owned(),
            icon: Stack::item("minecraft:stone"),
            ..Category::default()
        }
    }

    #[test]
    fn categories_replace_by_id_and_keep_their_place() {
        let seen = changed_since(None).map(|(v, _)| v);
        set_category(category("test:a", "A"));
        set_category(category("test:b", "B"));
        set_category(category("test:a", "A2"));
        let (v, all) = changed_since(seen).expect("changed");
        let ours: Vec<_> = all.iter().filter(|c| c.id.starts_with("test:")).collect();
        assert_eq!(ours.len(), 2);
        assert_eq!(ours[0].title, "A2");
        assert_eq!(ours[1].id, "test:b");
        // The same category again changes nothing.
        set_category(category("test:b", "B"));
        assert!(changed_since(Some(v)).is_none());
        remove_category("test:a");
        remove_category("test:b");
        assert!(changed_since(Some(v)).is_some());
        show_uses(Stack::item("minecraft:stick"));
        assert_eq!(take_lookups(), vec![(Stack::item("minecraft:stick"), true)]);
        assert!(take_lookups().is_empty());
    }
}
