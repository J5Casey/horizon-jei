// The item list and the recipe pages in the panel (Phase 0 spike, kept as the
// base of Phase 2): every item the game knows, filtered by the search box, and
// R/U over any stack to open its recipes or uses.
use crate::game::native_hooks::{
    NativePanelEvent, NativeRecipe, NativeRecipeKind, NativeRecipeLayout, NativeRecipeStack,
    NativeRecipeView, NativeRecipes, NativeStack,
};
use std::cell::RefCell;
use std::sync::Arc;

/// What the mod keeps between hooks: the world's tables and the search text.
struct State {
    catalog: Option<Arc<NativeRecipes>>,
    search: String,
}

thread_local! {
    static STATE: RefCell<State> = const { RefCell::new(State {
        catalog: None,
        search: String::new(),
    }) };
}

/// Builds the catalogue once per world and shows the list in the panel.
pub(super) fn sync(game: &mut crate::game::Game<'_>) {
    if !game.native_on(crate::game::jei::ID) {
        return;
    }
    if game.native_player().is_none() {
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.catalog = None;
            s.search.clear();
        });
        return;
    }
    let built = STATE.with(|s| s.borrow().catalog.is_some());
    if built {
        return;
    }
    let Some(catalog) = game.native_recipes() else {
        return;
    };
    let search = STATE.with(|s| s.borrow().search.clone());
    let stacks = list(&catalog, &search);
    game.native_panel_items(crate::game::jei::ID, stacks);
    STATE.with(|s| s.borrow_mut().catalog = Some(catalog));
}

/// The panel's events: the search box, R/U over a stack, and Back.
pub(super) fn event(game: &mut crate::game::Game<'_>, event: &NativePanelEvent) {
    if !game.native_on(crate::game::jei::ID) {
        return;
    }
    match event {
        NativePanelEvent::Search(text) => {
            let Some(catalog) = catalog() else {
                return;
            };
            STATE.with(|s| s.borrow_mut().search = text.clone());
            game.native_panel_items(crate::game::jei::ID, list(&catalog, text));
        }
        NativePanelEvent::Key { key, stack } => {
            let recipes = game.native_mod_key(crate::game::jei::ID, "recipe", "key.keyboard.r");
            let uses = game.native_mod_key(crate::game::jei::ID, "uses", "key.keyboard.u");
            let Some(catalog) = catalog() else {
                return;
            };
            if key == &recipes {
                open(game, &catalog, stack, false);
            } else if key == &uses {
                open(game, &catalog, stack, true);
            }
        }
        NativePanelEvent::Back => {
            game.native_panel_recipes(crate::game::jei::ID, "", Vec::new(), 0, Vec::new());
        }
        _ => {}
    }
}

fn catalog() -> Option<Arc<NativeRecipes>> {
    STATE.with(|s| s.borrow().catalog.clone())
}

/// The items the search keeps: JEI's words, phrases, exclusions and prefixes.
fn list(catalog: &NativeRecipes, search: &str) -> Vec<NativeStack> {
    use crate::game::jei as m;
    let stack = |item: &str| NativeStack {
        item: item.to_owned(),
        count: 1,
        components: String::new(),
    };
    let items: Vec<NativeStack> = catalog
        .items
        .iter()
        .filter(|item| item.id != "minecraft:air")
        .filter(|item| {
            m::keeps(
                &m::SearchItem {
                    name: &item.name,
                    id: &item.id,
                    tags: &item.tags,
                },
                search,
            )
        })
        .map(|item| stack(&item.id))
        .collect();
    // The panel goes away when its list is empty, and with it the search box
    // and the events: keep one invisible slot (count 0) so a search that finds
    // nothing can still be cleared.
    if items.is_empty() {
        return vec![NativeStack {
            item: "minecraft:air".to_owned(),
            count: 0,
            components: String::new(),
        }];
    }
    items
}

/// Opens the recipes (`uses` false) or uses of an item.
fn open(
    game: &mut crate::game::Game<'_>,
    catalog: &NativeRecipes,
    stack: &NativeStack,
    uses: bool,
) {
    let views: Vec<NativeRecipeView> = catalog
        .recipes
        .iter()
        // A recipe with no ingredients says nothing in a list of recipes.
        .filter(|r| !r.slots.is_empty())
        .filter(|r| matches(catalog, r, stack, uses))
        .take(256)
        .map(|r| view(catalog, r))
        .collect();
    if views.is_empty() {
        game.native_chat(if uses {
            "No uses for this item"
        } else {
            "No recipes for this item"
        });
        return;
    }
    let name = catalog
        .items
        .iter()
        .find(|i| i.id == stack.item)
        .map_or(stack.item.as_str(), |i| i.name.as_str());
    let title = format!("{}: {name}", if uses { "Uses" } else { "Recipes" });
    let tab = NativeStack {
        item: stack.item.clone(),
        count: 1,
        components: stack.components.clone(),
    };
    game.native_panel_recipes(crate::game::jei::ID, &title, vec![tab], 0, views);
}

fn matches(catalog: &NativeRecipes, recipe: &NativeRecipe, want: &NativeStack, uses: bool) -> bool {
    if uses {
        recipe
            .slots
            .iter()
            .flatten()
            .any(|s| same(catalog, s, want))
    } else {
        recipe.results.iter().any(|s| same(catalog, s, want))
    }
}

fn same(catalog: &NativeRecipes, stack: &NativeRecipeStack, want: &NativeStack) -> bool {
    let same_item = catalog
        .items
        .get(stack.item as usize)
        .is_some_and(|item| item.id == want.item);
    same_item && (want.components.is_empty() || stack.components == want.components)
}

fn convert(catalog: &NativeRecipes, stack: &NativeRecipeStack) -> NativeStack {
    NativeStack {
        item: catalog
            .items
            .get(stack.item as usize)
            .map_or_else(|| "minecraft:air".to_owned(), |item| item.id.clone()),
        count: stack.count,
        components: stack.components.clone(),
    }
}

fn view(catalog: &NativeRecipes, recipe: &NativeRecipe) -> NativeRecipeView {
    let slots = recipe
        .slots
        .iter()
        .map(|slot| slot.iter().map(|s| convert(catalog, s)).collect())
        .collect();
    let results = recipe.results.iter().map(|s| convert(catalog, s)).collect();
    let (layout, width, height, note) = match recipe.kind {
        NativeRecipeKind::CraftingShaped => (
            NativeRecipeLayout::Grid,
            recipe.width,
            recipe.height,
            String::new(),
        ),
        NativeRecipeKind::CraftingShapeless => {
            let n = recipe.slots.len() as u32;
            let width = match n {
                0 | 1 => 1,
                2..=4 => 2,
                _ => 3,
            };
            (
                NativeRecipeLayout::Grid,
                width,
                n.div_ceil(width).max(1),
                "Shapeless".to_owned(),
            )
        }
        NativeRecipeKind::Smelting
        | NativeRecipeKind::Blasting
        | NativeRecipeKind::Smoking
        | NativeRecipeKind::Campfire => {
            let seconds = recipe.cooking_time as f32 / 20.0;
            (
                NativeRecipeLayout::Cooking,
                1,
                1,
                format!(
                    "{seconds} s, {} XP",
                    (recipe.experience * 100.0).round() / 100.0
                ),
            )
        }
        NativeRecipeKind::Stonecutting => (NativeRecipeLayout::Single, 1, 1, String::new()),
        NativeRecipeKind::Smithing => (NativeRecipeLayout::Smithing, 3, 1, String::new()),
        NativeRecipeKind::Brewing => (NativeRecipeLayout::Brewing, 2, 1, String::new()),
    };
    NativeRecipeView {
        layout,
        width,
        height,
        slots,
        results,
        note,
        action: (recipe.book_id >= 0).then_some(false),
    }
}
