// /jei says what this build is; /jei debug turns debug on or off.
fn jei_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    use crate::game::jei as m;
    let Some(args) = command.strip_prefix("jei") else {
        return;
    };
    if !(args.is_empty() || args.starts_with(' ')) {
        return;
    }
    let Some(cmd) = m::command(args) else {
        *ran = true;
        game.native_chat("Unknown option: use /jei or /jei debug");
        return;
    };
    *ran = true;
    match cmd {
        m::Command::About => game.native_chat(&m::about()),
        m::Command::ToggleDebug => {
            let on = m::debug(game.native_setting(m::ID, "debug").as_deref());
            game.native_set_setting(m::ID, "debug", if on { "false" } else { "true" });
            game.native_chat(&m::debug_button(!on));
        }
    }
}
