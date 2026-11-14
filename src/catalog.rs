//! The game's tables as the viewer takes them: every item (id, name, the
//! tags the recipes name) and the vanilla recipe kinds it shows, from
//! `item_keys` and `recipes_of_type`, with each crafting recipe's book id
//! (`recipe_book_ids`) for its "+".

use crate::engine::{Ingredient, Item, Kind, Recipe};
use mod_api::{ClientHost, Identifier, ItemKey};
use std::collections::{BTreeMap, HashMap};

/// The recipe types the viewer shows, and the kind each is.
const TYPES: &[(&str, Kind)] = &[
    ("minecraft:crafting_shaped", Kind::CraftingShaped),
    ("minecraft:crafting_shapeless", Kind::CraftingShapeless),
    ("minecraft:smelting", Kind::Smelting),
    ("minecraft:blasting", Kind::Blasting),
    ("minecraft:smoking", Kind::Smoking),
    ("minecraft:campfire_cooking", Kind::Campfire),
    ("minecraft:stonecutting", Kind::Stonecutting),
    ("minecraft:smithing_transform", Kind::Smithing),
    ("minecraft:smithing_trim", Kind::Smithing),
];

/// The items and recipes, and each item's key for drawing.
pub struct Catalog {
    pub items: Vec<Item>,
    pub recipes: Vec<Recipe>,
    pub keys: HashMap<String, ItemKey>,
}

/// `minecraft:` for a bare id, as vanilla reads one.
fn qualified(id: &str) -> String {
    if id.contains(':') {
        id.to_owned()
    } else {
        format!("minecraft:{id}")
    }
}

/// The item's name in the player's language: its `item.` key, else its
/// `block.` key, else its path in words.
fn item_name(host: &dyn ClientHost, id: &str) -> String {
    let (ns, path) = id.split_once(':').unwrap_or(("minecraft", id));
    for prefix in ["item", "block"] {
        if let Some(name) = host.translate(&format!("{prefix}.{ns}.{path}")) {
            return name;
        }
    }
    let mut words = String::new();
    for (i, word) in path.split('_').enumerate() {
        if i > 0 {
            words.push(' ');
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            words.extend(first.to_uppercase());
            words.push_str(chars.as_str());
        }
    }
    words
}

/// Every `#tag` a recipe file (as JSON text) names.
fn tags_named(json: &str, out: &mut Vec<String>) {
    for part in json.split("\"#").skip(1) {
        if let Some(tag) = part.split('"').next()
            && !tag.is_empty()
        {
            out.push(qualified(tag));
        }
    }
}

struct Builder<'a> {
    index: &'a HashMap<String, u32>,
}

impl Builder<'_> {
    fn slot(&self, ingredient: &mod_api::Ingredient) -> Vec<Ingredient> {
        let mut ids: Vec<&String> = ingredient.order.iter().collect();
        if ids.is_empty() {
            ids = ingredient.items.iter().collect();
            ids.sort();
        }
        ids.into_iter()
            .filter_map(|id| self.index.get(id.as_str()))
            .map(|&item| Ingredient {
                item,
                count: 1,
                components: String::new(),
            })
            .collect()
    }

    fn item(&self, id: &str, count: u32) -> Option<Ingredient> {
        self.index.get(&qualified(id)).map(|&item| Ingredient {
            item,
            count,
            components: String::new(),
        })
    }

    /// The file's `result`: `{"id": .., "count": ..}` or an id.
    fn json_result(&self, id: Option<&str>, count: Option<u64>) -> Vec<Ingredient> {
        let count = count.unwrap_or(1) as u32;
        id.and_then(|id| self.item(id, count)).into_iter().collect()
    }
}

/// The world's tables now; None while the game has no items yet.
pub fn build(host: &mut dyn ClientHost) -> Option<Catalog> {
    let keys = host.item_keys();
    let mut items = Vec::with_capacity(keys.len());
    let mut by_id = HashMap::with_capacity(keys.len());
    for key in keys {
        let id = key.id(host);
        if id == "minecraft:air" || by_id.contains_key(&id) {
            continue;
        }
        items.push(Item {
            name: item_name(host, &id),
            id: id.clone(),
            tags: Vec::new(),
        });
        by_id.insert(id, key);
    }
    if items.is_empty() {
        return None;
    }
    let index: HashMap<String, u32> = (items.iter().enumerate())
        .map(|(i, item)| (item.id.clone(), i as u32))
        .collect();
    let b = Builder { index: &index };
    let mut recipes = Vec::new();
    let mut tags = Vec::new();
    for &(kind_id, kind) in TYPES {
        for r in host.recipes_of_type(kind_id).iter() {
            tags_named(&r.json.to_string(), &mut tags);
            let mut recipe = Recipe {
                kind,
                width: 0,
                height: 0,
                slots: Vec::new(),
                results: (r.results.iter())
                    .filter_map(|s| b.item(&s.item().id(host), s.count()))
                    .collect(),
                cooking_time: r.cooking_time.unwrap_or(0),
                experience: r.experience.unwrap_or(0.0),
                book_id: -1,
            };
            match kind {
                Kind::CraftingShaped => {
                    let rows: Vec<Vec<char>> = (r.json.get("pattern").and_then(|p| p.as_array()))
                        .into_iter()
                        .flatten()
                        .filter_map(|row| row.as_str().map(|s| s.chars().collect()))
                        .collect();
                    let Some((w, h, cells)) = crate::engine::pattern(&rows) else {
                        continue;
                    };
                    let mut given = r.ingredients.iter();
                    recipe.width = w;
                    recipe.height = h;
                    recipe.slots = cells
                        .into_iter()
                        .map(|used| match used {
                            true => given.next().map(|i| b.slot(i)).unwrap_or_default(),
                            false => Vec::new(),
                        })
                        .collect();
                }
                Kind::Stonecutting => {
                    if let Some(mod_api::RecipeRule::Stonecutting { input, result }) = &r.rule {
                        recipe.slots = vec![b.slot(input)];
                        recipe.results = (b.item(&result.item().id(host), result.count()))
                            .into_iter()
                            .collect();
                    }
                }
                Kind::Smithing => match &r.rule {
                    Some(mod_api::RecipeRule::SmithingTransform {
                        template,
                        base,
                        addition,
                        ..
                    }) => {
                        let opt = |i: &Option<mod_api::Ingredient>| {
                            i.as_ref().map(|i| b.slot(i)).unwrap_or_default()
                        };
                        recipe.slots = vec![opt(template), b.slot(base), opt(addition)];
                        let result = r.json.get("result");
                        recipe.results = b.json_result(
                            result
                                .and_then(|v| v.get("id").and_then(|i| i.as_str()).or(v.as_str())),
                            result.and_then(|v| v.get("count")).and_then(|c| c.as_u64()),
                        );
                    }
                    Some(mod_api::RecipeRule::SmithingTrim {
                        template,
                        base,
                        addition,
                        ..
                    }) => {
                        recipe.slots = vec![b.slot(template), b.slot(base), b.slot(addition)];
                        // A trim on any of the armour it takes.
                        recipe.results = b.slot(base);
                    }
                    _ => continue,
                },
                _ => recipe.slots = r.ingredients.iter().map(|i| b.slot(i)).collect(),
            }
            if recipe.slots.iter().all(Vec::is_empty) || recipe.results.is_empty() {
                continue;
            }
            if kind.station() == crate::engine::Station::Crafting {
                recipe.book_id = host.recipe_book_ids(&r.id).first().copied().unwrap_or(-1);
            }
            recipes.push(recipe);
        }
    }
    // Each `#tag` the recipes name, on its items, for the `#` search.
    tags.sort();
    tags.dedup();
    let mut item_tags: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    let tag_set = host.tags();
    for tag in &tags {
        let Some(key) = Identifier::parse(tag).ok().and_then(|id| tag_set.key(&id)) else {
            continue;
        };
        for member in tag_set.items(key).unwrap_or_default() {
            if let Some(&i) = index.get(&member.id(host)) {
                item_tags.entry(i).or_default().push(tag.clone());
            }
        }
    }
    for (i, tags) in item_tags {
        items[i as usize].tags = tags;
    }
    Some(Catalog {
        items,
        recipes,
        keys: by_id,
    })
}
