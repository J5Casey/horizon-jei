//! The game's tables as the viewer takes them: every item (id, name, the
//! tags the recipes name) and the vanilla recipe kinds it shows, from
//! `item_keys` and `recipes_of_type`, with an id of the catalog's on each
//! crafting recipe for its "+", which names the recipe (`Catalog::books`):
//! the "+" and its check go by the recipe's id (`transfer_recipe_id`,
//! `transfer_recipe_check`), learned or not.

use crate::engine::{Ingredient, Item, Kind, Recipe};
use mod_api::{ClientHost, Identifier, ItemKey, serde_json};
use std::collections::{BTreeMap, HashMap};

/// The recipe types the viewer reads (`recipes_of_type` takes a recipe's
/// type, not its file's serializer: every crafting serializer is
/// `minecraft:crafting`, both smithing ones `minecraft:smithing`), and
/// `kind` names the kind of each recipe from its file's `type`.
const TYPES: &[&str] = &[
    "minecraft:crafting",
    "minecraft:smelting",
    "minecraft:blasting",
    "minecraft:smoking",
    "minecraft:campfire_cooking",
    "minecraft:stonecutting",
    "minecraft:smithing",
    "minecraft:brewing",
];

/// The kind a recipe file's serializer (`type`) shows as, or None for one
/// the viewer doesn't draw (special crafting, transmute).
fn kind(serializer: &str) -> Option<Kind> {
    Some(match serializer {
        "minecraft:crafting_shaped" => Kind::CraftingShaped,
        "minecraft:crafting_shapeless" => Kind::CraftingShapeless,
        "minecraft:smelting" => Kind::Smelting,
        "minecraft:blasting" => Kind::Blasting,
        "minecraft:smoking" => Kind::Smoking,
        "minecraft:campfire_cooking" => Kind::Campfire,
        "minecraft:stonecutting" => Kind::Stonecutting,
        "minecraft:smithing_transform" | "minecraft:smithing_trim" => Kind::Smithing,
        "minecraft:brewing" => Kind::Brewing,
        _ => return None,
    })
}

/// The items and recipes, and each item's key for drawing.
pub struct Catalog {
    pub items: Vec<Item>,
    pub recipes: Vec<Recipe>,
    pub keys: HashMap<String, ItemKey>,
    /// The recipe each crafting recipe's catalog id (`Recipe::book_id`) names.
    pub books: HashMap<i32, String>,
}

/// A JSON value as SNBT, as `/give`'s component syntax reads it: object
/// keys bare when they can be, strings in double quotes.
fn snbt(value: &serde_json::Value) -> String {
    use serde_json::Value;
    let quoted = |s: &str| format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""));
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => quoted(s),
        Value::Array(a) => format!("[{}]", a.iter().map(snbt).collect::<Vec<_>>().join(",")),
        Value::Object(o) => {
            let key = |k: &str| {
                let bare = !k.is_empty()
                    && k.chars()
                        .all(|c| c.is_ascii_alphanumeric() || "_-.+".contains(c));
                if bare { k.to_owned() } else { quoted(k) }
            };
            let fields: Vec<String> = (o.iter())
                .map(|(k, v)| format!("{}:{}", key(k), snbt(v)))
                .collect();
            format!("{{{}}}", fields.join(","))
        }
    }
}

/// A recipe file's `components` object as an item argument's component
/// list (`[minecraft:potion_contents={potion:"minecraft:water"}]`), the form
/// the panel's stacks take; "" for none.
pub fn components(json: Option<&serde_json::Value>) -> String {
    let Some(object) = json.and_then(|c| c.as_object()).filter(|o| !o.is_empty()) else {
        return String::new();
    };
    let parts: Vec<String> = (object.iter())
        .map(|(k, v)| format!("{}={}", qualified(k), snbt(v)))
        .collect();
    format!("[{}]", parts.join(","))
}

/// A stack's components in the list's form, so a tooltip counts a potion's
/// own recipes (the viewer keeps only the subtype ones); "" for none.
pub fn stack_components(stack: &mod_api::ItemStack) -> String {
    use mod_api::Tag;
    fn json(tag: &Tag) -> serde_json::Value {
        match tag {
            Tag::Byte(n) => (*n).into(),
            Tag::Short(n) => (*n).into(),
            Tag::Int(n) => (*n).into(),
            Tag::Long(n) => (*n).into(),
            Tag::String(s) => s.to_string_lossy().into(),
            Tag::List(l) => l.iter().map(json).collect(),
            Tag::Compound(c) => (c.iter())
                .map(|(k, v)| (k.to_string_lossy(), json(v)))
                .collect::<serde_json::Map<_, _>>()
                .into(),
            _ => serde_json::Value::Null,
        }
    }
    let Some(patch) = stack.components().map(|c| c.as_nbt()) else {
        return String::new();
    };
    let kept: serde_json::Map<String, serde_json::Value> = (patch.iter())
        .map(|(k, v)| (k.to_string_lossy(), json(v)))
        .collect();
    components(Some(&kept.into()))
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

    /// A brewing recipe's `input` or `reagent` (`PotionIngredient`): its
    /// `item` (an id, a `#tag` or a list) as the game reads an ingredient, one
    /// stack for each potion its `potion_contents.potions` allows.
    fn potion_slot(&self, host: &dyn ClientHost, json: &serde_json::Value) -> Vec<Ingredient> {
        let Some(item) = json.get("item") else {
            return Vec::new();
        };
        let Ok(Some(ingredient)) = host.recipe_ingredient(&item.to_string()) else {
            return Vec::new();
        };
        let items = self.slot(&ingredient);
        let potions: Vec<&str> = match json.pointer("/potion_contents/potions") {
            Some(serde_json::Value::String(p)) => vec![p.as_str()],
            Some(serde_json::Value::Array(a)) => a.iter().filter_map(|p| p.as_str()).collect(),
            _ => return items,
        };
        let mut out = Vec::new();
        for potion in potions {
            let contents =
                serde_json::json!({ "minecraft:potion_contents": { "potion": qualified(potion) } });
            let components = components(Some(&contents));
            out.extend(items.iter().map(|i| Ingredient {
                components: components.clone(),
                ..i.clone()
            }));
        }
        out
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
    let mut books = HashMap::new();
    let mut tags = Vec::new();
    for &type_id in TYPES {
        for r in host.recipes_of_type(type_id).iter() {
            let Some(kind) = r.json.get("type").and_then(|t| t.as_str()).and_then(kind) else {
                continue;
            };
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
                Kind::Brewing => {
                    // A file of the game's own: the bottle and the reagent,
                    // and the bottle it makes with its components.
                    let json = &r.json;
                    recipe.slots = vec![
                        b.potion_slot(host, &json["input"]),
                        b.potion_slot(host, &json["reagent"]),
                    ];
                    let output = &json["output"];
                    recipe.results = (b.item(
                        output["id"].as_str().unwrap_or("minecraft:potion"),
                        output["count"].as_u64().unwrap_or(1) as u32,
                    ))
                    .map(|i| Ingredient {
                        components: components(output.get("components")),
                        ..i
                    })
                    .into_iter()
                    .collect();
                }
                _ => recipe.slots = r.ingredients.iter().map(|i| b.slot(i)).collect(),
            }
            if recipe.slots.iter().all(Vec::is_empty) || recipe.results.is_empty() {
                continue;
            }
            if kind.station() == crate::engine::Station::Crafting {
                recipe.book_id = books.len() as i32;
                books.insert(recipe.book_id, r.id.clone());
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
        books,
    })
}
