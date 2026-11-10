// The edges between Horizon and the viewer (crate::game::jei): the game's
// item and recipe tables in, the panel, give and place out, and the overlay
// (tooltips, the bookmark bar and its clicks) wired to the viewer.
use crate::game::Game;
use crate::game::jei::{
    self, Bookmarks, Config, Context, Counts, Event, Ingredient, Item, Kind, Layout, Out, Recipe,
    Session, Sort, Stack, Toggle, View, Viewer, screen as geo,
};
use crate::game::native_hooks::{
    NativePanelEvent, NativeRecipeKind, NativeRecipeLayout, NativeRecipeStack, NativeRecipeView,
    NativeRecipes, NativeScreenView, NativeStack, NativeTransferCheck,
};
use horizon_client::items::ItemStack;
use std::cell::RefCell;
use std::path::PathBuf;

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
    /// Where this frame's screen is, for the recipe screen
    /// `gui_render_post` draws.
    overlay_at: Option<ScreenAt>,
    /// The same, kept for the input and tooltip hooks of the next frame.
    last_at: Option<ScreenAt>,
    /// The bookmark list's page.
    bookmark_page: usize,
    /// The item list's look last asked of Horizon (`native_panel_style`).
    panel_style: Option<horizon_client::mod_api::NativePanelStyle>,
    /// The server's recipes arrived (`native_recipes_received`): build the
    /// viewer again from them.
    rebuild: bool,
    /// JEI's recipe screen, while it is open.
    overlay: Option<Overlay>,
    /// The last press went to the recipe screen: its release and drags too.
    overlay_pressed: bool,
    /// The GUI's size at the last frame drawn, for the input hooks' hit tests.
    gui: [f32; 2],
    /// Stacks the overlay's keys bookmark at the next frame.
    to_bookmark: Vec<Stack>,
    /// The mods' categories' version the viewer was given
    /// (`native_recipe_categories_version`).
    categories: Option<u64>,
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
    /// The server's dry run of the "+" under the pointer (`native_transfer_check`):
    /// the recipe the last frame drew it on, and the latest answer for it.
    static CHECK: RefCell<Check> = RefCell::new(Check::default());
}

#[derive(Default)]
struct Check {
    hovered: Option<i32>,
    answer: Option<(i32, NativeTransferCheck)>,
}

/// The folder the mod keeps its files in, `config/jei/`, inside the game's.
fn dir() -> Option<PathBuf> {
    let mut path = Game::native_config_dir().ok()?;
    path.push(jei::ID);
    Some(path)
}

fn config_path() -> Option<PathBuf> {
    Some(dir()?.join("jei.cfg"))
}

fn bookmarks_path() -> Option<PathBuf> {
    Some(dir()?.join("bookmarks.txt"))
}

fn read(path: Option<PathBuf>) -> Option<String> {
    std::fs::read_to_string(path?).ok()
}

fn write(path: Option<PathBuf>, text: &str) {
    let Some(path) = path else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, text);
}

/// Reads the config and the bookmarks once: `config/jei/jei.cfg` and
/// `config/jei/bookmarks.txt`.
fn load() {
    if STATE.with(|s| s.borrow().loaded) {
        return;
    }
    let config = read(config_path()).map_or_else(Config::default, |text| Config::parse(&text));
    let bookmarks =
        read(bookmarks_path()).map_or_else(Bookmarks::default, |text| jei::bookmarks_parse(&text));
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.config = config;
        state.bookmarks = bookmarks;
        state.loaded = true;
    });
}

/// The config now, read from disk the first time.
pub(super) fn config() -> Config {
    load();
    STATE.with(|s| s.borrow().config)
}

fn store_config(config: Config) {
    STATE.with(|s| s.borrow_mut().config = config);
    write(config_path(), &config.to_text());
}

fn save_bookmarks(bookmarks: &Bookmarks) {
    write(bookmarks_path(), &jei::bookmarks_text(bookmarks));
}

/// Where the config and the bookmarks live, for `/jei config`.
pub(super) fn files() -> String {
    match (config_path(), bookmarks_path()) {
        (Some(config), Some(bookmarks)) => {
            format!(
                "JEI files: {} and {}",
                config.display(),
                bookmarks.display()
            )
        }
        _ => "JEI: the game's config folder is unknown".to_owned(),
    }
}

/// The player's language.
fn lang(game: &Game<'_>) -> jei::Lang {
    jei::lang(game.native_setting(jei::ID, "language").as_deref())
}

/// Builds the viewer once per world and shows the item list in the panel.
pub(super) fn sync(game: &mut Game<'_>) {
    let has_player = game.native_player().is_some();
    let enabled = game.native_on(jei::ID);
    let needs_build = STATE.with(|s| {
        let mut state = s.borrow_mut();
        if !std::mem::take(&mut state.screen_seen) {
            state.screen = None;
        }
        let rebuild = has_player && enabled && std::mem::take(&mut state.rebuild);
        if rebuild {
            state.session.clear();
        }
        let needs_build = state.session.update(has_player, enabled);
        if !has_player {
            state.drag = None;
            state.pending.clear();
        }
        needs_build
    });
    if !needs_build {
        refresh_categories(game);
        return;
    }
    let Some(catalog) = game.native_recipes() else {
        return;
    };
    load();
    let sort = STATE.with(|s| s.borrow().config.sort);
    let recipes = jei::dedupe_recipes(recipes(&catalog));
    let found = jei::harvest_variants(&recipes, VARIANTS_PER_ITEM, jei::ITEMS_MAX);
    let (mut viewer, mut out) = Viewer::with_variants(items(&catalog), recipes, found, lang(game));
    if sort != viewer.sort() {
        out = viewer.set_sort(sort);
    }
    // A mod menu type with a crafting grid crafts as the crafting table: its
    // block (the item of the same id) is one more crafting catalyst.
    viewer.set_crafters(
        horizon_client::containers::mod_menu_types()
            .iter()
            .filter(|t| t.crafting.is_some())
            .map(|t| t.id.to_owned())
            .collect(),
    );
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.session.set_viewer(viewer);
        state.categories = None;
    });
    refresh_categories(game);
    STATE.with(|s| s.borrow_mut().panel_style = None);
    panel_style(game);
    if let Err(error) = apply(game, out) {
        game.native_chat(&format!("JEI: {error}"));
    }
}

/// JEI 1.12's item list (`NativePanelStyle::Classic`): its search box where
/// the player wants it, and laid out beside the recipe screen while that is
/// open, as JEI's list made room for its `RecipesGui`. Asked again only
/// when it changes.
fn panel_style(game: &mut Game<'_>) {
    let (config, open, last) = STATE.with(|s| {
        let state = s.borrow();
        (state.config, state.overlay.is_some(), state.panel_style)
    });
    let style = horizon_client::mod_api::NativePanelStyle::Classic {
        centred_search: config.centred_search,
        window_width: if open { geo::WIDTH as u16 } else { 0 },
    };
    if last != Some(style) {
        game.native_panel_style(jei::ID, style);
        STATE.with(|s| s.borrow_mut().panel_style = Some(style));
    }
}

/// Runs what the overlay's hooks left for it (a bookmark click opens the
/// panel), once a frame, where the game can still be changed.
pub(super) fn flush(game: &mut Game<'_>) {
    if !game.native_on(jei::ID) {
        return;
    }
    if STATE.with(|s| s.borrow().session.viewer().is_some()) {
        panel_style(game);
    }
    let (pending, marks) = STATE.with(|s| {
        let mut state = s.borrow_mut();
        (
            std::mem::take(&mut state.pending),
            std::mem::take(&mut state.to_bookmark),
        )
    });
    for stack in marks {
        bookmark(game, stack);
    }
    for event in pending {
        dispatch(game, event);
    }
    // The "+" under the pointer: the server tries it (the chests beside a
    // crafting station counted) and says what lacks; asked again as things change.
    let hovered = CHECK.with(|c| c.borrow_mut().hovered.take());
    let answer = hovered.and_then(|book| game.native_transfer_check(book).map(|a| (book, a)));
    CHECK.with(|c| c.borrow_mut().answer = answer);
}

/// The panel's events: search, clicks, keys, tabs, pages and the action button.
pub(super) fn event(game: &mut Game<'_>, event: &NativePanelEvent) {
    if !game.native_on(jei::ID) {
        return;
    }
    let Some(event) = convert_event(event) else {
        return;
    };
    // The bookmark key bookmarks the stack under the pointer, wherever it is:
    // the game says which one it is.
    if let Event::Key { key, stack } = &event {
        let bookmark_key = game.native_mod_key(jei::ID, "bookmark", jei::BOOKMARK_KEY);
        if *key == bookmark_key {
            bookmark(game, stack.clone());
            return;
        }
    }
    dispatch(game, event);
}

/// Bookmarks a stack, or takes its bookmark away. As JEI's, the list and the
/// tooltip's mark say so; only a full list is told in chat.
fn bookmark(game: &mut Game<'_>, stack: Stack) {
    let lang = lang(game);
    let result = STATE.with(|s| {
        let mut state = s.borrow_mut();
        let result = state.bookmarks.toggle(&stack);
        if result != Toggle::Full {
            save_bookmarks(&state.bookmarks);
        }
        result
    });
    if result == Toggle::Full {
        game.native_chat(lang.bookmarks_full());
    }
}

/// Takes a bookmark away by its slot, keeping the file.
fn remove_bookmark(index: usize) {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        if state.bookmarks.remove(index).is_some() {
            save_bookmarks(&state.bookmarks);
        }
        state.drag = None;
    });
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
pub(super) fn gui_render_post(game: &Game<'_>, batch: &mut horizon_client::hud::HudBatch) {
    let at = STATE.with(|s| s.borrow_mut().overlay_at.take());
    let Some(at) = at else {
        return;
    };
    if !game.native_on(jei::ID) {
        return;
    }
    STATE.with(|s| {
        if let Some(overlay) = s.borrow().overlay.as_ref() {
            draw_overlay(game, batch, &at, overlay);
        }
    });
}

pub(super) fn screen_render_post(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &NativeScreenView<'_>,
) {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.screen = Some((screen.id, screen.menu.map(str::to_owned)));
        state.screen_seen = true;
        state.gui = [batch.width, batch.height];
    });
    if !game.native_on(jei::ID) {
        return;
    }
    load();
    let (config, open) = STATE.with(|s| {
        let state = s.borrow();
        (state.config, state.overlay.is_some())
    });
    let gui = [batch.width, batch.height];
    let window_width = if open { geo::WIDTH } else { 0.0 };
    let panel = panel_parts(gui, screen, config.centred_search, window_width);
    // The recipe screen itself draws in `gui_render_post`, over this screen
    // and whatever a mod drew on it after us; it needs where this one is.
    STATE.with(|s| {
        let at = ScreenAt {
            rect: screen.rect,
            mouse: screen.mouse,
            panel,
        };
        let mut state = s.borrow_mut();
        state.overlay_at = Some(at);
        state.last_at = Some(at);
    });
    if let Some((_, row)) = panel {
        cheat_wrench(game, batch, row, screen.mouse);
    }
    // Under the recipe screen, the bookmarks draw with it, over its shade.
    if bar_screen(screen) && !open {
        draw_bookmarks(game, batch, screen.mouse, screen.rect[0]);
    }
}

/// In cheat mode (clicks give items) JEI's wrench beside the search box
/// turns red.
fn cheat_wrench(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    row: [f32; 4],
    mouse: [f32; 2],
) {
    if !jei::give(game.native_setting(jei::ID, "give").as_deref()) {
        return;
    }
    let button = [row[0] + row[2] - 20.0, row[1], 20.0, 20.0];
    // Under the pointer the game's tooltip is over the button: leave it.
    if jei::inside(mouse, button) {
        return;
    }
    whole(
        game,
        batch,
        "icons/config_button_cheat",
        [button[0] + 2.0, button[1] + 2.0, 16.0, 16.0],
    );
}

/// While the recipe screen is open, the screen under it shows no tooltips:
/// only the item list beside it and its search box do.
pub(super) fn tooltip_pre(game: &Game<'_>, at: [f32; 2], cancel: &mut bool) {
    if !game.native_on(jei::ID) {
        return;
    }
    let (open, panel) = STATE.with(|s| {
        let state = s.borrow();
        (
            state.overlay.is_some(),
            state.last_at.and_then(|at| at.panel),
        )
    });
    let on_list = panel.is_some_and(|(left, row)| at[0] >= left || jei::inside(at, row));
    if open && !on_list {
        *cancel = true;
    }
}

/// The bookmark list left of the window (its left edge `window_left`): the
/// page row, the bookmarks, the button that shows or hides them, and the
/// tooltip of the one under the pointer.
fn draw_bookmarks(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    mouse: [f32; 2],
    window_left: f32,
) {
    let (config, bookmarks, drag, page) = STATE.with(|s| {
        let state = s.borrow();
        (
            state.config,
            state.bookmarks.clone(),
            state.drag,
            state.bookmark_page,
        )
    });
    let area = jei::bookmark_area([batch.width, batch.height], window_left);
    let lang = lang(game);
    let over_button = jei::inside(mouse, area.button);
    button_box(
        game,
        batch,
        area.button,
        if config.bookmarks {
            "bookmark_button_enabled"
        } else {
            "bookmark_button_disabled"
        },
        true,
        over_button,
    );
    let mut tooltip: Vec<(String, [u8; 4])> = Vec::new();
    if over_button {
        tooltip.push((lang.bookmarks_button(config.bookmarks), jei::NAME_COLOR));
    }
    let shown = config.bookmarks && !bookmarks.is_empty() && area.per_page() > 0;
    if shown {
        let pages = area.pages(bookmarks.len());
        let page = page.min(pages - 1);
        let [back, forward] = area.nav_buttons();
        let many = pages > 1;
        button_box(
            game,
            batch,
            back,
            "arrow_previous",
            many,
            jei::inside(mouse, back),
        );
        button_box(
            game,
            batch,
            forward,
            "arrow_next",
            many,
            jei::inside(mouse, forward),
        );
        let text = lang.page_of(page, pages);
        let [nx, ny, nw, _] = area.nav;
        let mid = nx + nw / 2.0;
        centred(game, batch, &text, mid, ny + 6.0, nw - 40.0, WHITE, true);
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
                game.native_item_stack(batch, &item, [rect[0] + 1.0, rect[1] + 1.0], 16.0);
            }
            if hovered == Some(n) || drag.is_some_and(|drag| drag.from == n) {
                game.native_fill(batch, rect, LIST_LIT);
            }
        }
        if let Some(n) = hovered {
            let slot = &bookmarks.slots()[n];
            let name = STATE
                .with(|s| s.borrow().viewer().map(|v| v.item_name(&slot.item)))
                .unwrap_or_else(|| slot.item.clone());
            tooltip = jei::bookmark_tooltip(&name, lang);
        }
    }
    draw_tooltip(game, batch, mouse, &tooltip);
}

/// The screens the bookmark list shows on: the containers', and any screen
/// the recipe screen is open over.
fn bar_screen(screen: &NativeScreenView<'_>) -> bool {
    jei::overlay_screen(screen.id) || STATE.with(|s| s.borrow().overlay.is_some())
}

/// A vanilla tooltip at the pointer for lines in their colours.
fn draw_tooltip(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    mouse: [f32; 2],
    lines: &[(String, [u8; 4])],
) {
    if lines.is_empty() {
        return;
    }
    let texts: Vec<horizon_client::text::Text> = lines
        .iter()
        .map(|(text, [r, g, b, _])| {
            let code = match (r, g, b) {
                (255, 255, 255) => 'f',
                (255, 255, 85) => 'e',
                (85, 85, 85) => '8',
                _ => '7',
            };
            horizon_client::text::Text::literal(format!("§{code}{text}").as_str()).resolve_legacy()
        })
        .collect();
    game.native_gui_tooltip(batch, &texts, mouse);
}

/// The bookmark list's parts on this screen: left of its window, or of the
/// recipe screen when that is open over it.
fn bookmark_area(screen: &NativeScreenView<'_>) -> jei::LeftArea {
    let (gui, overlay_left) = STATE.with(|s| {
        let state = s.borrow();
        let gui = state.gui;
        (
            gui,
            state.overlay.as_ref().map(|o| overlay_geom(o, gui).left()),
        )
    });
    let left = overlay_left.map_or(screen.rect[0], |l| l.min(screen.rect[0]));
    jei::bookmark_area(gui, left)
}

/// A press on the bookmark list: its button shows or hides it, its page
/// buttons turn its pages; on a bookmark the left button takes it (opening
/// its recipes when the release comes without a move), the right one opens
/// its uses, Shift and the middle button take the bookmark away.
pub(super) fn screen_mouse_pressed(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    button: i32,
    modifiers: u32,
    canceled: &mut bool,
) {
    if !game.native_on(jei::ID) || !bar_screen(screen) {
        return;
    }
    load();
    let (config, bookmarks, page) = STATE.with(|s| {
        let state = s.borrow();
        (state.config, state.bookmarks.clone(), state.bookmark_page)
    });
    let area = bookmark_area(screen);
    if jei::inside(screen.mouse, area.button) {
        *canceled = true;
        if button == 0 {
            toggle_bar();
        }
        return;
    }
    let shown = config.bookmarks && !bookmarks.is_empty() && area.per_page() > 0;
    let pages = area.pages(bookmarks.len());
    let [back, forward] = area.nav_buttons();
    if shown && (jei::inside(screen.mouse, back) || jei::inside(screen.mouse, forward)) {
        *canceled = true;
        if button == 0 && pages > 1 {
            let page = page.min(pages - 1);
            let to = if jei::inside(screen.mouse, forward) {
                (page + 1) % pages
            } else {
                (page + pages - 1) % pages
            };
            STATE.with(|s| s.borrow_mut().bookmark_page = to);
        }
        return;
    }
    let page = page.min(pages - 1);
    let hit = shown
        .then(|| area.hit(screen.mouse, page, bookmarks.len()))
        .flatten();
    let Some(index) = hit else {
        overlay_pressed(game, screen, button, modifiers, canceled);
        return;
    };
    *canceled = true;
    let stack = bookmarks.slots()[index].clone();
    match button {
        // Left: a click opens the recipes, a drag reorders.
        0 if modifiers & SHIFT == 0 => {
            STATE.with(|s| {
                s.borrow_mut().drag = Some(Drag {
                    from: index,
                    moved: false,
                })
            });
        }
        0 => remove_bookmark(index),
        // Right: the uses.
        1 => pending(Event::Open { stack, uses: true }),
        // Middle: take it away.
        2 => remove_bookmark(index),
        _ => {}
    }
}

/// A release over the list: a left click that never left its slot opens the
/// stack's recipes.
pub(super) fn screen_mouse_released(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    button: i32,
    _modifiers: u32,
    canceled: &mut bool,
) {
    if !game.native_on(jei::ID) {
        return;
    }
    if STATE.with(|s| std::mem::take(&mut s.borrow_mut().overlay_pressed)) {
        *canceled = true;
        return;
    }
    if !bar_screen(screen) || button != 0 {
        return;
    }
    let Some(drag) = STATE.with(|s| s.borrow_mut().drag.take()) else {
        return;
    };
    *canceled = true;
    if drag.moved {
        return;
    }
    let stack = STATE.with(|s| s.borrow().bookmarks.get(drag.from).cloned());
    if let Some(stack) = stack {
        pending(Event::Open { stack, uses: false });
    }
}

/// Dragging a bookmark over another slot moves it there.
pub(super) fn screen_mouse_dragged(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    button: i32,
    _delta: [f32; 2],
    _modifiers: u32,
    canceled: &mut bool,
) {
    if !game.native_on(jei::ID) {
        return;
    }
    if STATE.with(|s| s.borrow().overlay_pressed) {
        *canceled = true;
        return;
    }
    if !bar_screen(screen) || button != 0 {
        return;
    }
    let Some(mut drag) = STATE.with(|s| s.borrow().drag) else {
        return;
    };
    let (count, page) = STATE.with(|s| {
        let state = s.borrow();
        (state.bookmarks.len(), state.bookmark_page)
    });
    let area = bookmark_area(screen);
    let page = page.min(area.pages(count) - 1);
    let Some(to) = area.hit(screen.mouse, page, count) else {
        return;
    };
    *canceled = true;
    if to == drag.from {
        return;
    }
    let moved = STATE.with(|s| {
        let mut state = s.borrow_mut();
        let moved = state.bookmarks.move_slot(drag.from, to);
        if moved {
            save_bookmarks(&state.bookmarks);
        }
        moved
    });
    if moved {
        drag.from = to;
        drag.moved = true;
        STATE.with(|s| s.borrow_mut().drag = Some(drag));
    }
}

fn pending(event: Event) {
    STATE.with(|s| s.borrow_mut().pending.push(event));
}

/// Adds the viewer's lines to any item's tooltip: what makes it, what uses it
/// and whether it is bookmarked.
pub(super) fn tooltip(
    game: &Game<'_>,
    stack: &ItemStack,
    lines: &mut Vec<(horizon_client::text::Text, u32)>,
) {
    if !game.native_on(jei::ID) {
        return;
    }
    load();
    let components = crate::game::native_recipes::components(stack);
    let config = STATE.with(|s| s.borrow().config);
    let counts = STATE.with(|s| {
        let state = s.borrow();
        state.viewer().map_or(Counts::default(), |viewer| {
            viewer.counts(&stack.item, &components)
        })
    });
    let marked = STATE.with(|s| {
        s.borrow()
            .bookmarks
            .position(&Stack {
                item: stack.item.to_string(),
                count: 1,
                components: components.clone(),
            })
            .is_some()
    });
    let counts = if config.hints {
        counts
    } else {
        Counts::default()
    };
    let added = jei::tooltip_lines(counts, marked, lang(game));
    if added.is_empty() {
        return;
    }
    if !lines.is_empty() {
        lines.push((horizon_client::text::Text::literal(""), u32::MAX));
    }
    for (text, [r, g, b, a]) in added {
        let color = u32::from_le_bytes([r, g, b, a]);
        lines.push((horizon_client::text::Text::literal(&text), color));
    }
}

/// The settings page's config, as its buttons show it.
pub(super) fn settings() -> Config {
    config()
}

/// The next order, shown in the list at once.
pub(super) fn cycle_sort(game: &mut Game<'_>) {
    set_sort(game, config().sort.next());
}

pub(super) fn set_sort(game: &mut Game<'_>, sort: Sort) {
    let mut config = config();
    config.sort = sort;
    store_config(config);
    let out = STATE.with(|s| s.borrow_mut().viewer_mut().map(|v| v.set_sort(sort)));
    if let Some(out) = out {
        if let Err(error) = apply(game, out) {
            game.native_chat(&format!("JEI: {error}"));
        }
    }
}

pub(super) fn toggle_hints() -> Config {
    let mut config = config();
    config.hints = !config.hints;
    store_config(config);
    config
}

pub(super) fn toggle_search() -> Config {
    let mut config = config();
    config.centred_search = !config.centred_search;
    store_config(config);
    config
}

pub(super) fn toggle_bar() -> Config {
    let mut config = config();
    config.bookmarks = !config.bookmarks;
    store_config(config);
    config
}

/// Empties the bar, keeping the file.
pub(super) fn clear_bookmarks() {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.bookmarks.clear();
        state.drag = None;
        save_bookmarks(&state.bookmarks);
    });
}

/// What a "+" takes from, as JEI counts it: the inventory's 36 slots and the
/// open crafting grid, as the game shows them to the client. None on a mod's
/// crafting grid, which can take from sources the client doesn't list.
fn available(game: &Game<'_>) -> Option<Vec<Stack>> {
    let vanilla = STATE.with(|s| {
        s.borrow().screen.as_ref().is_some_and(|(id, menu)| {
            *id == "inventory" && matches!(menu.as_deref(), None | Some("minecraft:crafting"))
        })
    });
    if !vanilla {
        return None;
    }
    let cells = (game.craft_size * game.craft_size).min(game.craft_grid.len());
    let stacks = game
        .inventory
        .iter()
        .take(36)
        .chain(&game.craft_grid[..cells]);
    Some(
        stacks
            .flatten()
            .map(|s| Stack {
                item: s.item.to_string(),
                count: s.count,
                components: crate::game::native_recipes::components(s),
            })
            .collect(),
    )
}

/// The server's answer for the "+" of recipe `book`, when it has one.
fn server_check(book: Option<i32>) -> Option<NativeTransferCheck> {
    let book = book?;
    CHECK.with(|c| {
        (c.borrow().answer.as_ref())
            .filter(|(b, _)| *b == book)
            .map(|(_, a)| a.clone())
    })
}

/// The inputs of `view` the player lacks, when its "+" is on (else none):
/// the server's word when it has one, as it counts what the client can't see
/// (a crafting station's chests), else the client's count on a vanilla grid.
fn lacking(view: &View, have: Option<&[Stack]>, check: Option<&NativeTransferCheck>) -> Vec<usize> {
    if view.action != Some(true) {
        return Vec::new();
    }
    match (check, have) {
        (Some(check), _) if check.result == 1 => {
            let filled = |i: &usize| view.slots.get(*i).is_some_and(|s| !s.is_empty());
            let cells: Vec<usize> = (check.missing.iter())
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
        (Some(_), _) => Vec::new(),
        (None, Some(have)) => jei::missing_slots(&view.slots, have),
        (None, None) => Vec::new(),
    }
}

/// The recipe book display id of the shown recipe `view`.
fn view_book(view: usize) -> Option<i32> {
    STATE.with(|s| s.borrow().viewer().and_then(|v| v.shown_book(view)))
}

/// The crafting grid of the screen drawn last: 0 when none takes recipes.
fn open_grid() -> u32 {
    STATE.with(|s| {
        s.borrow().screen.as_ref().map_or(0, |(id, menu)| {
            let mod_grid = menu
                .as_deref()
                .and_then(horizon_client::containers::mod_menu_type)
                .and_then(|t| t.crafting)
                .map(|c| c.size as u32);
            jei::screen_grid(id, menu.as_deref(), mod_grid)
        })
    })
}

/// The server's recipes came in: the next frame builds the viewer from them.
pub(super) fn recipes_received() {
    STATE.with(|s| s.borrow_mut().rebuild = true);
}

fn dispatch(game: &mut Game<'_>, event: Event) {
    let grid = open_grid();
    let recipe_key = game.native_mod_key(jei::ID, "recipe", "key.keyboard.r");
    let uses_key = game.native_mod_key(jei::ID, "uses", "key.keyboard.u");
    let give = jei::give(game.native_setting(jei::ID, "give").as_deref());
    let lang = lang(game);
    let ctx = Context {
        recipe_key: &recipe_key,
        uses_key: &uses_key,
        give,
        // `native_give` succeeds in creative only: try, and fall back below.
        creative: give,
        grid,
    };

    let mut fallback = false;
    for out in handle(&event, &ctx, lang) {
        if let Out::Give { item, count } = out {
            if game.native_give(&item, count).is_err() {
                fallback = true;
            }
            continue;
        }
        if let Err(error) = apply(game, out) {
            game.native_chat(&format!("JEI: {error}"));
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
            if let Err(error) = apply(game, out) {
                game.native_chat(&format!("JEI: {error}"));
            }
        }
    }
}

fn handle(event: &Event, ctx: &Context<'_>, lang: jei::Lang) -> Vec<Out> {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        let Some(viewer) = state.viewer_mut() else {
            return Vec::new();
        };
        viewer.set_lang(lang);
        viewer.handle(event.clone(), ctx)
    })
}

/// The viewer's size, for `/jei`; none outside a world.
pub(super) fn counts() -> Option<(usize, usize)> {
    STATE.with(|s| {
        s.borrow()
            .viewer()
            .map(|viewer| (viewer.item_count(), viewer.recipe_count()))
    })
}

fn apply(game: &mut Game<'_>, out: Out) -> Result<(), String> {
    match out {
        Out::Items(stacks) => {
            game.native_panel_items(jei::ID, panel_stacks(stacks));
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
            if config().docked {
                game.native_panel_recipes(
                    jei::ID,
                    &title,
                    to_native(tabs),
                    tab,
                    views.into_iter().flat_map(native_views).collect(),
                );
                return Ok(());
            }
            if views.is_empty() {
                close_overlay(game);
                return Ok(());
            }
            let (open, same) = STATE.with(|s| {
                let state = s.borrow();
                let open = state.overlay.as_ref();
                (
                    open.map(|o| o.own_screen),
                    open.is_some_and(|o| o.title == title && o.tab == tab as usize),
                )
            });
            // Nothing open (a lookup from the world): JEI's own screen holds it.
            let own_screen = match open {
                Some(own) => own,
                None => {
                    let nothing_open = STATE.with(|s| s.borrow().screen.is_none());
                    nothing_open && game.native_mod_screen_open(jei::ID)
                }
            };
            // Notes wrap to the window's width (198, less its border and the
            // recipe's frame), so the geometry, which counts their lines, sizes them.
            let mut views = views;
            for view in &mut views {
                if !view.note.is_empty() {
                    view.note =
                        jei::wrap(&view.note, 176.0, |t| game.native_text_width(t)).join("\n");
                }
            }
            STATE.with(|s| {
                let mut state = s.borrow_mut();
                let page = match &state.overlay {
                    Some(o) if same => o.page,
                    _ => 0,
                };
                state.overlay = Some(Overlay {
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
        Out::Give { item, count } => game.native_give(&item, count),
        Out::Transfer { book_id, all } => {
            // All: as many sets as the items (and a station's sources) allow.
            game.native_transfer_recipe_all(book_id, all);
            Ok(())
        }
        Out::Say(text) => {
            game.native_chat(&text);
            Ok(())
        }
    }
}

/// The game's stacks. An empty list keeps one invisible slot: the panel (and
/// its search box and events) goes away when the list is empty.
fn panel_stacks(stacks: Vec<Stack>) -> Vec<NativeStack> {
    if stacks.is_empty() {
        return vec![NativeStack {
            item: "minecraft:air".to_owned(),
            count: 0,
            components: String::new(),
        }];
    }
    to_native(stacks)
}

fn to_native(stacks: Vec<Stack>) -> Vec<NativeStack> {
    stacks
        .into_iter()
        .map(|s| NativeStack {
            item: s.item,
            count: s.count,
            components: s.components,
        })
        .collect()
}

/// A bookmark as the game's stack, with the components `/give` reads.
fn item_stack(stack: &Stack) -> Option<ItemStack> {
    let (damage, extra) = if stack.components.is_empty() {
        (0, None)
    } else {
        let tag = horizon_client::item_data::command_components(&stack.components).ok()?;
        horizon_client::item_data::decode_components(Some(&tag))
    };
    Some(ItemStack {
        item: stack.item.as_str().into(),
        count: stack.count.clamp(1, 99),
        damage,
        extra,
    })
}

/// A view as the item panel draws it: a category recipe becomes one
/// "input -> output" per output, its chance and count in the note.
fn native_views(view: View) -> Vec<NativeRecipeView> {
    if view.layout != Layout::Chance {
        return vec![native_view(view)];
    }
    let input = view.slots.first().cloned().unwrap_or_default();
    view.chances
        .iter()
        .map(|c| {
            native_view(View {
                layout: Layout::Single,
                width: 1,
                height: 1,
                slots: vec![input.clone()],
                results: vec![c.stack.clone()],
                note: format!(
                    "{} {}",
                    jei::percent(c.chance),
                    jei::count_range(c.min, c.max)
                )
                .trim_end()
                .to_owned(),
                shapeless: false,
                action: None,
                chances: Vec::new(),
            })
        })
        .collect()
}

fn native_view(view: View) -> NativeRecipeView {
    NativeRecipeView {
        layout: match view.layout {
            Layout::Grid => NativeRecipeLayout::Grid,
            Layout::Cooking => NativeRecipeLayout::Cooking,
            Layout::Single => NativeRecipeLayout::Single,
            Layout::Smithing => NativeRecipeLayout::Smithing,
            Layout::Brewing => NativeRecipeLayout::Brewing,
            Layout::Chance => NativeRecipeLayout::Single,
        },
        width: view.width,
        height: view.height,
        slots: view.slots.into_iter().map(to_native).collect(),
        results: to_native(view.results),
        note: view.note,
        action: view.action,
    }
}

fn convert_event(event: &NativePanelEvent) -> Option<Event> {
    Some(match event {
        NativePanelEvent::Search(text) => Event::Search(text.clone()),
        NativePanelEvent::Clicked { index, right } => Event::Clicked {
            index: *index as usize,
            right: *right,
        },
        NativePanelEvent::Key { key, stack } => Event::Key {
            key: key.clone(),
            stack: stack_of(stack),
        },
        NativePanelEvent::Tab(tab) => Event::Tab(*tab as usize),
        NativePanelEvent::Back => Event::Back,
        NativePanelEvent::SlotClicked { stack, right, .. } => Event::SlotClicked {
            stack: stack_of(stack),
            right: *right,
        },
        NativePanelEvent::Action(index) => Event::Action(*index as usize, false),
        _ => return None,
    })
}

fn stack_of(stack: &NativeStack) -> Stack {
    Stack {
        item: stack.item.clone(),
        count: stack.count,
        components: stack.components.clone(),
    }
}

fn items(catalog: &NativeRecipes) -> Vec<Item> {
    catalog
        .items
        .iter()
        .map(|item| Item {
            id: item.id.clone(),
            name: item.name.clone(),
            tags: item.tags.clone(),
        })
        .collect()
}

fn recipes(catalog: &NativeRecipes) -> Vec<Recipe> {
    catalog
        .recipes
        .iter()
        // A recipe with no ingredients says nothing in a list of recipes.
        .filter(|recipe| !recipe.slots.is_empty())
        .map(|recipe| Recipe {
            kind: kind(recipe.kind),
            width: recipe.width,
            height: recipe.height,
            slots: ingredient_slots(&recipe.slots),
            results: ingredient_list(&recipe.results),
            cooking_time: recipe.cooking_time,
            experience: recipe.experience,
            book_id: recipe.book_id,
        })
        .collect()
}

fn ingredient_slots(slots: &[Vec<NativeRecipeStack>]) -> Vec<Vec<Ingredient>> {
    slots.iter().map(|slot| ingredient_list(slot)).collect()
}

fn ingredient_list(stacks: &[NativeRecipeStack]) -> Vec<Ingredient> {
    stacks
        .iter()
        .map(|stack| Ingredient {
            item: stack.item,
            count: stack.count,
            components: stack.components.clone(),
        })
        .collect()
}

fn kind(kind: NativeRecipeKind) -> Kind {
    match kind {
        NativeRecipeKind::CraftingShaped => Kind::CraftingShaped,
        NativeRecipeKind::CraftingShapeless => Kind::CraftingShapeless,
        NativeRecipeKind::Smelting => Kind::Smelting,
        NativeRecipeKind::Blasting => Kind::Blasting,
        NativeRecipeKind::Smoking => Kind::Smoking,
        NativeRecipeKind::Campfire => Kind::Campfire,
        NativeRecipeKind::Stonecutting => Kind::Stonecutting,
        NativeRecipeKind::Smithing => Kind::Smithing,
        NativeRecipeKind::Brewing => Kind::Brewing,
    }
}

// ---------------------------------------------------------------- the recipe screen

/// JEI's text colours: the header's white, a recipe's grey notes.
const WHITE: u32 = 0xFF_FF_FF;
const TEXT: u32 = 0x40_40_40;
const NOTE: u32 = 0x80_80_80;
/// JEI's bars between the header's buttons, its slot highlight.
const BAR: u32 = 0x30_00_00_00;
const LIT: u32 = 0x7F_FF_FF_FF;
/// JEI's red: over the slots a "+" lacks, and over its button.
const MISSING_SLOT: u32 = 0x66_FF_00_00;
const MISSING_BUTTON: u32 = 0x80_FF_00_00;
const LIST_LIT: u32 = 0x80_FF_FF_FF;
/// Vanilla's `drawDefaultBackground` over the world: its top and bottom.
const SHADE: [u32; 2] = [0xC0_10_10_10, 0xD0_10_10_10];

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
fn close_overlay(game: &mut Game<'_>) {
    let closed = STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.overlay_pressed = false;
        state.overlay.take()
    });
    if closed.is_some_and(|o| o.own_screen) {
        game.native_mod_screen_close(jei::ID);
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

/// JEI's GUI sprites (1.12's, MIT), from JEI's content half; when it isn't
/// installed the screen draws the same shapes with fills.
const SPRITES: &str = "jei:gui/sprites/";

/// Sprite `name` of [`SPRITES`] stretched over `r` as a nine-slice with
/// borders of `[left, top, right, bottom]` pixels; false when the sprite
/// isn't there.
fn nine(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    name: &str,
    r: [f32; 4],
    border: [f32; 4],
) -> bool {
    let name = format!("{SPRITES}{name}");
    let Some([sw, sh]) = game.native_sprite_size(&name) else {
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
                game.native_sprite(batch, &name, [sx, sy, sw_, sh_], [dx, dy, dw, dh]);
            }
        }
    }
    true
}

/// Sprite `name` of [`SPRITES`] whole at `r`; false when it isn't there.
fn whole(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    name: &str,
    r: [f32; 4],
) -> bool {
    let name = format!("{SPRITES}{name}");
    let Some([w, h]) = game.native_sprite_size(&name) else {
        return false;
    };
    game.native_sprite(batch, &name, [0.0, 0.0, w, h], r)
}

/// Icon `name` of [`SPRITES`] at its own size, centred in `r`, in JEI's
/// tint for a button that is `on` and `hovered` (its `_on`, `_off` and
/// `_hover` copies).
fn icon(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    name: &str,
    r: [f32; 4],
    on: bool,
    hovered: bool,
) -> bool {
    let tint = match (on, hovered) {
        (false, _) => "off",
        (true, true) => "hover",
        (true, false) => "on",
    };
    let name = format!("{SPRITES}icons/{name}_{tint}");
    let Some([w, h]) = game.native_sprite_size(&name) else {
        return false;
    };
    let at = [
        (r[0] + (r[2] - w) / 2.0).floor(),
        (r[1] + (r[3] - h) / 2.0).floor(),
        w,
        h,
    ];
    game.native_sprite(batch, &name, [0.0, 0.0, w, h], at)
}

/// The recipe window: JEI's grey panel.
fn frame_box(game: &Game<'_>, batch: &mut horizon_client::hud::HudBatch, r: [f32; 4]) {
    if nine(game, batch, "gui_background", r, [16.0; 4]) {
        return;
    }
    let [x, y, w, h] = r;
    game.native_fill(batch, r, 0xFF_00_00_00);
    game.native_fill(batch, [x + 1.0, y + 1.0, w - 2.0, h - 2.0], 0xFF_C6_C6_C6);
    game.native_fill(batch, [x + 1.0, y + 1.0, w - 3.0, 2.0], 0xFF_FF_FF_FF);
    game.native_fill(batch, [x + 1.0, y + 1.0, 2.0, h - 3.0], 0xFF_FF_FF_FF);
    game.native_fill(batch, [x + 3.0, y + h - 3.0, w - 4.0, 2.0], 0xFF_55_55_55);
    game.native_fill(batch, [x + w - 3.0, y + 3.0, 2.0, h - 4.0], 0xFF_55_55_55);
}

/// A slot: JEI's, else vanilla's colours.
fn slot_box(game: &Game<'_>, batch: &mut horizon_client::hud::HudBatch, r: [f32; 4]) {
    if nine(game, batch, "slot", r, [4.0; 4]) {
        return;
    }
    let [x, y, w, h] = r;
    game.native_fill(batch, r, 0xFF_8B_8B_8B);
    game.native_fill(batch, [x, y, w - 1.0, 1.0], 0xFF_37_37_37);
    game.native_fill(batch, [x, y, 1.0, h - 1.0], 0xFF_37_37_37);
    game.native_fill(batch, [x + 1.0, y + h - 1.0, w - 1.0, 1.0], 0xFF_FF_FF_FF);
    game.native_fill(batch, [x + w - 1.0, y + 1.0, 1.0, h - 1.0], 0xFF_FF_FF_FF);
}

/// One of JEI's buttons (`GuiIconButton`, `GuiIconButtonSmall`) with its
/// icon: a vanilla button, dark when off, white-edged under the pointer.
fn button_box(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    r: [f32; 4],
    name: &str,
    on: bool,
    hovered: bool,
) {
    let hovered = hovered && on;
    let sprite = match (on, hovered) {
        (false, _) => "button_disabled",
        (true, true) => "button_highlight",
        (true, false) => "button_enabled",
    };
    if !nine(game, batch, sprite, r, [2.0; 4]) {
        let fill = if on { 0xFF_6E_6E_6E } else { 0xFF_2B_2B_2B };
        game.native_fill(
            batch,
            r,
            if hovered {
                0xFF_FF_FF_FF
            } else {
                0xFF_00_00_00
            },
        );
        game.native_fill(
            batch,
            [r[0] + 1.0, r[1] + 1.0, r[2] - 2.0, r[3] - 2.0],
            fill,
        );
    }
    icon(game, batch, name, r, on, hovered);
}

fn draw_stack(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    stack: &Stack,
    r: [f32; 4],
) {
    if let Some(item) = item_stack(stack) {
        let count = (stack.count > 1).then(|| stack.count.to_string());
        game.native_gui_draw_item_with(batch, &item, [r[0] + 1.0, r[1] + 1.0], count.as_deref());
    }
}

/// The text cut with "..." to fit `max`.
fn fitted(game: &Game<'_>, text: &str, max: f32) -> String {
    let mut text = text.to_owned();
    while game.native_text_width(&text) > max && text.chars().count() > 3 {
        let keep: String = text.chars().take(text.chars().count() - 4).collect();
        text = format!("{keep}...");
    }
    text
}

/// The text centred on `x`, cut with "..." to fit `max`.
#[allow(clippy::too_many_arguments)]
fn centred(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    text: &str,
    x: f32,
    y: f32,
    max: f32,
    rgb: u32,
    shadow: bool,
) {
    let text = fitted(game, text, max);
    let w = game.native_text_width(&text);
    game.native_text(batch, &text, [(x - w / 2.0).floor(), y], rgb, shadow);
}

/// The item list's left edge and the search box's row (with its settings
/// button), as Horizon lays out JEI's classic panel beside `window` (the
/// screen's window, `[x, y, w, h]`) with the recipe screen open; None for a
/// screen with no item list.
fn panel_parts(
    gui: [f32; 2],
    screen: &NativeScreenView<'_>,
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

/// Vanilla's darkened background (`drawDefaultBackground`) over `r`, top to
/// bottom.
fn shade(game: &Game<'_>, batch: &mut horizon_client::hud::HudBatch, r: [f32; 4], gui_h: f32) {
    let [x, y, w, h] = r;
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    // Strips of the gradient, by where they are on the screen.
    let strips = ((h / 8.0).ceil() as usize).max(1);
    for n in 0..strips {
        let top = y + n as f32 * h / strips as f32;
        let t = ((top + h / strips as f32 / 2.0) / gui_h.max(1.0)).clamp(0.0, 1.0);
        let alpha = (0xC0 as f32 + t * (0xD0 - 0xC0) as f32).round() as u32;
        let color = (alpha << 24) | (SHADE[0] & 0x00_FF_FF_FF);
        game.native_fill(batch, [x, top, w, h / strips as f32], color);
    }
}

/// The screen under the recipe screen, darkened as vanilla darkens the
/// world under a screen: all of it but the item list and its search box,
/// which stay as JEI's overlay stayed over its recipe screen.
fn shade_under(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    panel: Option<(f32, [f32; 4])>,
) {
    let [gw, gh] = [batch.width, batch.height];
    let Some((left, row)) = panel else {
        shade(game, batch, [0.0, 0.0, gw, gh], gh);
        return;
    };
    let [rx, ry, rw, rh] = row;
    if rx >= left {
        // The search box is under the list: everything left of it.
        shade(game, batch, [0.0, 0.0, left, gh], gh);
        return;
    }
    shade(game, batch, [0.0, 0.0, left, ry], gh);
    shade(game, batch, [0.0, ry, rx, gh - ry], gh);
    shade(game, batch, [rx + rw, ry, left - rx - rw, gh - ry], gh);
    shade(game, batch, [rx, ry + rh, rw, gh - ry - rh], gh);
}

/// The vanilla background a recipe is drawn on, from the player's own
/// game files: the crafting table's grid and arrow, the furnace's slots.
fn draw_background(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    overlay: &Overlay,
    view: &View,
    placed: &geo::Placed,
) {
    let [x, y, w, h] = placed.rect;
    match placed.background {
        geo::Background::Crafting => {
            let src = [29.0, 16.0, geo::CRAFTING[0], geo::CRAFTING[1]];
            let dst = [x, y, geo::CRAFTING[0], geo::CRAFTING[1]];
            if !game.native_sprite(batch, "gui/container/crafting_table", src, dst) {
                for (_, rect) in &placed.slots {
                    slot_box(game, batch, *rect);
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
            if game.native_sprite(batch, texture, src, dst) {
                // The flame burning down over 15 s and the arrow filling
                // over 10, as JEI animates them.
                let burnt = (cycle(15_000) * 14.0).floor();
                let flame = "gui/sprites/container/furnace/lit_progress";
                game.native_sprite(
                    batch,
                    flame,
                    [0.0, burnt, 14.0, 14.0 - burnt],
                    [x + 1.0, y + 20.0 + burnt, 14.0, 14.0 - burnt],
                );
                let done = (cycle(10_000) * 24.0).floor();
                let arrow = "gui/sprites/container/furnace/burn_progress";
                if done > 0.0 {
                    game.native_sprite(
                        batch,
                        arrow,
                        [0.0, 0.0, done, 16.0],
                        [x + 24.0, y + 18.0, done, 16.0],
                    );
                }
            } else {
                for (_, rect) in &placed.slots {
                    slot_box(game, batch, *rect);
                }
            }
            // The time and experience at the top right, as JEI's.
            if !view.note.is_empty() {
                let note = view.note.replace('\n', " ");
                let tw = game.native_text_width(&note);
                game.native_text(batch, &note, [x + w - tw, y], NOTE, false);
            }
        }
        geo::Background::Slots => {
            for (_, rect) in &placed.slots {
                slot_box(game, batch, *rect);
            }
            if let Some([ax, ay, aw, ah]) = placed.arrow {
                let src = [90.0, 35.0, 22.0, 15.0];
                if !game.native_sprite(batch, "gui/container/crafting_table", src, [ax, ay, aw, ah])
                {
                    let mid = ay + ah / 2.0;
                    game.native_fill(batch, [ax, mid - 2.0, aw - 7.0, 4.0], 0xFF_8B_8B_8B);
                    game.native_quad(
                        batch,
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
    let _ = h;
}

/// The recipe screen, as JEI 1.12's `RecipesGui` draws it, over the screen
/// it opened on (`screen`: where it is and the pointer).
fn draw_overlay(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &ScreenAt,
    overlay: &Overlay,
) {
    let gui = [batch.width, batch.height];
    let mouse = screen.mouse;
    let g = overlay_geom(overlay, gui);
    let lang = lang(game);
    let hit = overlay_hit(overlay, gui, mouse);
    shade_under(game, batch, screen.panel);
    // The bookmarks stay, as JEI's left list stays over its recipe screen.
    draw_bookmarks(game, batch, mouse, g.left().min(screen.rect[0]));

    frame_box(game, batch, g.panel);
    game.native_fill(batch, g.title_bar, BAR);
    game.native_fill(batch, g.page_bar, BAR);
    let room = g.title_bar[2] - 2.0 * 14.0;
    let name = overlay.names.get(overlay.tab).cloned().unwrap_or_default();
    let [bx, by, bw, _] = g.title_bar;
    centred(
        game,
        batch,
        &name,
        bx + bw / 2.0,
        by + 2.0,
        room,
        WHITE,
        true,
    );
    let pages = overlay_pages(overlay, &g);
    let page = overlay.page.min(pages.len().saturating_sub(1));
    let [px, py, pw, _] = g.page_bar;
    let page_text = lang.page_of(page, pages.len());
    centred(
        game,
        batch,
        &page_text,
        px + pw / 2.0,
        py + 2.0,
        pw,
        WHITE,
        true,
    );
    let many_tabs = overlay.tabs.len() > 1;
    let many_pages = pages.len() > 1;
    for (rect, name, on, which) in [
        (g.prev_tab, "arrow_previous", many_tabs, Hit::PrevTab),
        (g.next_tab, "arrow_next", many_tabs, Hit::NextTab),
        (g.prev_page, "arrow_previous", many_pages, Hit::PrevPage),
        (g.next_page, "arrow_next", many_pages, Hit::NextPage),
    ] {
        button_box(game, batch, rect, name, on, hit.as_ref() == Some(&which));
    }
    if !overlay.subtitle.is_empty() {
        let [cx, cy, cw, _] = g.content;
        let line = overlay.subtitle.replace('\n', " ");
        centred(
            game,
            batch,
            &line,
            cx + cw / 2.0,
            cy + 2.0,
            cw - 12.0,
            TEXT,
            false,
        );
    }

    let content = overlay_content(overlay, &g);
    let have = available(game);
    let mut lacks_hovered = false;
    let hovered_book = match &hit {
        Some(Hit::Plus(v, true)) => view_book(*v),
        _ => None,
    };
    CHECK.with(|c| c.borrow_mut().hovered = hovered_book);
    let check = server_check(hovered_book);
    if let Some(pieces) = pages.get(page) {
        for (n, at) in geo::layout(&overlay.views, pieces, content) {
            let piece = &pieces[n];
            let view = &overlay.views[piece.view];
            let placed = geo::place(view, piece, at, content[2]);
            let [x, y, w, h] = placed.rect;
            let f = geo::FRAME;
            if !nine(
                game,
                batch,
                "single_recipe_background",
                [x - f, y - f, w + 2.0 * f, h + 2.0 * f],
                [16.0; 4],
            ) {
                game.native_fill(
                    batch,
                    [x - f, y - f, w + 2.0 * f, h + 2.0 * f],
                    0x20_00_00_00,
                );
            }
            draw_background(game, batch, overlay, view, &placed);
            let plus_hovered = matches!(hit, Some(Hit::Plus(v, _)) if v == piece.view);
            let lacks = lacking(
                view,
                have.as_deref(),
                check.as_ref().filter(|_| plus_hovered),
            );
            lacks_hovered |= plus_hovered && !lacks.is_empty();
            if let Some(rect) = placed.shapeless
                && !whole(game, batch, "icons/shapeless_icon", rect)
            {
                game.native_text(batch, "*", [rect[0] + 2.0, rect[1]], NOTE, false);
            }
            for (part, rect) in &placed.slots {
                let stack = match part {
                    geo::Part::Input(i) => view.slots.get(*i).and_then(|s| cycled(s)),
                    geo::Part::Result(i) => view.results.get(*i),
                };
                if let Some(stack) = stack {
                    draw_stack(game, batch, stack, *rect);
                }
                // JEI marks what the "+" lacks while the pointer is on it.
                if plus_hovered && matches!(part, geo::Part::Input(i) if lacks.contains(i)) {
                    game.native_fill(
                        batch,
                        [rect[0] + 1.0, rect[1] + 1.0, 16.0, 16.0],
                        MISSING_SLOT,
                    );
                }
                if jei::inside(mouse, *rect) && stack.is_some() {
                    game.native_fill(batch, [rect[0] + 1.0, rect[1] + 1.0, 16.0, 16.0], LIT);
                }
                if let (geo::Part::Result(i), Layout::Chance) = (part, view.layout)
                    && let Some(c) = view.chances.get(*i)
                {
                    let mark = if c.notes.is_empty() { "" } else { "*" };
                    let tx = rect[0] + geo::SLOT + 2.0;
                    game.native_text(
                        batch,
                        &format!("{}{mark}", jei::percent(c.chance)),
                        [tx, rect[1] + 1.0],
                        TEXT,
                        false,
                    );
                    let range = jei::count_range(c.min, c.max);
                    if !range.is_empty() {
                        game.native_text(batch, &range, [tx, rect[1] + 10.0], NOTE, false);
                    }
                }
            }
            if let (Some(rect), Some(on)) = (placed.button, view.action) {
                let hovered = hit == Some(Hit::Plus(piece.view, on));
                let on = on && lacks.is_empty();
                button_box(game, batch, rect, "recipe_transfer", on, hovered);
                if !lacks.is_empty() {
                    game.native_fill(batch, rect, MISSING_BUTTON);
                }
            }
            let notes = !view.note.is_empty()
                && !view.shapeless
                && view.layout != Layout::Cooking
                && piece.outputs.start == 0;
            if notes {
                for (k, text) in view.note.split('\n').enumerate() {
                    game.native_text(
                        batch,
                        text,
                        [placed.notes[0], placed.notes[1] + k as f32 * geo::LINE],
                        NOTE,
                        false,
                    );
                }
            }
        }
    }

    // The catalysts' column, fused to the window's left edge.
    if !g.catalysts.is_empty() {
        let [cx, cy, cw, ch] = g.catalyst_panel;
        if !nine(
            game,
            batch,
            "catalyst_tab",
            g.catalyst_panel,
            [8.0, 8.0, 9.0, 8.0],
        ) {
            frame_box(game, batch, g.catalyst_panel);
        }
        slot_box(game, batch, [cx + 5.0, cy + 5.0, cw - 10.0, ch - 10.0]);
        for (stack, rect) in overlay.catalysts.iter().zip(&g.catalysts) {
            let at = [rect[0] - 1.0, rect[1] - 1.0, geo::SLOT, geo::SLOT];
            draw_stack(game, batch, stack, at);
            if jei::inside(mouse, *rect) {
                game.native_fill(batch, *rect, LIT);
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
        if !whole(game, batch, sprite, *rect) {
            let r = if selected {
                *rect
            } else {
                [rect[0], rect[1] + 2.0, rect[2], rect[3] - 2.0]
            };
            frame_box(game, batch, r);
            if !selected {
                game.native_fill(batch, r, 0x30_00_00_00);
            }
        }
        if let Some(icon) = overlay.tabs.get(*i) {
            draw_stack(
                game,
                batch,
                icon,
                [rect[0] + 3.0, rect[1] + 3.0, 18.0, 18.0],
            );
        }
    }
    if g.tab_pages > 1 {
        let [back, forward] = g.tab_buttons();
        button_box(
            game,
            batch,
            back,
            "arrow_previous",
            true,
            hit == Some(Hit::PrevTabs),
        );
        button_box(
            game,
            batch,
            forward,
            "arrow_next",
            true,
            hit == Some(Hit::NextTabs),
        );
        let text = lang.page_of(g.tab_page, g.tab_pages);
        let [nx, ny, nw, nh] = g.tab_nav;
        centred(
            game,
            batch,
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
        }) => overlay_stack_lines(game, overlay, stack, *view, *output),
        Some(Hit::Plus(..)) if lacks_hovered => vec![format!("§c{}", lang.missing_hint())],
        Some(Hit::Plus(_, on)) => vec![format!("§f{}", lang.transfer_hint(*on))],
        Some(Hit::Shapeless) => vec![format!("§f{}", lang.shapeless_hint())],
        Some(Hit::Tab(i)) => overlay
            .names
            .get(*i)
            .map(|n| vec![format!("§f{n}")])
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    if !lines.is_empty() {
        let texts: Vec<horizon_client::text::Text> = lines
            .iter()
            .map(|l| horizon_client::text::Text::literal(l.as_str()).resolve_legacy())
            .collect();
        game.native_gui_tooltip(batch, &texts, mouse);
    }
}

/// A stack's tooltip on the recipe screen: its name, a category output's
/// chance, count and notes, and JEI's hints.
fn overlay_stack_lines(
    game: &Game<'_>,
    overlay: &Overlay,
    stack: &Stack,
    view: Option<usize>,
    output: Option<usize>,
) -> Vec<String> {
    let lang = lang(game);
    let (name, counts) = STATE.with(|s| {
        let state = s.borrow();
        state
            .viewer()
            .map_or((stack.item.clone(), Counts::default()), |v| {
                (
                    v.item_name(&stack.item),
                    v.counts(&stack.item, &stack.components),
                )
            })
    });
    let mut lines = vec![format!("§f{name}")];
    if let (Some(v), Some(o)) = (view, output)
        && let Some(c) = overlay.views.get(v).and_then(|v| v.chances.get(o))
    {
        let range = jei::count_range(c.min, c.max);
        lines.push(format!(
            "§e{}{}",
            jei::percent(c.chance),
            if range.is_empty() {
                String::new()
            } else {
                format!(" {range}")
            }
        ));
        for note in &c.notes {
            lines.push(format!("§7{note}"));
        }
    }
    let marked = STATE.with(|s| s.borrow().bookmarks.position(stack).is_some());
    for (text, _) in jei::tooltip_lines(counts, marked, lang) {
        lines.push(format!("§8{text}"));
    }
    lines
}

/// A press on the recipe screen: it takes every press on its window, and
/// the presses on the screen under it (it is modal), but not the item list
/// beside the screen or the bookmarks.
fn overlay_pressed(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    button: i32,
    modifiers: u32,
    canceled: &mut bool,
) {
    // GLFW's shift bit: "+" with Shift moves as many sets as the items allow.
    let shift = modifiers & 1 != 0;
    let gui = STATE.with(|s| s.borrow().gui);
    let hit = STATE.with(|s| {
        let state = s.borrow();
        state
            .overlay
            .as_ref()
            .map(|o| overlay_hit(o, gui, screen.mouse))
    });
    let Some(hit) = hit else {
        return;
    };
    let Some(hit) = hit else {
        // Outside the window: the item list and its search box still take
        // clicks; the screen under the window doesn't.
        let panel = STATE.with(|s| s.borrow().last_at.and_then(|at| at.panel));
        let on_list = panel
            .is_some_and(|(left, row)| screen.mouse[0] >= left || jei::inside(screen.mouse, row));
        if !on_list {
            *canceled = true;
            STATE.with(|s| s.borrow_mut().overlay_pressed = true);
        }
        return;
    };
    *canceled = true;
    // A "+" that lacks items does nothing, as JEI's.
    let plus_ready = match hit {
        Hit::Plus(view, true) => {
            let have = available(game);
            let check = server_check(view_book(view));
            STATE.with(|s| {
                let state = s.borrow();
                let view = state.overlay.as_ref().and_then(|o| o.views.get(view));
                view.is_none_or(|v| lacking(v, have.as_deref(), check.as_ref()).is_empty())
            })
        }
        _ => false,
    };
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.overlay_pressed = true;
        let tabs = state.overlay.as_ref().map_or(0, |o| o.tabs.len());
        let tab = state.overlay.as_ref().map_or(0, |o| o.tab);
        let pages = state
            .overlay
            .as_ref()
            .map_or(1, |o| overlay_pages(o, &overlay_geom(o, gui)).len().max(1));
        let tab_pages = state.overlay.as_ref().map(|o| {
            let g = overlay_geom(o, gui);
            (g.tab_page, g.tab_pages)
        });
        match hit {
            Hit::Tab(i) => state.pending.push(Event::Tab(i)),
            Hit::PrevTabs | Hit::NextTabs if button == 0 => {
                if let (Some(o), Some((page, count))) = (state.overlay.as_mut(), tab_pages) {
                    let count = count.max(1);
                    o.tab_page = Some(if hit == Hit::NextTabs {
                        (page + 1) % count
                    } else {
                        (page + count - 1) % count
                    });
                }
            }
            Hit::PrevTab if tabs > 0 => state.pending.push(Event::Tab((tab + tabs - 1) % tabs)),
            Hit::NextTab if tabs > 0 => state.pending.push(Event::Tab((tab + 1) % tabs)),
            Hit::PrevPage | Hit::NextPage => {
                if let Some(o) = state.overlay.as_mut() {
                    let page = o.page.min(pages - 1);
                    o.page = if hit == Hit::NextPage {
                        (page + 1) % pages
                    } else {
                        (page + pages - 1) % pages
                    };
                }
            }
            Hit::Stack { stack, .. } if button == 0 || button == 1 => {
                state.pending.push(Event::Open {
                    stack,
                    uses: button == 1,
                });
            }
            Hit::Plus(view, true) if button == 0 && plus_ready => {
                // JEI moves the items and goes back to the screen they went to.
                state.pending.push(Event::Action(view, shift));
                state.pending.push(Event::Close);
            }
            _ => {}
        }
    });
    let _ = game;
}

/// The wheel over the recipe screen turns its pages.
pub(super) fn screen_mouse_scrolled(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    scroll: [f64; 2],
    _modifiers: u32,
    canceled: &mut bool,
) {
    if !game.native_on(jei::ID) {
        return;
    }
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        let gui = state.gui;
        let Some(o) = state.overlay.as_mut() else {
            return;
        };
        if overlay_hit(o, gui, screen.mouse).is_none() || scroll[1] == 0.0 {
            return;
        }
        *canceled = true;
        let pages = overlay_pages(o, &overlay_geom(o, gui)).len().max(1);
        let page = o.page.min(pages - 1);
        o.page = if scroll[1] < 0.0 {
            (page + 1).min(pages - 1)
        } else {
            page.saturating_sub(1)
        };
    });
}

/// Keys on the recipe screen: Escape and Backspace go back a page (closing
/// it at the first), and the viewer's keys (recipes, uses, bookmark) act on
/// the stack under the pointer.
pub(super) fn screen_key_pressed(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    key: &str,
    _repeat: bool,
    _modifiers: u32,
    canceled: &mut bool,
) {
    if !game.native_on(jei::ID) {
        return;
    }
    let gui = STATE.with(|s| s.borrow().gui);
    let hit = STATE.with(|s| {
        let state = s.borrow();
        state
            .overlay
            .as_ref()
            .map(|o| overlay_hit(o, gui, screen.mouse))
    });
    let Some(hit) = hit else {
        return;
    };
    if key == "key.keyboard.escape" || (key == "key.keyboard.backspace" && hit.is_some()) {
        *canceled = true;
        pending(Event::Back);
        return;
    }
    let recipe = game.native_mod_key(jei::ID, "recipe", "key.keyboard.r");
    let uses = game.native_mod_key(jei::ID, "uses", "key.keyboard.u");
    let mark = game.native_mod_key(jei::ID, "bookmark", jei::BOOKMARK_KEY);
    if ![&recipe, &uses, &mark].contains(&&key.to_owned()) || hit.is_none() {
        return;
    }
    // Over the window, its keys never reach the slots hidden under it.
    *canceled = true;
    let Some(Hit::Stack { stack, .. }) = hit else {
        return;
    };
    if key == mark {
        STATE.with(|s| s.borrow_mut().to_bookmark.push(stack));
    } else {
        pending(Event::Open {
            stack,
            uses: key == uses,
        });
    }
}

/// The screen under the recipe screen closed (or JEI's own did): so does
/// the recipe screen, and the viewer forgets its history.
pub(super) fn screen_closing(game: &Game<'_>, _screen: &NativeScreenView<'_>) {
    if !game.native_on(jei::ID) {
        return;
    }
    let open = STATE.with(|s| {
        let mut state = s.borrow_mut();
        if let Some(o) = state.overlay.as_mut() {
            // JEI's own screen is closing: nothing to close after it.
            o.own_screen = false;
            true
        } else {
            false
        }
    });
    if open {
        pending(Event::Close);
    }
}

/// The settings page's "Recipes" button: the recipe screen or the panel.
pub(super) fn toggle_docked(game: &mut Game<'_>) -> Config {
    let mut config = config();
    config.docked = !config.docked;
    store_config(config);
    close_overlay(game);
    dispatch(game, Event::Close);
    config
}

// ---------------------------------------------------------------- Horizon's recipe viewer API

/// The mods' categories (`native_collect_recipe_categories`), asked again
/// whenever their version moves (a mod's server data arrived).
fn refresh_categories(game: &mut Game<'_>) {
    let version = game.native_recipe_categories_version();
    let current = STATE.with(|s| {
        let state = s.borrow();
        state.viewer().is_none() || state.categories == Some(version)
    });
    if current {
        return;
    }
    let categories: Vec<jei::Category> = game
        .native_collect_recipe_categories()
        .into_iter()
        .map(|c| jei::Category {
            id: c.id,
            title: c.title,
            subtitle: c.subtitle,
            icon: stack_of(&c.icon),
            catalysts: c.catalysts.iter().map(stack_of).collect(),
            recipes: c
                .recipes
                .into_iter()
                .map(|r| jei::ChanceRecipe {
                    inputs: (r.inputs.iter())
                        .map(|slot| slot.iter().map(stack_of).collect())
                        .collect(),
                    outputs: (r.outputs.iter())
                        .map(|o| jei::Chance {
                            stack: stack_of(&o.stack),
                            chance: o.chance,
                            min: o.min,
                            max: o.max,
                            notes: o.notes.clone(),
                        })
                        .collect(),
                    notes: r.notes,
                })
                .collect(),
        })
        .collect();
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.categories = Some(version);
        if let Some(viewer) = state.viewer_mut() {
            viewer.set_categories(categories);
        }
    });
}

/// Another mod's lookup (`native_recipe_lookup`): recipes, uses, or a key
/// over its stack, which does what it does over JEI's list (R, U, A).
pub(super) fn lookup(
    game: &mut Game<'_>,
    stack: &NativeStack,
    lookup: &crate::game::native_hooks::NativeRecipeLookup,
    handled: &mut bool,
) {
    use crate::game::native_hooks::NativeRecipeLookup as L;
    if !game.native_on(jei::ID) || STATE.with(|s| s.borrow().viewer().is_none()) {
        return;
    }
    let stack = stack_of(stack);
    match lookup {
        L::Recipes => dispatch(game, Event::Open { stack, uses: false }),
        L::Uses => dispatch(game, Event::Open { stack, uses: true }),
        L::Key(key) => {
            let mark = game.native_mod_key(jei::ID, "bookmark", jei::BOOKMARK_KEY);
            if *key == mark {
                bookmark(game, stack);
            } else {
                dispatch(
                    game,
                    Event::Key {
                        key: key.clone(),
                        stack,
                    },
                );
            }
        }
        _ => return,
    }
    *handled = true;
}
