use anyhow::Result;
use shlib::state::Terminal;

fn main() -> Result<()> {
    let mut term = Terminal::new()?;
    term.start()
}
