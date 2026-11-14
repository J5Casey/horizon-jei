//! The edges between Horizon and the viewer (`crate::engine`): the game's
//! item and recipe tables in, the panel, give and transfer out, and the
//! overlay (tooltips, the bookmark bar, the recipe screen and their clicks)
//! wired to the viewer.
//!
//! Client hooks run on the window's thread, so the state is a thread local.
//! The drawing and input hooks only borrow the host; what they decide to do
//! to the game waits in `pending` for `frame`, which runs once a frame with
//! the host to change it.

use crate::catalog;
use crate::engine::{
    self as jei, Bookmarks, Config, Context, Counts, Event, Layout, Out, Session, Stack, Toggle,
    View, Viewer, screen as geo,
};
use mod_api::{
    ClientHost, ConfigPath, DrawContext, Identifier, ItemKey, ItemStack, PanelEvent, PanelStyle,
    RecipeTransferCheck, RecipeTransferResult, ScreenView, Text,
};
use std::cell::RefCell;
use std::collections::HashMap;

/// Variants one item adds to the list (potions, tipped arrows).
const VARIANTS_PER_ITEM: usize = 8;

/// Shift's GLFW bit.
const SHIFT: u32 = 1;

/// The viewer while a world is open, and what the overlay keeps.
struct State {
    session: Session,
    bookmarks: Bookmarks,
    config: Config,
    /// The config and the bookmarks have been read from disk.
    loaded: bool,
    /// Actions a hook that only borrows the game leaves to the frame.
    pending: Vec<Event>,
    /// The bookmark under the left button, while it is down.
    drag: Option<Drag>,
    /// The screen drawn last (`screen_render_post`): its id and menu type,
    /// for the "+" (which grid is open). None once a frame passes without one.
    screen: Option<(&'static str, Option<String>)>,
    /// A screen was drawn since the last frame began.
    screen_seen: bool,
    /// Where this frame's screen is, for the recipe screen `gui_render_post` draws.
    overlay_at: Option<ScreenAt>,
    /// The same, kept for the input and tooltip hooks of the next frame.
    last_at: Option<ScreenAt>,
    /// The bookmark list's page.
    bookmark_page: usize,
    /// The item list's look last asked of Horizon (`panel_style`).
    panel_style: Option<PanelStyle>,
    /// The server's recipes arrived (`recipes_received`): build the viewer
    /// again from them.
    rebuild: bool,
    /// JEI's recipe screen, while it is open.
    overlay: Option<Overlay>,
    /// The last press went to the recipe screen: its release and drags too.
    overlay_pressed: bool,
    /// The GUI's size at the last frame drawn, for the input hooks' hit tests.
    gui: [f32; 2],
    /// Stacks the overlay's keys bookmark at the next frame.
    to_bookmark: Vec<Stack>,
    /// The other mods' categories' version the viewer was given (`jei::api`).
    categories: Option<u64>,
    /// Each item's key, for drawing it.
    keys: HashMap<String, ItemKey>,
    /// The recipe each book id of the catalog names (`catalog::Catalog::books`).
    books: HashMap<i32, String>,
    /// The "+" under the pointer: the recipe the last frame drew it on, and
    /// the server's latest answer for it (`transfer_recipe_check`).
    hovered_book: Option<i32>,
    answer: Option<(i32, RecipeTransferCheck)>,
}

/// The recipe screen: the viewer's page, the page of it shown, and whether
/// JEI opened a screen of its own to show it (nothing was open).
struct Overlay {
    title: String,
    tabs: Vec<Stack>,
    names: Vec<String>,
    tab: usize,
    views: Vec<View>,
    catalysts: Vec<Stack>,
    subtitle: String,
    page: usize,
    /// The tabs' page the player turned to, else the chosen tab's.
    tab_page: Option<usize>,
    own_screen: bool,
}

/// A bookmark being dragged to another slot.
#[derive(Clone, Copy, Debug)]
struct Drag {
    from: usize,
    /// The pointer moved to another slot: a drop reorders instead of opening.
    moved: bool,
}

impl State {
    fn new() -> Self {
        Self {
            session: Session::new(),
            bookmarks: Bookmarks::default(),
            config: Config::default(),
            loaded: false,
            pending: Vec::new(),
            drag: None,
            screen: None,
            screen_seen: false,
            overlay_at: None,
            last_at: None,
            bookmark_page: 0,
            panel_style: None,
            rebuild: false,
            overlay: None,
            overlay_pressed: false,
            gui: [0.0, 0.0],
            to_bookmark: Vec::new(),
            categories: None,
            keys: HashMap::new(),
            books: HashMap::new(),
            hovered_book: None,
            answer: None,
        }
    }

    fn viewer(&self) -> Option<&Viewer> {
        self.session.viewer()
    }

    fn viewer_mut(&mut self) -> Option<&mut Viewer> {
        self.session.viewer_mut()
    }
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::new());
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

// ---------------------------------------------------------------- files and settings

const CONFIG_FILE: &str = "jei/jei.cfg";
const BOOKMARKS_FILE: &str = "jei/bookmarks.txt";

fn read(host: &dyn ClientHost, path: &str) -> Option<String> {
    let path = ConfigPath::new(path).ok()?;
    let bytes = host.config_files().read(&path).ok()?;
    String::from_utf8(bytes).ok()
}

fn write(host: &dyn ClientHost, path: &str, text: &str) {
    if let Ok(path) = ConfigPath::new(path) {
        let _ = host.config_files().write(&path, text.as_bytes());
    }
}

/// Reads the config and the bookmarks once: `config/jei/jei.cfg` (the
/// list's order and whether the bar shows) and `config/jei/bookmarks.txt`.
/// The tooltip hints and the search box's place are settings on the mod's
/// page in Options > Mods, read every frame.
fn load(host: &dyn ClientHost) {
    if with(|s| s.loaded) {
        return;
    }
    let config = read(host, CONFIG_FILE).map_or_else(Config::default, |t| Config::parse(&t));
    let bookmarks =
        read(host, BOOKMARKS_FILE).map_or_else(Bookmarks::default, |t| jei::bookmarks_parse(&t));
    with(|s| {
        s.config = Config {
            hints: s.config.hints,
            centred_search: s.config.centred_search,
            ..config
        };
        s.bookmarks = bookmarks;
        s.loaded = true;
    });
}

/// The settings of the mod's page into the config.
fn read_settings(host: &dyn ClientHost) {
    let hints = host.setting_bool("hints").unwrap_or(true);
    let centred = host.setting_bool("centred_search").unwrap_or(true);
    with(|s| {
        s.config.hints = hints;
        s.config.centred_search = centred;
    });
}

fn store_config(host: &dyn ClientHost, config: Config) {
    with(|s| s.config = config);
    write(host, CONFIG_FILE, &config.to_text());
}

fn save_bookmarks(host: &dyn ClientHost, bookmarks: &Bookmarks) {
    write(host, BOOKMARKS_FILE, &jei::bookmarks_text(bookmarks));
}

/// Where the config and the bookmarks live, for `/jei config`.
pub fn files() -> String {
    format!("JEI files: config/{CONFIG_FILE} and config/{BOOKMARKS_FILE}")
}

/// The player's language (the mod's page: English or Spanish).
fn lang(host: &dyn ClientHost) -> jei::Lang {
    jei::lang(match host.setting_choice("language") {
        Some(1) => Some("es"),
        _ => None,
    })
}

/// "Click gives items" (the mod's page, or `/jei give`).
fn give_on(host: &dyn ClientHost) -> bool {
    host.setting_bool("give").unwrap_or(false)
}

/// A key the player bound for one of the mod's mappings.
fn key(host: &dyn ClientHost, name: &str, default: &str) -> String {
    host.mod_key(jei::ID, name)
        .map_or_else(|| default.to_owned(), |(key, _)| key)
}

fn say(host: &mut dyn ClientHost, text: &str) {
    let _ = host.chat_line(Text::literal(text));
}

// ---------------------------------------------------------------- the frame

/// Once a frame: builds the viewer once per world (again when the server's
/// recipes arrive), shows the item list in the panel, runs what the input
/// hooks left and asks the server about the "+" under the pointer.
pub fn frame(host: &mut dyn ClientHost) {
    let has_player = host.local_player().is_some();
    let enabled = host.on(jei::ID);
    let needs_build = with(|s| {
        if !std::mem::take(&mut s.screen_seen) {
            s.screen = None;
        }
        let rebuild = has_player && enabled && std::mem::take(&mut s.rebuild);
        if rebuild {
            s.session.clear();
        }
        let needs_build = s.session.update(has_player, enabled);
        if !has_player {
            s.drag = None;
            s.pending.clear();
            s.overlay = None;
        }
        needs_build
    });
    if !enabled || !has_player {
        return;
    }
    read_settings(host);
    if needs_build {
        build(host);
    }
    if with(|s| s.viewer().is_none()) {
        return;
    }
    refresh_categories();
    panel_style(host);
    let (pending, marks) = with(|s| {
        (
            std::mem::take(&mut s.pending),
            std::mem::take(&mut s.to_bookmark),
        )
    });
    for stack in marks {
        bookmark(host, stack);
    }
    for event in pending {
        dispatch(host, event);
    }
    for (stack, uses) in jei::api::take_lookups() {
        dispatch(host, Event::Open { stack, uses });
    }
    // The "+" under the pointer: the server tries it (the chests beside a
    // crafting station counted) and says what lacks; asked again as things change.
    let hovered = with(|s| s.hovered_book.take());
    let answer = hovered.and_then(|book| {
        let now = current_book(host, book);
        host.transfer_recipe_check(now).map(|a| (book, a))
    });
    with(|s| s.answer = answer);
}

/// The book id recipe `book` (an id of the catalog's) has now: a recipe the
/// player learns since takes the server's number.
fn current_book(host: &mut dyn ClientHost, book: i32) -> i32 {
    let Some(id) = with(|s| s.books.get(&book).cloned()) else {
        return book;
    };
    host.recipe_book_ids(&id).first().copied().unwrap_or(book)
}

/// The viewer from the world's tables, and the item list in the panel.
fn build(host: &mut dyn ClientHost) {
    let Some(catalog) = catalog::build(host) else {
        return;
    };
    load(host);
    let sort = with(|s| s.config.sort);
    let recipes = jei::dedupe_recipes(catalog.recipes);
    let found = jei::harvest_variants(&recipes, VARIANTS_PER_ITEM, jei::ITEMS_MAX);
    let (mut viewer, mut out) = Viewer::with_variants(catalog.items, recipes, found, lang(host));
    if sort != viewer.sort() {
        out = viewer.set_sort(sort);
    }
    with(|s| {
        s.session.set_viewer(viewer);
        s.categories = None;
        s.keys = catalog.keys;
        s.books = catalog.books;
        s.panel_style = None;
    });
    refresh_categories();
    panel_style(host);
    if let Err(error) = apply(host, out) {
        say(host, &format!("JEI: {error}"));
    }
}

/// JEI 1.12's item list (`PanelStyle::Classic`): its search box where the
/// player wants it, and laid out beside the recipe screen while that is
/// open, as JEI's list made room for its `RecipesGui`. Asked again only when
/// it changes.
fn panel_style(host: &mut dyn ClientHost) {
    let (config, open, last) = with(|s| (s.config, s.overlay.is_some(), s.panel_style));
    let style = PanelStyle::Classic {
        centred_search: config.centred_search,
        window_width: if open { geo::WIDTH as u16 } else { 0 },
    };
    if last != Some(style) {
        host.panel_style(jei::ID, style);
        with(|s| s.panel_style = Some(style));
    }
}

/// The other mods' categories (`jei::api::set_category`), given to the
/// viewer again whenever they change.
fn refresh_categories() {
    let seen = with(|s| s.categories);
    let Some((version, categories)) = jei::api::changed_since(seen) else {
        return;
    };
    with(|s| {
        if let Some(viewer) = s.viewer_mut() {
            viewer.set_categories(categories);
            s.categories = Some(version);
        }
    });
}

/// The panel's events: search, clicks, keys over a stack.
pub fn panel_event(host: &mut dyn ClientHost, event: &PanelEvent) {
    if !host.on(jei::ID) {
        return;
    }
    let Some(event) = convert_event(host, event) else {
        return;
    };
    // The bookmark key bookmarks the stack under the pointer, wherever it is:
    // the game says which one it is.
    if let Event::Key {
        key: pressed,
        stack,
    } = &event
        && *pressed == key(host, "bookmark", jei::BOOKMARK_KEY)
    {
        bookmark(host, stack.clone());
        return;
    }
    dispatch(host, event);
}

/// Bookmarks a stack, or takes its bookmark away. As JEI's, the list and the
/// tooltip's mark say so; only a full list is told in chat.
fn bookmark(host: &mut dyn ClientHost, stack: Stack) {
    let lang = lang(host);
    load(host);
    let (result, bookmarks) = with(|s| (s.bookmarks.toggle(&stack), s.bookmarks.clone()));
    if result == Toggle::Full {
        say(host, lang.bookmarks_full());
    } else {
        save_bookmarks(host, &bookmarks);
    }
}

/// Takes a bookmark away by its slot, keeping the file.
fn remove_bookmark(host: &dyn ClientHost, index: usize) {
    let changed = with(|s| {
        s.drag = None;
        s.bookmarks.remove(index).map(|_| s.bookmarks.clone())
    });
    if let Some(bookmarks) = changed {
        save_bookmarks(host, &bookmarks);
    }
}

fn pending(event: Event) {
    with(|s| s.pending.push(event));
}

/// The server's recipes came in: the next frame builds the viewer from them.
pub fn recipes_received() {
    with(|s| s.rebuild = true);
}

fn dispatch(host: &mut dyn ClientHost, event: Event) {
    let grid = open_grid();
    let recipe_key = key(host, "recipe", "key.keyboard.r");
    let uses_key = key(host, "uses", "key.keyboard.u");
    let give = give_on(host);
    let lang = lang(host);
    let ctx = Context {
        recipe_key: &recipe_key,
        uses_key: &uses_key,
        give,
        // `give` succeeds in creative only: try, and fall back below.
        creative: give,
        grid,
    };
    let mut fallback = false;
    for out in handle(&event, &ctx, lang) {
        if let Out::Give { item, count } = &out {
            if give_item(host, item, *count).is_err() {
                fallback = true;
            }
            continue;
        }
        if let Err(error) = apply(host, out) {
            say(host, &format!("JEI: {error}"));
        }
    }
    if fallback {
        // Survival: the same click opens the recipes, as JEI without cheat mode.
        let ctx = Context {
            give: false,
            creative: false,
            ..ctx
        };
        for out in handle(&event, &ctx, lang) {
            if let Err(error) = apply(host, out) {
                say(host, &format!("JEI: {error}"));
            }
        }
    }
}

fn handle(event: &Event, ctx: &Context<'_>, lang: jei::Lang) -> Vec<Out> {
    with(|s| {
        let Some(viewer) = s.viewer_mut() else {
            return Vec::new();
        };
        viewer.set_lang(lang);
        viewer.handle(event.clone(), ctx)
    })
}

/// The viewer's size, for `/jei`; none outside a world.
pub fn counts() -> Option<(usize, usize)> {
    with(|s| {
        s.viewer()
            .map(|viewer| (viewer.item_count(), viewer.recipe_count()))
    })
}

fn give_item(host: &mut dyn ClientHost, item: &str, count: u32) -> Result<(), String> {
    let key = item_key(item).ok_or_else(|| format!("no item {item}"))?;
    host.give(key, count)
}

fn apply(host: &mut dyn ClientHost, out: Out) -> Result<(), String> {
    match out {
        Out::Items(stacks) => {
            let stacks = panel_stacks(stacks);
            host.panel_items(jei::ID, stacks);
            Ok(())
        }
        Out::Recipes {
            title,
            tabs,
            tab,
            views,
            names,
            catalysts,
            subtitle,
        } => {
            if views.is_empty() {
                close_overlay(host);
                return Ok(());
            }
            let (open, same) = with(|s| {
                let open = s.overlay.as_ref();
                (
                    open.map(|o| o.own_screen),
                    open.is_some_and(|o| o.title == title && o.tab == tab as usize),
                )
            });
            // Nothing open (a lookup from the world or another mod): JEI's
            // own screen holds it.
            let own_screen = match open {
                Some(own) => own,
                None => with(|s| s.screen.is_none()) && host.mod_screen_open(jei::ID),
            };
            // Notes wrap to the window's width (198, less its border and the
            // recipe's frame), so the geometry, which counts their lines, sizes them.
            let mut views = views;
            for view in &mut views {
                if !view.note.is_empty() {
                    let width = |t: &str| {
                        host.text_width(&Text::literal(t))
                            .map_or(t.len() as f32 * 6.0, |w| w as f32)
                    };
                    view.note = jei::wrap(&view.note, 176.0, width).join("\n");
                }
            }
            with(|s| {
                let page = match &s.overlay {
                    Some(o) if same => o.page,
                    _ => 0,
                };
                s.overlay = Some(Overlay {
                    title,
                    tabs,
                    names,
                    tab: tab as usize,
                    views,
                    catalysts,
                    subtitle,
                    page,
                    tab_page: None,
                    own_screen,
                });
            });
            Ok(())
        }
        Out::Give { item, count } => give_item(host, &item, count),
        Out::Transfer { book_id, all: _ } => {
            // One set: the mod API's transfer has no "as many as the items
            // allow" yet (Shift on "+").
            let book_id = current_book(host, book_id);
            host.transfer_recipe(book_id);
            Ok(())
        }
        Out::Say(text) => {
            say(host, &text);
            Ok(())
        }
    }
}

/// An item's key, from the catalog.
fn item_key(id: &str) -> Option<ItemKey> {
    with(|s| s.keys.get(id).copied())
}

/// The game's stacks. An empty list keeps one invisible slot: the panel (and
/// its search box and events) goes away when the list is empty.
fn panel_stacks(stacks: Vec<Stack>) -> Vec<mod_api::Stack> {
    let out: Vec<mod_api::Stack> = stacks
        .into_iter()
        .filter_map(|s| {
            Some(mod_api::Stack {
                item: item_key(&s.item)?,
                count: s.count,
                components: String::new(),
            })
        })
        .collect();
    if out.is_empty()
        && let Some(air) = item_key("minecraft:barrier")
    {
        return vec![mod_api::Stack {
            item: air,
            count: 0,
            components: String::new(),
        }];
    }
    out
}

/// A viewer stack as the game's.
fn item_stack(stack: &Stack) -> Option<ItemStack> {
    ItemStack::new(item_key(&stack.item)?, stack.count.clamp(1, 99))
}

fn convert_event(host: &dyn ClientHost, event: &PanelEvent) -> Option<Event> {
    let stack_of = |s: &mod_api::Stack| Stack {
        item: s.item.id(host),
        count: s.count,
        components: String::new(),
    };
    Some(match event {
        PanelEvent::Search(text) => Event::Search(text.clone()),
        PanelEvent::Clicked { index, right } => Event::Clicked {
            index: *index as usize,
            right: *right,
        },
        PanelEvent::Key { key, stack } => Event::Key {
            key: key.clone(),
            stack: stack_of(stack),
        },
        PanelEvent::Tab(tab) => Event::Tab(*tab as usize),
        PanelEvent::Back => Event::Back,
        PanelEvent::SlotClicked { stack, right, .. } => Event::SlotClicked {
            stack: stack_of(stack),
            right: *right,
        },
        PanelEvent::Action(index) => Event::Action(*index as usize, false),
        _ => return None,
    })
}

/// The crafting grid of the screen drawn last: 0 when none takes recipes.
/// A mod's menu counts as a 3x3 grid; the server's check of the "+" turns
/// it off where it doesn't fit (`NotIntoMenu`).
fn open_grid() -> u32 {
    with(|s| {
        s.screen.as_ref().map_or(0, |(id, menu)| {
            let mod_grid = menu
                .as_deref()
                .filter(|m| !m.starts_with("minecraft:"))
                .map(|_| 3);
            jei::screen_grid(id, menu.as_deref(), mod_grid)
        })
    })
}

// ---------------------------------------------------------------- settings and commands

/// The next order, shown in the list at once.
pub fn set_sort(host: &mut dyn ClientHost, sort: jei::Sort) {
    load(host);
    let config = Config {
        sort,
        ..with(|s| s.config)
    };
    store_config(host, config);
    let out = with(|s| s.viewer_mut().map(|v| v.set_sort(sort)));
    if let Some(out) = out
        && let Err(error) = apply(host, out)
    {
        say(host, &format!("JEI: {error}"));
    }
}

pub fn sort() -> jei::Sort {
    with(|s| s.config.sort)
}

pub fn toggle_bar(host: &dyn ClientHost) -> Config {
    load(host);
    let mut config = with(|s| s.config);
    config.bookmarks = !config.bookmarks;
    store_config(host, config);
    config
}

/// Empties the bar, keeping the file.
pub fn clear_bookmarks(host: &dyn ClientHost) {
    load(host);
    let bookmarks = with(|s| {
        s.bookmarks.clear();
        s.drag = None;
        s.bookmarks.clone()
    });
    save_bookmarks(host, &bookmarks);
}

/// `/jei ...`: what this build is, "Click gives items", the list's order,
/// the bookmarks and where the files are. True when it was JEI's command.
pub fn command(host: &mut dyn ClientHost, command: &str) -> bool {
    let Some(args) = command.strip_prefix("jei") else {
        return false;
    };
    if !(args.is_empty() || args.starts_with(' ')) {
        return false;
    }
    if args.trim() == "debug" {
        // The "+"'s wiring: the shown recipes' book ids and the server's last check.
        let probe: Vec<String> = [
            "minecraft:stick",
            "minecraft:stick_from_bamboo_item",
            "minecraft:chest",
        ]
        .iter()
        .map(|id| format!("{id}={:?}", host.recipe_book_ids(id)))
        .collect();
        say(host, &format!("JEI debug: now {}", probe.join(" ")));
        let text = with(|s| {
            let books: Vec<Option<i32>> = (0..8)
                .map(|v| s.viewer().and_then(|viewer| viewer.shown_book(v)))
                .collect();
            format!("JEI debug: books {books:?}, check {:?}", s.answer)
        });
        say(host, &text);
        return true;
    }
    let Some(cmd) = jei::command(args) else {
        say(host, jei::COMMAND_USAGE);
        return true;
    };
    let lang = lang(host);
    match cmd {
        jei::Command::About => {
            let text = match counts() {
                Some((items, recipes)) => jei::about_counts(items, recipes),
                None => jei::about(),
            };
            say(host, &text);
        }
        jei::Command::ToggleGive => {
            let on = give_on(host);
            host.set_setting(jei::ID, "give", if on { "false" } else { "true" });
            say(host, &jei::give_button(!on));
        }
        jei::Command::Sort(sort) => {
            set_sort(host, sort);
            say(host, &lang.sort_button(sort));
        }
        jei::Command::SortNone => say(host, &lang.sort_button(sort())),
        jei::Command::BookmarksClear => {
            clear_bookmarks(host);
            say(host, lang.bookmarks_cleared());
        }
        jei::Command::Config => say(host, &files()),
    }
    true
}

// ---------------------------------------------------------------- tooltips

/// Adds the viewer's lines to any item's tooltip: what makes it, what uses it
/// and whether it is bookmarked.
pub fn tooltip(host: &dyn ClientHost, stack: &ItemStack, lines: &mut Vec<(Text, u32)>) {
    if !host.on(jei::ID) {
        return;
    }
    load(host);
    let item = stack.item().id(host);
    let (config, counts, marked) = with(|s| {
        let counts = s
            .viewer()
            .map_or(Counts::default(), |viewer| viewer.counts(&item, ""));
        let marked = s.bookmarks.position(&Stack::item(&item)).is_some();
        (s.config, counts, marked)
    });
    let counts = if config.hints {
        counts
    } else {
        Counts::default()
    };
    let added = jei::tooltip_lines(counts, marked, lang(host));
    if added.is_empty() {
        return;
    }
    lines.push((Text::literal(""), u32::MAX));
    for (text, [r, g, b, a]) in added {
        let color = u32::from_le_bytes([r, g, b, a]);
        lines.push((Text::literal(text), color));
    }
}

/// While the recipe screen is open, the screen under it shows no tooltips:
/// only the item list beside it and its search box do.
pub fn tooltip_pre(host: &dyn ClientHost, at: [f32; 2]) -> bool {
    if !host.on(jei::ID) {
        return false;
    }
    let (open, panel) = with(|s| (s.overlay.is_some(), s.last_at.and_then(|at| at.panel)));
    let on_list = panel.is_some_and(|(left, row)| at[0] >= left || jei::inside(at, row));
    open && !on_list
}

// ---------------------------------------------------------------- drawing

/// JEI's text colours: the header's white, a recipe's grey notes.
const WHITE: u32 = 0xFF_FF_FF_FF;
const TEXT: u32 = 0xFF_40_40_40;
const NOTE: u32 = 0xFF_80_80_80;
/// JEI's bars between the header's buttons, its slot highlight.
const BAR: u32 = 0x30_00_00_00;
const LIT: u32 = 0x7F_FF_FF_FF;
/// JEI's red: over the slots a "+" lacks, and over its button.
const MISSING_SLOT: u32 = 0x66_FF_00_00;
const MISSING_BUTTON: u32 = 0x80_FF_00_00;
const LIST_LIT: u32 = 0x80_FF_FF_FF;
/// Vanilla's `drawDefaultBackground` over the world: its top and bottom.
const SHADE: [u32; 2] = [0xC0_10_10_10, 0xD0_10_10_10];

/// JEI's GUI sprites (1.12's, MIT), from `assets/jei/textures/gui/sprites/`;
/// when a sprite is missing the screen draws the same shapes with fills.
const SPRITES: &str = "jei:gui/sprites/";

/// The host and one hook's drawing context.
struct Paint<'a> {
    host: &'a dyn ClientHost,
    d: &'a mut dyn DrawContext,
}

impl Paint<'_> {
    fn size(&self) -> [f32; 2] {
        self.d.size()
    }

    fn fill(&mut self, r: [f32; 4], argb: u32) {
        self.d.fill(r, argb);
    }

    fn text(&mut self, text: &str, at: [f32; 2], argb: u32, shadow: bool) {
        self.d.text(text, at, argb, shadow);
    }

    fn width(&self, text: &str) -> f32 {
        self.d.text_width(text)
    }

    fn sprite(&mut self, name: &str, src: [f32; 4], dst: [f32; 4]) -> bool {
        self.d.sprite(name, src, dst, None)
    }

    /// A GUI sprite's size in its own pixels.
    fn sprite_size(&self, name: &str) -> Option<[f32; 2]> {
        let id = Identifier::parse(name).ok()?;
        let region = self.host.gui_sprite(&id)?;
        Some([region.size[0] as f32, region.size[1] as f32])
    }

    fn tooltip(&mut self, lines: &[String], at: [f32; 2]) {
        let texts: Vec<Text> = lines
            .iter()
            .filter_map(|l| self.host.text_from_legacy(l).ok())
            .collect();
        if !texts.is_empty() {
            let _ = self.d.gui_tooltip(&texts, at);
        }
    }

    fn stack(&mut self, stack: &Stack, r: [f32; 4]) {
        if let Some(item) = item_stack(stack) {
            let count = (stack.count > 1).then(|| stack.count.to_string());
            self.d
                .gui_draw_item_with(&item, [r[0] + 1.0, r[1] + 1.0], count.as_deref());
        }
    }

    /// Sprite `name` of [`SPRITES`] stretched over `r` as a nine-slice with
    /// borders of `[left, top, right, bottom]` pixels; false when the sprite
    /// isn't there.
    fn nine(&mut self, name: &str, r: [f32; 4], border: [f32; 4]) -> bool {
        let name = format!("{SPRITES}{name}");
        let Some([sw, sh]) = self.sprite_size(&name) else {
            return false;
        };
        let [x, y, w, h] = r;
        let [l, t, rr, b] = border;
        let (l, rr) = (l.min(w / 2.0), rr.min(w / 2.0));
        let (t, b) = (t.min(h / 2.0), b.min(h / 2.0));
        let cols = [
            (0.0, l, x, l),
            (l, sw - l - rr, x + l, w - l - rr),
            (sw - rr, rr, x + w - rr, rr),
        ];
        let rows = [
            (0.0, t, y, t),
            (t, sh - t - b, y + t, h - t - b),
            (sh - b, b, y + h - b, b),
        ];
        for (sx, sw_, dx, dw) in cols {
            for (sy, sh_, dy, dh) in rows {
                if dw > 0.0 && dh > 0.0 {
                    self.sprite(&name, [sx, sy, sw_, sh_], [dx, dy, dw, dh]);
                }
            }
        }
        true
    }

    /// Sprite `name` of [`SPRITES`] whole at `r`; false when it isn't there.
    fn whole(&mut self, name: &str, r: [f32; 4]) -> bool {
        let name = format!("{SPRITES}{name}");
        let Some([w, h]) = self.sprite_size(&name) else {
            return false;
        };
        self.sprite(&name, [0.0, 0.0, w, h], r)
    }

    /// Icon `name` of [`SPRITES`] at its own size, centred in `r`, in JEI's
    /// tint for a button that is `on` and `hovered`.
    fn icon(&mut self, name: &str, r: [f32; 4], on: bool, hovered: bool) -> bool {
        let tint = match (on, hovered) {
            (false, _) => "off",
            (true, true) => "hover",
            (true, false) => "on",
        };
        let name = format!("{SPRITES}icons/{name}_{tint}");
        let Some([w, h]) = self.sprite_size(&name) else {
            return false;
        };
        let at = [
            (r[0] + (r[2] - w) / 2.0).floor(),
            (r[1] + (r[3] - h) / 2.0).floor(),
            w,
            h,
        ];
        self.sprite(&name, [0.0, 0.0, w, h], at)
    }

    /// The recipe window: JEI's grey panel.
    fn frame_box(&mut self, r: [f32; 4]) {
        if self.nine("gui_background", r, [16.0; 4]) {
            return;
        }
        let [x, y, w, h] = r;
        self.fill(r, 0xFF_00_00_00);
        self.fill([x + 1.0, y + 1.0, w - 2.0, h - 2.0], 0xFF_C6_C6_C6);
        self.fill([x + 1.0, y + 1.0, w - 3.0, 2.0], 0xFF_FF_FF_FF);
        self.fill([x + 1.0, y + 1.0, 2.0, h - 3.0], 0xFF_FF_FF_FF);
        self.fill([x + 3.0, y + h - 3.0, w - 4.0, 2.0], 0xFF_55_55_55);
        self.fill([x + w - 3.0, y + 3.0, 2.0, h - 4.0], 0xFF_55_55_55);
    }

    /// A slot: JEI's, else vanilla's colours.
    fn slot_box(&mut self, r: [f32; 4]) {
        if self.nine("slot", r, [4.0; 4]) {
            return;
        }
        let [x, y, w, h] = r;
        self.fill(r, 0xFF_8B_8B_8B);
        self.fill([x, y, w - 1.0, 1.0], 0xFF_37_37_37);
        self.fill([x, y, 1.0, h - 1.0], 0xFF_37_37_37);
        self.fill([x + 1.0, y + h - 1.0, w - 1.0, 1.0], 0xFF_FF_FF_FF);
        self.fill([x + w - 1.0, y + 1.0, 1.0, h - 1.0], 0xFF_FF_FF_FF);
    }

    /// One of JEI's buttons with its icon: a vanilla button, dark when off,
    /// white-edged under the pointer.
    fn button_box(&mut self, r: [f32; 4], name: &str, on: bool, hovered: bool) {
        let hovered = hovered && on;
        let sprite = match (on, hovered) {
            (false, _) => "button_disabled",
            (true, true) => "button_highlight",
            (true, false) => "button_enabled",
        };
        if !self.nine(sprite, r, [2.0; 4]) {
            let fill = if on { 0xFF_6E_6E_6E } else { 0xFF_2B_2B_2B };
            self.fill(
                r,
                if hovered {
                    0xFF_FF_FF_FF
                } else {
                    0xFF_00_00_00
                },
            );
            self.fill([r[0] + 1.0, r[1] + 1.0, r[2] - 2.0, r[3] - 2.0], fill);
        }
        self.icon(name, r, on, hovered);
    }

    /// The text cut with "..." to fit `max`.
    fn fitted(&self, text: &str, max: f32) -> String {
        let mut text = text.to_owned();
        while self.width(&text) > max && text.chars().count() > 3 {
            let keep: String = text.chars().take(text.chars().count() - 4).collect();
            text = format!("{keep}...");
        }
        text
    }

    /// The text centred on `x`, cut with "..." to fit `max`.
    #[allow(clippy::too_many_arguments)]
    fn centred(&mut self, text: &str, x: f32, y: f32, max: f32, argb: u32, shadow: bool) {
        let text = self.fitted(text, max);
        let w = self.width(&text);
        self.text(&text, [(x - w / 2.0).floor(), y], argb, shadow);
    }

    /// Vanilla's darkened background (`drawDefaultBackground`) over `r`, top
    /// to bottom.
    fn shade(&mut self, r: [f32; 4], gui_h: f32) {
        let [x, y, w, h] = r;
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let strips = ((h / 8.0).ceil() as usize).max(1);
        for n in 0..strips {
            let top = y + n as f32 * h / strips as f32;
            let t = ((top + h / strips as f32 / 2.0) / gui_h.max(1.0)).clamp(0.0, 1.0);
            let alpha = (0xC0 as f32 + t * (0xD0 - 0xC0) as f32).round() as u32;
            let color = (alpha << 24) | (SHADE[0] & 0x00_FF_FF_FF);
            self.fill([x, top, w, h / strips as f32], color);
        }
    }

    /// The screen under the recipe screen, darkened as vanilla darkens the
    /// world under a screen: all of it but the item list and its search box.
    fn shade_under(&mut self, panel: Option<(f32, [f32; 4])>) {
        let [gw, gh] = self.size();
        let Some((left, row)) = panel else {
            self.shade([0.0, 0.0, gw, gh], gh);
            return;
        };
        let [rx, ry, rw, rh] = row;
        if rx >= left {
            self.shade([0.0, 0.0, left, gh], gh);
            return;
        }
        self.shade([0.0, 0.0, left, ry], gh);
        self.shade([0.0, ry, rx, gh - ry], gh);
        self.shade([rx + rw, ry, left - rx - rw, gh - ry], gh);
        self.shade([rx, ry + rh, rw, gh - ry - rh], gh);
    }
}

/// Where this frame's screen is, the pointer, and the item list's parts
/// beside it, for the recipe screen `gui_render_post` draws.
#[derive(Clone, Copy, Debug)]
struct ScreenAt {
    rect: [f32; 4],
    mouse: [f32; 2],
    /// The item list's left edge and its search row (`panel_parts`).
    panel: Option<(f32, [f32; 4])>,
}

/// The recipe screen, after the whole GUI: over the screen it opened on
/// (`screen_render_post` said where that is this frame).
pub fn gui_render_post(host: &dyn ClientHost, d: &mut dyn DrawContext) {
    let Some(at) = with(|s| s.overlay_at.take()) else {
        return;
    };
    if !host.on(jei::ID) {
        return;
    }
    // Out of the state while it draws: drawing reads the state too.
    let Some(overlay) = with(|s| s.overlay.take()) else {
        return;
    };
    let mut p = Paint { host, d };
    draw_overlay(&mut p, &at, &overlay);
    with(|s| {
        if s.overlay.is_none() {
            s.overlay = Some(overlay);
        }
    });
}

pub fn screen_render_post(host: &dyn ClientHost, d: &mut dyn DrawContext, screen: &ScreenView<'_>) {
    let gui = d.size();
    with(|s| {
        s.screen = Some((screen.id, screen.menu.map(str::to_owned)));
        s.screen_seen = true;
        s.gui = gui;
    });
    if !host.on(jei::ID) {
        return;
    }
    load(host);
    let (config, open) = with(|s| (s.config, s.overlay.is_some()));
    let window_width = if open { geo::WIDTH } else { 0.0 };
    let panel = panel_parts(gui, screen, config.centred_search, window_width);
    // The recipe screen itself draws in `gui_render_post`, over this screen
    // and whatever a mod drew on it after us; it needs where this one is.
    let at = ScreenAt {
        rect: screen.rect,
        mouse: screen.mouse,
        panel,
    };
    with(|s| {
        s.overlay_at = Some(at);
        s.last_at = Some(at);
    });
    let mut p = Paint { host, d };
    if let Some((_, row)) = panel {
        cheat_wrench(&mut p, row, screen.mouse);
    }
    // Under the recipe screen, the bookmarks draw with it, over its shade.
    if bar_screen(screen) && !open {
        draw_bookmarks(&mut p, screen.mouse, screen.rect[0]);
    }
}

/// In cheat mode (clicks give items) JEI's wrench beside the search box
/// turns red.
fn cheat_wrench(p: &mut Paint<'_>, row: [f32; 4], mouse: [f32; 2]) {
    if !give_on(p.host) {
        return;
    }
    let button = [row[0] + row[2] - 20.0, row[1], 20.0, 20.0];
    // Under the pointer the game's tooltip is over the button: leave it.
    if jei::inside(mouse, button) {
        return;
    }
    p.whole(
        "icons/config_button_cheat",
        [button[0] + 2.0, button[1] + 2.0, 16.0, 16.0],
    );
}

/// The item list's left edge and the search box's row (with its settings
/// button), as Horizon lays out JEI's classic panel beside `window` (the
/// screen's window, `[x, y, w, h]`) with the recipe screen open; None for a
/// screen with no item list.
fn panel_parts(
    gui: [f32; 2],
    screen: &ScreenView<'_>,
    centred_search: bool,
    window_width: f32,
) -> Option<(f32, [f32; 4])> {
    // Horizon shows a mod's list beside the containers, not the creative screen.
    if screen.id != "inventory" {
        return None;
    }
    let [x, y, w, h] = screen.rect;
    let (x, w) = if window_width > w {
        (((gui[0] - window_width) / 2.0).floor(), window_width)
    } else {
        (x, w)
    };
    let left = x + w + 2.0;
    let centred = centred_search && y + h + 20.0 < gui[1];
    let row = if centred {
        [x, gui[1] - 22.0, w, 20.0]
    } else {
        [left, gui[1] - 22.0, gui[0] - 2.0 - left, 20.0]
    };
    Some((left, row))
}

/// The screens the bookmark list shows on: the containers', and any screen
/// the recipe screen is open over.
fn bar_screen(screen: &ScreenView<'_>) -> bool {
    jei::overlay_screen(screen.id) || with(|s| s.overlay.is_some())
}

/// The bookmark list left of the window (its left edge `window_left`): the
/// page row, the bookmarks, the button that shows or hides them, and the
/// tooltip of the one under the pointer.
fn draw_bookmarks(p: &mut Paint<'_>, mouse: [f32; 2], window_left: f32) {
    let (config, bookmarks, drag, page) =
        with(|s| (s.config, s.bookmarks.clone(), s.drag, s.bookmark_page));
    let area = jei::bookmark_area(p.size(), window_left);
    let lang = lang(p.host);
    let over_button = jei::inside(mouse, area.button);
    let name = if config.bookmarks {
        "bookmark_button_enabled"
    } else {
        "bookmark_button_disabled"
    };
    p.button_box(area.button, name, true, over_button);
    let mut tooltip: Vec<String> = Vec::new();
    if over_button {
        tooltip.push(format!("§f{}", lang.bookmarks_button(config.bookmarks)));
    }
    let shown = config.bookmarks && !bookmarks.is_empty() && area.per_page() > 0;
    if shown {
        let pages = area.pages(bookmarks.len());
        let page = page.min(pages - 1);
        let [back, forward] = area.nav_buttons();
        let many = pages > 1;
        p.button_box(back, "arrow_previous", many, jei::inside(mouse, back));
        p.button_box(forward, "arrow_next", many, jei::inside(mouse, forward));
        let text = lang.page_of(page, pages);
        let [nx, ny, nw, _] = area.nav;
        p.centred(&text, nx + nw / 2.0, ny + 6.0, nw - 40.0, WHITE, true);
        let hovered = area.hit(mouse, page, bookmarks.len());
        let first = page * area.per_page();
        for (n, stack) in bookmarks
            .slots()
            .iter()
            .enumerate()
            .skip(first)
            .take(area.per_page())
        {
            let rect = area.slot(n - first);
            if let Some(item) = item_stack(stack) {
                p.d.item_stack(&item, [rect[0] + 1.0, rect[1] + 1.0], 16.0);
            }
            if hovered == Some(n) || drag.is_some_and(|drag| drag.from == n) {
                p.fill(rect, LIST_LIT);
            }
        }
        if let Some(n) = hovered {
            let slot = &bookmarks.slots()[n];
            let name = with(|s| s.viewer().map(|v| v.item_name(&slot.item)))
                .unwrap_or_else(|| slot.item.clone());
            tooltip = jei::bookmark_tooltip(&name, lang)
                .into_iter()
                .map(|(text, [r, g, b, _])| {
                    let code = match (r, g, b) {
                        (255, 255, 255) => 'f',
                        (255, 255, 85) => 'e',
                        (85, 85, 85) => '8',
                        _ => '7',
                    };
                    format!("§{code}{text}")
                })
                .collect();
        }
    }
    p.tooltip(&tooltip, mouse);
}

/// The bookmark list's parts on this screen: left of its window, or of the
/// recipe screen when that is open over it.
fn bookmark_area(screen: &ScreenView<'_>) -> jei::LeftArea {
    let (gui, overlay_left) = with(|s| {
        let gui = s.gui;
        (gui, s.overlay.as_ref().map(|o| overlay_geom(o, gui).left()))
    });
    let left = overlay_left.map_or(screen.rect[0], |l| l.min(screen.rect[0]));
    jei::bookmark_area(gui, left)
}

// ---------------------------------------------------------------- input

/// A press on the bookmark list: its button shows or hides it, its page
/// buttons turn its pages; on a bookmark the left button takes it (opening
/// its recipes when the release comes without a move), the right one opens
/// its uses, Shift and the middle button take the bookmark away. Else the
/// recipe screen's. True when JEI took it.
pub fn screen_mouse_pressed(
    host: &dyn ClientHost,
    screen: &ScreenView<'_>,
    button: i32,
    modifiers: u32,
) -> bool {
    if !host.on(jei::ID) || !bar_screen(screen) {
        return false;
    }
    load(host);
    let (config, bookmarks, page) = with(|s| (s.config, s.bookmarks.clone(), s.bookmark_page));
    let area = bookmark_area(screen);
    if jei::inside(screen.mouse, area.button) {
        if button == 0 {
            toggle_bar(host);
        }
        return true;
    }
    let shown = config.bookmarks && !bookmarks.is_empty() && area.per_page() > 0;
    let pages = area.pages(bookmarks.len());
    let [back, forward] = area.nav_buttons();
    if shown && (jei::inside(screen.mouse, back) || jei::inside(screen.mouse, forward)) {
        if button == 0 && pages > 1 {
            let page = page.min(pages - 1);
            let to = if jei::inside(screen.mouse, forward) {
                (page + 1) % pages
            } else {
                (page + pages - 1) % pages
            };
            with(|s| s.bookmark_page = to);
        }
        return true;
    }
    let page = page.min(pages - 1);
    let hit = shown
        .then(|| area.hit(screen.mouse, page, bookmarks.len()))
        .flatten();
    let Some(index) = hit else {
        return overlay_pressed(screen, button, modifiers);
    };
    let stack = bookmarks.slots()[index].clone();
    match button {
        // Left: a click opens the recipes, a drag reorders.
        0 if modifiers & SHIFT == 0 => with(|s| {
            s.drag = Some(Drag {
                from: index,
                moved: false,
            })
        }),
        0 | 2 => remove_bookmark(host, index),
        // Right: the uses.
        1 => pending(Event::Open { stack, uses: true }),
        _ => {}
    }
    true
}

/// A release over the list: a left click that never left its slot opens the
/// stack's recipes.
pub fn screen_mouse_released(host: &dyn ClientHost, screen: &ScreenView<'_>, button: i32) -> bool {
    if !host.on(jei::ID) {
        return false;
    }
    if with(|s| std::mem::take(&mut s.overlay_pressed)) {
        return true;
    }
    if !bar_screen(screen) || button != 0 {
        return false;
    }
    let Some(drag) = with(|s| s.drag.take()) else {
        return false;
    };
    if !drag.moved
        && let Some(stack) = with(|s| s.bookmarks.get(drag.from).cloned())
    {
        pending(Event::Open { stack, uses: false });
    }
    true
}

/// Dragging a bookmark over another slot moves it there.
pub fn screen_mouse_dragged(host: &dyn ClientHost, screen: &ScreenView<'_>, button: i32) -> bool {
    if !host.on(jei::ID) {
        return false;
    }
    if with(|s| s.overlay_pressed) {
        return true;
    }
    if !bar_screen(screen) || button != 0 {
        return false;
    }
    let Some(mut drag) = with(|s| s.drag) else {
        return false;
    };
    let (count, page) = with(|s| (s.bookmarks.len(), s.bookmark_page));
    let area = bookmark_area(screen);
    let page = page.min(area.pages(count) - 1);
    let Some(to) = area.hit(screen.mouse, page, count) else {
        return false;
    };
    if to == drag.from {
        return true;
    }
    let moved = with(|s| {
        let moved = s.bookmarks.move_slot(drag.from, to);
        moved.then(|| s.bookmarks.clone())
    });
    if let Some(bookmarks) = moved {
        save_bookmarks(host, &bookmarks);
        drag.from = to;
        drag.moved = true;
        with(|s| s.drag = Some(drag));
    }
    true
}

/// The wheel over the recipe screen turns its pages.
pub fn screen_mouse_scrolled(
    host: &dyn ClientHost,
    screen: &ScreenView<'_>,
    scroll: [f64; 2],
) -> bool {
    if !host.on(jei::ID) {
        return false;
    }
    with(|s| {
        let gui = s.gui;
        let Some(o) = s.overlay.as_mut() else {
            return false;
        };
        if overlay_hit(o, gui, screen.mouse).is_none() || scroll[1] == 0.0 {
            return false;
        }
        let pages = overlay_pages(o, &overlay_geom(o, gui)).len().max(1);
        let page = o.page.min(pages - 1);
        o.page = if scroll[1] < 0.0 {
            (page + 1).min(pages - 1)
        } else {
            page.saturating_sub(1)
        };
        true
    })
}

/// Keys on the recipe screen: Escape and Backspace go back a page (closing
/// it at the first), and the viewer's keys (recipes, uses, bookmark) act on
/// the stack under the pointer.
pub fn screen_key_pressed(host: &dyn ClientHost, screen: &ScreenView<'_>, pressed: &str) -> bool {
    if !host.on(jei::ID) {
        return false;
    }
    let hit = with(|s| {
        let gui = s.gui;
        s.overlay
            .as_ref()
            .map(|o| overlay_hit(o, gui, screen.mouse))
    });
    let Some(hit) = hit else {
        return false;
    };
    if pressed == "key.keyboard.escape" || (pressed == "key.keyboard.backspace" && hit.is_some()) {
        pending(Event::Back);
        return true;
    }
    let recipe = key(host, "recipe", "key.keyboard.r");
    let uses = key(host, "uses", "key.keyboard.u");
    let mark = key(host, "bookmark", jei::BOOKMARK_KEY);
    if ![&recipe, &uses, &mark].contains(&&pressed.to_owned()) || hit.is_none() {
        return false;
    }
    // Over the window, its keys never reach the slots hidden under it.
    if let Some(Hit::Stack { stack, .. }) = hit {
        if pressed == mark {
            with(|s| s.to_bookmark.push(stack));
        } else {
            pending(Event::Open {
                stack,
                uses: pressed == uses,
            });
        }
    }
    true
}

/// The screen under the recipe screen closed (or JEI's own did): so does
/// the recipe screen, and the viewer forgets its history.
pub fn screen_closing(host: &dyn ClientHost) {
    if !host.on(jei::ID) {
        return;
    }
    let open = with(|s| match s.overlay.as_mut() {
        Some(o) => {
            // JEI's own screen is closing: nothing to close after it.
            o.own_screen = false;
            true
        }
        None => false,
    });
    if open {
        pending(Event::Close);
    }
}

// ---------------------------------------------------------------- the recipe screen

/// The recipe screen's parts now, from the GUI's size.
fn overlay_geom(overlay: &Overlay, gui: [f32; 2]) -> geo::Geom {
    geo::geom(
        gui,
        overlay.tabs.len(),
        overlay.tab,
        overlay.tab_page,
        overlay.catalysts.len(),
    )
}

/// Where the recipes go: under the header, and under the subtitle's line
/// when the category has one.
fn overlay_content(overlay: &Overlay, g: &geo::Geom) -> [f32; 4] {
    let [x, y, w, h] = g.content;
    if overlay.subtitle.is_empty() {
        [x, y, w, h]
    } else {
        [x, y + geo::LINE + 2.0, w, h - geo::LINE - 2.0]
    }
}

fn overlay_pages(overlay: &Overlay, g: &geo::Geom) -> Vec<Vec<geo::Piece>> {
    geo::pages(&overlay.views, overlay_content(overlay, g))
}

/// Closes the recipe screen (and JEI's own screen under it).
fn close_overlay(host: &mut dyn ClientHost) {
    let closed = with(|s| {
        s.overlay_pressed = false;
        s.overlay.take()
    });
    if closed.is_some_and(|o| o.own_screen) {
        host.mod_screen_close(jei::ID);
    }
}

/// The stack a slot of alternatives shows now: one a second, as JEI cycles.
fn cycled(stacks: &[Stack]) -> Option<&Stack> {
    if stacks.is_empty() {
        return None;
    }
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs()) as usize;
    stacks.get(secs % stacks.len())
}

/// How far through a cycle of `ms` milliseconds the clock is, 0 to 1: the
/// furnace's arrow and flame.
fn cycle(ms: u128) -> f32 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    (now % ms) as f32 / ms as f32
}

/// What is under the pointer on the recipe screen.
#[derive(Clone, Debug, PartialEq)]
enum Hit {
    Tab(usize),
    PrevTab,
    NextTab,
    /// The tabs' page buttons, when the tabs need pages.
    PrevTabs,
    NextTabs,
    PrevPage,
    NextPage,
    /// A stack: an input, a result, an output or a catalyst. `output` is
    /// the category output's index in its view, for its tooltip.
    Stack {
        stack: Stack,
        view: Option<usize>,
        output: Option<usize>,
    },
    /// The "+" of a view, and whether it is on.
    Plus(usize, bool),
    /// A shapeless recipe's icon.
    Shapeless,
    /// Somewhere else on the window.
    Inside,
}

/// Every hit place of the page shown, in drawing order.
fn overlay_places(overlay: &Overlay, g: &geo::Geom) -> Vec<(Hit, [f32; 4])> {
    let mut out = Vec::new();
    let pages = overlay_pages(overlay, g);
    if overlay.tabs.len() > 1 {
        out.push((Hit::PrevTab, g.prev_tab));
        out.push((Hit::NextTab, g.next_tab));
    }
    if pages.len() > 1 {
        out.push((Hit::PrevPage, g.prev_page));
        out.push((Hit::NextPage, g.next_page));
    }
    let page = overlay.page.min(pages.len().saturating_sub(1));
    if let Some(pieces) = pages.get(page) {
        let content = overlay_content(overlay, g);
        for (n, at) in geo::layout(&overlay.views, pieces, content) {
            let piece = &pieces[n];
            let view = &overlay.views[piece.view];
            let placed = geo::place(view, piece, at, content[2]);
            if let Some(rect) = placed.shapeless {
                out.push((Hit::Shapeless, rect));
            }
            for (part, rect) in placed.slots {
                let (stack, output) = match part {
                    geo::Part::Input(i) => (view.slots.get(i).and_then(|s| cycled(s)), None),
                    geo::Part::Result(i) => (view.results.get(i), Some(i)),
                };
                if let Some(stack) = stack {
                    out.push((
                        Hit::Stack {
                            stack: stack.clone(),
                            view: Some(piece.view),
                            output: output.filter(|_| view.layout == Layout::Chance),
                        },
                        rect,
                    ));
                }
            }
            if let (Some(rect), Some(on)) = (placed.button, view.action) {
                out.push((Hit::Plus(piece.view, on), rect));
            }
        }
    }
    for (stack, rect) in overlay.catalysts.iter().zip(&g.catalysts) {
        out.push((
            Hit::Stack {
                stack: stack.clone(),
                view: None,
                output: None,
            },
            *rect,
        ));
    }
    for (i, rect) in &g.tabs {
        out.push((Hit::Tab(*i), *rect));
    }
    if g.tab_pages > 1 {
        let [back, forward] = g.tab_buttons();
        out.push((Hit::PrevTabs, back));
        out.push((Hit::NextTabs, forward));
    }
    out
}

/// The hit under `point`, if the point is on the recipe screen at all.
fn overlay_hit(overlay: &Overlay, gui: [f32; 2], point: [f32; 2]) -> Option<Hit> {
    let g = overlay_geom(overlay, gui);
    let places = overlay_places(overlay, &g);
    if let Some((hit, _)) = places.iter().rev().find(|(_, r)| jei::inside(point, *r)) {
        return Some(hit.clone());
    }
    let on = jei::inside(point, g.panel)
        || (!g.catalysts.is_empty() && jei::inside(point, g.catalyst_panel))
        || (g.tab_pages > 1 && jei::inside(point, g.tab_nav));
    on.then_some(Hit::Inside)
}

/// The server's answer for the "+" of recipe `book`, when it has one.
fn server_check(book: Option<i32>) -> Option<RecipeTransferCheck> {
    let book = book?;
    with(|s| {
        (s.answer.as_ref())
            .filter(|(b, _)| *b == book)
            .map(|(_, a)| a.clone())
    })
}

/// The inputs of `view` the "+" lacks, by the server's check: it counts what
/// the client can't see (a crafting station's chests). None until it answers.
fn lacking(view: &View, check: Option<&RecipeTransferCheck>) -> Vec<usize> {
    if view.action != Some(true) {
        return Vec::new();
    }
    match check {
        Some(check) if check.result == RecipeTransferResult::Missing => {
            let filled = |i: &usize| view.slots.get(*i).is_some_and(|s| !s.is_empty());
            let cells: Vec<usize> = (check.missing_cells.iter())
                .map(|&i| i as usize)
                .filter(filled)
                .collect();
            if cells.is_empty() {
                // Missing, but no cell this view shows: all of them.
                (0..view.slots.len()).filter(filled).collect()
            } else {
                cells
            }
        }
        _ => Vec::new(),
    }
}

/// The recipe book id of the shown recipe `view`.
fn view_book(view: usize) -> Option<i32> {
    with(|s| s.viewer().and_then(|v| v.shown_book(view)))
}

/// The vanilla background a recipe is drawn on, from the player's own game
/// files: the crafting table's grid and arrow, the furnace's slots.
fn draw_background(p: &mut Paint<'_>, overlay: &Overlay, view: &View, placed: &geo::Placed) {
    let [x, y, w, _] = placed.rect;
    match placed.background {
        geo::Background::Crafting => {
            let src = [29.0, 16.0, geo::CRAFTING[0], geo::CRAFTING[1]];
            let dst = [x, y, geo::CRAFTING[0], geo::CRAFTING[1]];
            if !p.sprite("gui/container/crafting_table", src, dst) {
                for (_, rect) in &placed.slots {
                    p.slot_box(*rect);
                }
            }
        }
        geo::Background::Cooking => {
            let station = overlay
                .tabs
                .get(overlay.tab)
                .map_or("", |s| s.item.as_str());
            let texture = match station {
                "minecraft:smoker" => "gui/container/smoker",
                "minecraft:blast_furnace" => "gui/container/blast_furnace",
                _ => "gui/container/furnace",
            };
            let src = [55.0, 16.0, geo::COOKING[0], geo::COOKING[1]];
            let dst = [x, y, geo::COOKING[0], geo::COOKING[1]];
            if p.sprite(texture, src, dst) {
                // The flame burning down over 15 s and the arrow filling
                // over 10, as JEI animates them.
                let burnt = (cycle(15_000) * 14.0).floor();
                p.sprite(
                    "gui/sprites/container/furnace/lit_progress",
                    [0.0, burnt, 14.0, 14.0 - burnt],
                    [x + 1.0, y + 20.0 + burnt, 14.0, 14.0 - burnt],
                );
                let done = (cycle(10_000) * 24.0).floor();
                if done > 0.0 {
                    p.sprite(
                        "gui/sprites/container/furnace/burn_progress",
                        [0.0, 0.0, done, 16.0],
                        [x + 24.0, y + 18.0, done, 16.0],
                    );
                }
            } else {
                for (_, rect) in &placed.slots {
                    p.slot_box(*rect);
                }
            }
            // The time and experience at the top right, as JEI's.
            if !view.note.is_empty() {
                let note = view.note.replace('\n', " ");
                let tw = p.width(&note);
                p.text(&note, [x + w - tw, y], NOTE, false);
            }
        }
        geo::Background::Slots => {
            for (_, rect) in &placed.slots {
                p.slot_box(*rect);
            }
            if let Some([ax, ay, aw, ah]) = placed.arrow {
                let src = [90.0, 35.0, 22.0, 15.0];
                if !p.sprite("gui/container/crafting_table", src, [ax, ay, aw, ah]) {
                    let mid = ay + ah / 2.0;
                    p.fill([ax, mid - 2.0, aw - 7.0, 4.0], 0xFF_8B_8B_8B);
                    p.d.quad(
                        [
                            [ax + aw - 8.0, ay],
                            [ax + aw, mid],
                            [ax + aw, mid],
                            [ax + aw - 8.0, ay + ah],
                        ],
                        0xFF_8B_8B_8B,
                    );
                }
            }
        }
    }
}

/// The recipe screen, as JEI 1.12's `RecipesGui` draws it, over the screen
/// it opened on (`screen`: where it is and the pointer).
fn draw_overlay(p: &mut Paint<'_>, screen: &ScreenAt, overlay: &Overlay) {
    let gui = p.size();
    let mouse = screen.mouse;
    let g = overlay_geom(overlay, gui);
    let lang = lang(p.host);
    let hit = overlay_hit(overlay, gui, mouse);
    p.shade_under(screen.panel);
    // The bookmarks stay, as JEI's left list stays over its recipe screen.
    draw_bookmarks(p, mouse, g.left().min(screen.rect[0]));

    p.frame_box(g.panel);
    p.fill(g.title_bar, BAR);
    p.fill(g.page_bar, BAR);
    let room = g.title_bar[2] - 2.0 * 14.0;
    let name = overlay.names.get(overlay.tab).cloned().unwrap_or_default();
    let [bx, by, bw, _] = g.title_bar;
    p.centred(&name, bx + bw / 2.0, by + 2.0, room, WHITE, true);
    let pages = overlay_pages(overlay, &g);
    let page = overlay.page.min(pages.len().saturating_sub(1));
    let [px, py, pw, _] = g.page_bar;
    let page_text = lang.page_of(page, pages.len());
    p.centred(&page_text, px + pw / 2.0, py + 2.0, pw, WHITE, true);
    let many_tabs = overlay.tabs.len() > 1;
    let many_pages = pages.len() > 1;
    for (rect, name, on, which) in [
        (g.prev_tab, "arrow_previous", many_tabs, Hit::PrevTab),
        (g.next_tab, "arrow_next", many_tabs, Hit::NextTab),
        (g.prev_page, "arrow_previous", many_pages, Hit::PrevPage),
        (g.next_page, "arrow_next", many_pages, Hit::NextPage),
    ] {
        p.button_box(rect, name, on, hit.as_ref() == Some(&which));
    }
    if !overlay.subtitle.is_empty() {
        let [cx, cy, cw, _] = g.content;
        let line = overlay.subtitle.replace('\n', " ");
        p.centred(&line, cx + cw / 2.0, cy + 2.0, cw - 12.0, TEXT, false);
    }

    let content = overlay_content(overlay, &g);
    let mut lacks_hovered = false;
    let hovered_book = match &hit {
        Some(Hit::Plus(v, true)) => view_book(*v),
        _ => None,
    };
    with(|s| s.hovered_book = hovered_book);
    let check = server_check(hovered_book);
    if let Some(pieces) = pages.get(page) {
        for (n, at) in geo::layout(&overlay.views, pieces, content) {
            let piece = &pieces[n];
            let view = &overlay.views[piece.view];
            let placed = geo::place(view, piece, at, content[2]);
            let [x, y, w, h] = placed.rect;
            let f = geo::FRAME;
            let framed = [x - f, y - f, w + 2.0 * f, h + 2.0 * f];
            if !p.nine("single_recipe_background", framed, [16.0; 4]) {
                p.fill(framed, 0x20_00_00_00);
            }
            draw_background(p, overlay, view, &placed);
            let plus_hovered = matches!(hit, Some(Hit::Plus(v, _)) if v == piece.view);
            let lacks = lacking(view, check.as_ref().filter(|_| plus_hovered));
            lacks_hovered |= plus_hovered && !lacks.is_empty();
            if let Some(rect) = placed.shapeless
                && !p.whole("icons/shapeless_icon", rect)
            {
                p.text("*", [rect[0] + 2.0, rect[1]], NOTE, false);
            }
            for (part, rect) in &placed.slots {
                let stack = match part {
                    geo::Part::Input(i) => view.slots.get(*i).and_then(|s| cycled(s)),
                    geo::Part::Result(i) => view.results.get(*i),
                };
                if let Some(stack) = stack {
                    p.stack(stack, *rect);
                }
                // JEI marks what the "+" lacks while the pointer is on it.
                if plus_hovered && matches!(part, geo::Part::Input(i) if lacks.contains(i)) {
                    p.fill([rect[0] + 1.0, rect[1] + 1.0, 16.0, 16.0], MISSING_SLOT);
                }
                if jei::inside(mouse, *rect) && stack.is_some() {
                    p.fill([rect[0] + 1.0, rect[1] + 1.0, 16.0, 16.0], LIT);
                }
                if let (geo::Part::Result(i), Layout::Chance) = (part, view.layout)
                    && let Some(c) = view.chances.get(*i)
                {
                    let mark = if c.notes.is_empty() { "" } else { "*" };
                    let tx = rect[0] + geo::SLOT + 2.0;
                    let chance = format!("{}{mark}", jei::percent(c.chance));
                    p.text(&chance, [tx, rect[1] + 1.0], TEXT, false);
                    let range = jei::count_range(c.min, c.max);
                    if !range.is_empty() {
                        p.text(&range, [tx, rect[1] + 10.0], NOTE, false);
                    }
                }
            }
            if let (Some(rect), Some(on)) = (placed.button, view.action) {
                let hovered = hit == Some(Hit::Plus(piece.view, on));
                let on = on && lacks.is_empty();
                p.button_box(rect, "recipe_transfer", on, hovered);
                if !lacks.is_empty() {
                    p.fill(rect, MISSING_BUTTON);
                }
            }
            let notes = !view.note.is_empty()
                && !view.shapeless
                && view.layout != Layout::Cooking
                && piece.outputs.start == 0;
            if notes {
                for (k, text) in view.note.split('\n').enumerate() {
                    let at = [placed.notes[0], placed.notes[1] + k as f32 * geo::LINE];
                    p.text(text, at, NOTE, false);
                }
            }
        }
    }

    // The catalysts' column, fused to the window's left edge.
    if !g.catalysts.is_empty() {
        let [cx, cy, cw, ch] = g.catalyst_panel;
        if !p.nine("catalyst_tab", g.catalyst_panel, [8.0, 8.0, 9.0, 8.0]) {
            p.frame_box(g.catalyst_panel);
        }
        p.slot_box([cx + 5.0, cy + 5.0, cw - 10.0, ch - 10.0]);
        for (stack, rect) in overlay.catalysts.iter().zip(&g.catalysts) {
            p.stack(stack, [rect[0] - 1.0, rect[1] - 1.0, geo::SLOT, geo::SLOT]);
            if jei::inside(mouse, *rect) {
                p.fill(*rect, LIT);
            }
        }
    }

    // The tabs over the window's top edge, the chosen one joined to it.
    for (i, rect) in &g.tabs {
        let selected = *i == overlay.tab;
        let sprite = if selected {
            "tab_selected"
        } else {
            "tab_unselected"
        };
        if !p.whole(sprite, *rect) {
            let r = if selected {
                *rect
            } else {
                [rect[0], rect[1] + 2.0, rect[2], rect[3] - 2.0]
            };
            p.frame_box(r);
            if !selected {
                p.fill(r, 0x30_00_00_00);
            }
        }
        if let Some(icon) = overlay.tabs.get(*i) {
            p.stack(icon, [rect[0] + 3.0, rect[1] + 3.0, 18.0, 18.0]);
        }
    }
    if g.tab_pages > 1 {
        let [back, forward] = g.tab_buttons();
        p.button_box(back, "arrow_previous", true, hit == Some(Hit::PrevTabs));
        p.button_box(forward, "arrow_next", true, hit == Some(Hit::NextTabs));
        let text = lang.page_of(g.tab_page, g.tab_pages);
        let [nx, ny, nw, nh] = g.tab_nav;
        p.centred(
            &text,
            nx + nw / 2.0,
            ny + ((nh - 9.0) / 2.0).round(),
            nw,
            WHITE,
            true,
        );
    }

    // The tooltip of what the pointer is on.
    let lines: Vec<String> = match &hit {
        Some(Hit::Stack {
            stack,
            view,
            output,
        }) => overlay_stack_lines(lang, overlay, stack, *view, *output),
        Some(Hit::Plus(..)) if lacks_hovered => vec![format!("§c{}", lang.missing_hint())],
        Some(Hit::Plus(_, on)) => vec![format!("§f{}", lang.transfer_hint(*on))],
        Some(Hit::Shapeless) => vec![format!("§f{}", lang.shapeless_hint())],
        Some(Hit::Tab(i)) => (overlay.names.get(*i))
            .map(|n| vec![format!("§f{n}")])
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    p.tooltip(&lines, mouse);
}

/// A stack's tooltip on the recipe screen: its name, a category output's
/// chance, count and notes, and JEI's hints.
fn overlay_stack_lines(
    lang: jei::Lang,
    overlay: &Overlay,
    stack: &Stack,
    view: Option<usize>,
    output: Option<usize>,
) -> Vec<String> {
    let (name, counts, marked) = with(|s| {
        let (name, counts) = s
            .viewer()
            .map_or((stack.item.clone(), Counts::default()), |v| {
                (
                    v.item_name(&stack.item),
                    v.counts(&stack.item, &stack.components),
                )
            });
        (name, counts, s.bookmarks.position(stack).is_some())
    });
    let mut lines = vec![format!("§f{name}")];
    if let (Some(v), Some(o)) = (view, output)
        && let Some(c) = overlay.views.get(v).and_then(|v| v.chances.get(o))
    {
        let range = jei::count_range(c.min, c.max);
        let range = if range.is_empty() {
            String::new()
        } else {
            format!(" {range}")
        };
        lines.push(format!("§e{}{range}", jei::percent(c.chance)));
        for note in &c.notes {
            lines.push(format!("§7{note}"));
        }
    }
    for (text, _) in jei::tooltip_lines(counts, marked, lang) {
        lines.push(format!("§8{text}"));
    }
    lines
}

/// A press on the recipe screen: it takes every press on its window, and
/// the presses on the screen under it (it is modal), but not the item list
/// beside the screen or the bookmarks.
fn overlay_pressed(screen: &ScreenView<'_>, button: i32, modifiers: u32) -> bool {
    // GLFW's shift bit: "+" with Shift moves as many sets as the items allow.
    let shift = modifiers & SHIFT != 0;
    let gui = with(|s| s.gui);
    let hit = with(|s| {
        s.overlay
            .as_ref()
            .map(|o| overlay_hit(o, gui, screen.mouse))
    });
    let Some(hit) = hit else {
        return false;
    };
    let Some(hit) = hit else {
        // Outside the window: the item list and its search box still take
        // clicks; the screen under the window doesn't.
        let panel = with(|s| s.last_at.and_then(|at| at.panel));
        let on_list = panel
            .is_some_and(|(left, row)| screen.mouse[0] >= left || jei::inside(screen.mouse, row));
        if !on_list {
            with(|s| s.overlay_pressed = true);
        }
        return !on_list;
    };
    // A "+" that lacks items does nothing, as JEI's. Before the server has
    // answered (a click in the frame the pointer arrived) it tries anyway and
    // keeps the screen open: the server refuses a set it can't make.
    let (plus_ready, plus_known) = match hit {
        Hit::Plus(view, true) => {
            let check = server_check(view_book(view));
            let known = check.is_some();
            let ready = with(|s| {
                let view = s.overlay.as_ref().and_then(|o| o.views.get(view));
                view.is_none_or(|v| lacking(v, check.as_ref()).is_empty())
            });
            (ready, known)
        }
        _ => (false, false),
    };
    with(|s| {
        s.overlay_pressed = true;
        let tabs = s.overlay.as_ref().map_or(0, |o| o.tabs.len());
        let tab = s.overlay.as_ref().map_or(0, |o| o.tab);
        let pages = (s.overlay.as_ref())
            .map_or(1, |o| overlay_pages(o, &overlay_geom(o, gui)).len().max(1));
        let tab_pages = s.overlay.as_ref().map(|o| {
            let g = overlay_geom(o, gui);
            (g.tab_page, g.tab_pages)
        });
        match hit {
            Hit::Tab(i) => s.pending.push(Event::Tab(i)),
            Hit::PrevTabs | Hit::NextTabs if button == 0 => {
                if let (Some(o), Some((page, count))) = (s.overlay.as_mut(), tab_pages) {
                    let count = count.max(1);
                    o.tab_page = Some(if hit == Hit::NextTabs {
                        (page + 1) % count
                    } else {
                        (page + count - 1) % count
                    });
                }
            }
            Hit::PrevTab if tabs > 0 => s.pending.push(Event::Tab((tab + tabs - 1) % tabs)),
            Hit::NextTab if tabs > 0 => s.pending.push(Event::Tab((tab + 1) % tabs)),
            Hit::PrevPage | Hit::NextPage => {
                if let Some(o) = s.overlay.as_mut() {
                    let page = o.page.min(pages - 1);
                    o.page = if hit == Hit::NextPage {
                        (page + 1) % pages
                    } else {
                        (page + pages - 1) % pages
                    };
                }
            }
            Hit::Stack { stack, .. } if button == 0 || button == 1 => {
                s.pending.push(Event::Open {
                    stack,
                    uses: button == 1,
                });
            }
            Hit::Plus(view, true) if button == 0 && plus_ready => {
                // JEI moves the items and goes back to the screen they went to.
                s.pending.push(Event::Action(view, shift));
                if plus_known {
                    s.pending.push(Event::Close);
                }
            }
            _ => {}
        }
    });
    true
}
