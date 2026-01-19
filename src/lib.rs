use crate::rline::ShellHelper;
use rustyline::config::BellStyle;
use rustyline::history::DefaultHistory;
use rustyline::{Config, CompletionType, EditMode};

pub mod parse;
pub mod rline;
pub mod builtins;
pub mod executables;

pub fn create_editor(h: ShellHelper) -> rustyline::Result<rustyline::Editor<ShellHelper, DefaultHistory>> {
    let config = Config::builder()
        .completion_type(CompletionType::List) // default: Emacs-style, cycles through candidates
        .bell_style(BellStyle::Audible)
        .edit_mode(EditMode::Emacs) // e.g. Ctrl-A, Home - Move cursor to the beginning of line
        .max_history_size(1000)
        .unwrap()
        .completion_prompt_limit(200) // trigger alert when completion is too ambiguous
        .build();

    let mut rl = rustyline::Editor::<ShellHelper, DefaultHistory>::with_config(config)?;
    rl.set_helper(Some(h));
    Ok(rl)
}
