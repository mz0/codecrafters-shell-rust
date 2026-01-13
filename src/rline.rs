use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

pub struct ShellHelper {
    pub builtins: Vec<&'static str>,
    pub system_commands: Vec<String>,
}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) 
        -> Result<(usize, Vec<Pair>), ReadlineError> 
    {
        let sub = &line[..pos];
        let start = sub.rfind(' ').map_or(0, |i| i + 1);
        let word = &sub[start..];

        let builtin_matches = self.builtins.iter()
            .filter(|c| c.starts_with(word))
            .map(|&c| c.to_string());

        let system_matches = self.system_commands.iter()
            .filter(|c| c.starts_with(word))
            .cloned();

        let candidates: Vec<Pair> = builtin_matches
            .chain(system_matches)
            .map(|cmd| Pair {
                display: cmd.clone(),
                replacement: format!("{} ", cmd),
            })
            .collect();

        Ok((start, candidates))
    }
}

// empty Traits, required by rustyline
impl Hinter for ShellHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None // No "ghost text" hints
    }
}

impl Highlighter for ShellHelper {} // No syntax highlighting
impl Validator for ShellHelper {}   // No input validation (Enter always submits)
impl Helper for ShellHelper {}      // Ties everything together
