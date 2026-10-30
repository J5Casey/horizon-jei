// JEI for Horizon: an item and recipe viewer built on Horizon's native hook
// points. This file is both the mod's Cargo library (its tests run with
// `cargo test`) and the module the bundle adds to the game as crate::game::jei.
//
// Phase 0: the bundle's skeleton. It registers its row in Options > Mods, a
// settings page with one toggle, and the /jei command, to prove the wiring
// before the viewer itself (tasks 1.x and 2.x in tasks/todo.md).

/// The mod's id: its row, its settings folder and its keys.
pub const ID: &str = "jei";
pub const NAME: &str = "JEI";
pub const VERSION: &str = "0.0.1";
pub const DESCRIPTION: &str = "An item and recipe viewer: every item beside container screens, with search, recipes and uses.";

/// Its keys: name, label and default (the Java mod's R and U).
pub const KEYS: [[&str; 3]; 2] = [
    ["recipe", "Show recipes", "key.keyboard.r"],
    ["uses", "Show uses", "key.keyboard.u"],
];

/// `JEI 0.0.1`, as Options > Mods lists it.
pub fn title() -> String {
    format!("{NAME} {VERSION}")
}

/// The line `/jei` says in chat.
pub fn about() -> String {
    format!("{NAME} {VERSION} (alpha) on hook points version 2")
}

/// Whether the debug setting is on: `true` unless the player turned it off.
pub fn debug(saved: Option<&str>) -> bool {
    saved != Some("false")
}

/// The settings page's one button.
pub fn debug_button(on: bool) -> String {
    format!("Debug: {}", if on { "ON" } else { "OFF" })
}

/// `/jei` and `/jei debug`, without the slash. `None` for another command.
pub fn command(args: &str) -> Option<Command> {
    match args.trim() {
        "" | "about" => Some(Command::About),
        "debug" => Some(Command::ToggleDebug),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    About,
    ToggleDebug,
}

/// One word of a search, as JEI's `SearchTokenizer` builds it (MIT, © mezz:
/// `Gui/src/main/java/mezz/jei/gui/search/SearchTokenizer.java`). `start` and
/// `end` are character offsets, for highlighting; `operator` is a `|` between
/// alternatives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub text: String,
    pub exclusion: bool,
    pub operator: bool,
    pub start: usize,
    pub end: usize,
}

impl Token {
    fn operator(at: usize) -> Self {
        Self {
            text: "|".to_owned(),
            exclusion: false,
            operator: true,
            start: at,
            end: at + 1,
        }
    }
}

/// Splits a search into tokens: words, quoted phrases, `-exclusions` and `|`
/// alternatives. Escapes (`\"`) keep a quote in a word or phrase. Unicode
/// offsets count characters, not UTF-16 units.
pub fn tokenize(filter: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    if filter.is_empty() {
        return tokens;
    }
    let mut current = String::new();
    let mut inside_quotes = false;
    let mut exclusion = false;
    let mut escaped = false;
    let mut start: Option<usize> = None;
    let mut end: Option<usize> = None;

    for (at, c) in filter.char_indices() {
        if escaped {
            current.push(c);
            end = Some(at + c.len_utf8());
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            start.get_or_insert(at);
            end = Some(at + 1);
            continue;
        }
        if c == '"' {
            if inside_quotes {
                add_token(&mut tokens, &current, exclusion, start, Some(at + 1));
                current.clear();
                inside_quotes = false;
                exclusion = false;
                start = None;
                end = None;
            } else if !current.is_empty() && !is_prefix(&current) {
                add_token(&mut tokens, &current, exclusion, start, end);
                current.clear();
                exclusion = false;
                start = None;
                end = None;
            } else {
                inside_quotes = true;
                start.get_or_insert(at);
                end = Some(at + 1);
            }
            continue;
        }
        if !inside_quotes && c.is_whitespace() {
            if !current.is_empty() {
                add_token(&mut tokens, &current, exclusion, start, end);
                current.clear();
            }
            exclusion = false;
            start = None;
            end = None;
            continue;
        }
        if !inside_quotes && c == '|' {
            if !current.is_empty() {
                add_token(&mut tokens, &current, exclusion, start, end);
                current.clear();
            }
            tokens.push(Token::operator(at));
            exclusion = false;
            start = None;
            end = None;
            continue;
        }
        if !inside_quotes && current.is_empty() && c == '-' {
            exclusion = true;
            start = Some(at);
            end = Some(at + 1);
            continue;
        }
        start.get_or_insert(at);
        end = Some(at + c.len_utf8());
        current.push(c);
    }

    if !current.is_empty() || inside_quotes {
        add_token(&mut tokens, &current, exclusion, start, end);
    }
    tokens
}

fn add_token(
    tokens: &mut Vec<Token>,
    content: &str,
    exclusion: bool,
    start: Option<usize>,
    end: Option<usize>,
) {
    let text = content.trim();
    if !text.is_empty() {
        tokens.push(Token {
            text: text.to_owned(),
            exclusion,
            operator: false,
            start: start.unwrap_or_default(),
            end: end.unwrap_or_default(),
        });
    }
}

/// A single character that is not a word, `-` or `|`: the `@`, `#`, `$`, `%`
/// prefixes, which mark a quoted phrase as prefixed (`@"two words"`).
fn is_prefix(text: &str) -> bool {
    let mut chars = text.chars();
    let (Some(c), None) = (chars.next(), chars.next()) else {
        return false;
    };
    !c.is_alphanumeric() && c != '-' && c != '|'
}

/// The tokens grouped by `|`: each group is one alternative of the search; a
/// group is a conjunction of its tokens.
pub fn split_by_operator(tokens: &[Token]) -> Vec<Vec<Token>> {
    let mut groups: Vec<Vec<Token>> = Vec::new();
    let mut current: Vec<Token> = Vec::new();
    for token in tokens {
        if token.operator {
            groups.push(std::mem::take(&mut current));
        } else {
            current.push(token.clone());
        }
    }
    groups.push(current);
    groups
}

/// One item as the search sees it.
pub struct SearchItem<'a> {
    pub name: &'a str,
    pub id: &'a str,
    /// Tags without the `#` (`minecraft:logs`).
    pub tags: &'a [String],
}

/// Whether `item` passes the search: any `|` alternative matches, and every
/// token of it matches. Prefixes: `@` namespace, `#` tag; `$` (tooltip) and
/// `%` (creative tab) have no data yet and match nothing.
pub fn keeps(item: &SearchItem<'_>, search: &str) -> bool {
    let groups = split_by_operator(&tokenize(search));
    groups
        .iter()
        .any(|group| group.iter().all(|token| token_matches(item, token)))
}

fn token_matches(item: &SearchItem<'_>, token: &Token) -> bool {
    let text = token.text.to_lowercase();
    let hit = if let Some(namespace) = text.strip_prefix('@') {
        item.id
            .split(':')
            .next()
            .is_some_and(|ns| ns.starts_with(namespace))
    } else if let Some(tag) = text.strip_prefix('#') {
        item.tags.iter().any(|t| t.to_lowercase().contains(tag))
    } else if text.starts_with('$') || text.starts_with('%') {
        false
    } else {
        item.name.to_lowercase().contains(&text) || item.id.to_lowercase().contains(&text)
    };
    hit != token.exclusion
}

#[cfg(test)]
mod tests {
    #[test]
    fn title_has_name_and_version() {
        assert_eq!(super::title(), "JEI 0.0.1");
    }

    #[test]
    fn about_names_the_hook_points() {
        assert!(super::about().contains("hook points version 2"));
    }

    #[test]
    fn debug_defaults_to_on() {
        assert!(super::debug(None));
        assert!(!super::debug(Some("false")));
        assert!(super::debug(Some("true")));
    }

    #[test]
    fn commands() {
        assert_eq!(super::command(""), Some(super::Command::About));
        assert_eq!(super::command("  about "), Some(super::Command::About));
        assert_eq!(super::command("debug"), Some(super::Command::ToggleDebug));
        assert_eq!(super::command("config"), None);
    }

    // SearchTokenizerTest (JEI, MIT): the same cases, with character offsets.
    fn token(text: &str, exclusion: bool, start: usize, end: usize) -> super::Token {
        super::Token {
            text: text.to_owned(),
            exclusion,
            operator: false,
            start,
            end,
        }
    }

    #[test]
    fn search_simple_words() {
        assert_eq!(
            super::tokenize("iron sword"),
            vec![token("iron", false, 0, 4), token("sword", false, 5, 10),]
        );
    }

    #[test]
    fn search_quoted_phrase() {
        assert_eq!(
            super::tokenize("\"iron sword\""),
            vec![token("iron sword", false, 0, 12)]
        );
        assert_eq!(
            super::tokenize("\"iron sword"),
            vec![token("iron sword", false, 0, 11)]
        );
    }

    #[test]
    fn search_exclusion() {
        assert_eq!(
            super::tokenize("-diamond"),
            vec![token("diamond", true, 0, 8)]
        );
        assert_eq!(
            super::tokenize("-\"iron sword\""),
            vec![token("iron sword", true, 0, 13)]
        );
    }

    #[test]
    fn search_dash_inside_quotes_is_text() {
        assert_eq!(
            super::tokenize("\"-iron sword\""),
            vec![token("-iron sword", false, 0, 13)]
        );
        assert_eq!(
            super::tokenize("-\"-iron sword\""),
            vec![token("-iron sword", true, 0, 14)]
        );
    }

    #[test]
    fn search_mixed() {
        assert_eq!(
            super::tokenize("sword \"iron pick\" -wood"),
            vec![
                token("sword", false, 0, 5),
                token("iron pick", false, 6, 17),
                token("wood", true, 18, 23),
            ]
        );
    }

    #[test]
    fn search_empty_and_whitespace() {
        assert!(super::tokenize("").is_empty());
        assert!(super::tokenize("   ").is_empty());
        assert!(super::tokenize("-   ").is_empty());
        assert!(super::tokenize(" \"  ").is_empty());
        assert!(super::tokenize("\"\"").is_empty());
    }

    #[test]
    fn search_escaped_quotes() {
        assert_eq!(
            super::tokenize("iron\\\"sword \\\"quoted\\\""),
            vec![
                token("iron\"sword", false, 0, 11),
                token("\"quoted\"", false, 12, 22),
            ]
        );
        assert_eq!(
            super::tokenize("\"the \\\"best\\\" sword\""),
            vec![token("the \"best\" sword", false, 0, 20)]
        );
    }

    #[test]
    fn search_prefixed_quoted_phrases() {
        assert_eq!(
            super::tokenize("%\"redstone blocks\""),
            vec![token("%redstone blocks", false, 0, 18)]
        );
        assert_eq!(
            super::tokenize("-%\"redstone blocks\""),
            vec![token("%redstone blocks", true, 0, 19)]
        );
        assert_eq!(
            super::tokenize("%\"redstone blocks"),
            vec![token("%redstone blocks", false, 0, 17)]
        );
    }

    #[test]
    fn search_trims_inside_quotes() {
        assert_eq!(
            super::tokenize("\"  iron sword  \" -\"  gold ingot  \""),
            vec![
                token("iron sword", false, 0, 16),
                token("gold ingot", true, 17, 34),
            ]
        );
    }

    #[test]
    fn search_whitespace_separates() {
        assert_eq!(
            super::tokenize("iron\t\"gold ingot\"\n-wood\r\n\"red\tstone\""),
            vec![
                token("iron", false, 0, 4),
                token("gold ingot", false, 5, 17),
                token("wood", true, 18, 23),
                token("red\tstone", false, 25, 36),
            ]
        );
    }

    #[test]
    fn search_bare_dash_does_not_exclude() {
        assert_eq!(
            super::tokenize("- diamond - \"iron sword\""),
            vec![
                token("diamond", false, 2, 9),
                token("iron sword", false, 12, 24),
            ]
        );
        assert_eq!(
            super::tokenize("-\"\" diamond -\"  \" gold"),
            vec![token("diamond", false, 4, 11), token("gold", false, 18, 22),]
        );
    }

    #[test]
    fn search_dash_in_a_word_is_not_exclusion() {
        assert_eq!(
            super::tokenize("iron-sword @mod-name %-redstone"),
            vec![
                token("iron-sword", false, 0, 10),
                token("@mod-name", false, 11, 20),
                token("%-redstone", false, 21, 31),
            ]
        );
    }

    #[test]
    fn search_or_operator() {
        assert_eq!(
            super::tokenize("iron|sword"),
            vec![
                token("iron", false, 0, 4),
                super::Token::operator(4),
                token("sword", false, 5, 10),
            ]
        );
        assert_eq!(
            super::tokenize("iron | sword"),
            vec![
                token("iron", false, 0, 4),
                super::Token::operator(5),
                token("sword", false, 7, 12),
            ]
        );
        assert_eq!(
            super::tokenize("\"iron|sword\""),
            vec![token("iron|sword", false, 0, 12)]
        );
    }

    #[test]
    fn search_groups_by_or() {
        let groups = super::split_by_operator(&super::tokenize("iron|gold sword"));
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].len(), 1);
        assert_eq!(groups[0][0].text, "iron");
        assert_eq!(groups[1].len(), 2);
        assert_eq!(groups[1][0].text, "gold");
        assert_eq!(groups[1][1].text, "sword");
    }

    fn stone<'a>(tags: &'a [String]) -> super::SearchItem<'a> {
        super::SearchItem {
            name: "Stone",
            id: "minecraft:stone",
            tags,
        }
    }

    #[test]
    fn search_keeps_by_name_and_id() {
        let tags = ["minecraft:logs".to_owned()];
        let item = stone(&tags);
        assert!(super::keeps(&item, ""));
        assert!(super::keeps(&item, "stone"));
        assert!(super::keeps(&item, "STONE"));
        assert!(super::keeps(&item, "minecraft:sto"));
        assert!(!super::keeps(&item, "diamond"));
        assert!(!super::keeps(&item, "-stone"));
        assert!(super::keeps(&item, "-diamond"));
    }

    #[test]
    fn search_keeps_by_prefix() {
        let tags = ["minecraft:logs".to_owned()];
        let item = stone(&tags);
        assert!(super::keeps(&item, "@minecraft"));
        assert!(!super::keeps(&item, "@other"));
        assert!(super::keeps(&item, "#logs"));
        assert!(!super::keeps(&item, "#stone"));
        // No tooltip or creative-tab data yet: those prefixes match nothing.
        assert!(!super::keeps(&item, "$anything"));
        assert!(!super::keeps(&item, "%anything"));
    }

    #[test]
    fn search_keeps_or_and_phrases() {
        let tags: [String; 0] = [];
        let item = stone(&tags);
        assert!(super::keeps(&item, "iron|stone"));
        assert!(!super::keeps(&item, "iron|gold"));
        assert!(super::keeps(&item, "\"one stone\"|stone"));
        assert!(!super::keeps(&item, "\"one stone\""));
    }
}
