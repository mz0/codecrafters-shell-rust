
This is a starting point for Rust solutions to the
["Build Your Own Shell" Challenge](https://app.codecrafters.io/courses/shell/overview).

In this challenge, you'll build your own POSIX compliant shell that's capable of
interpreting shell commands, running external programs and builtin commands like
cd, pwd, echo and more. Along the way, you'll learn about shell command parsing,
REPLs, builtin commands, and more.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to try the challenge.

## See also
* [aaron-ang (2024-05-24..2025-06-04) + **all**](https://github.com/aaron-ang/shell-rust)
  * `command.rs`  `history.rs`  `main.rs`  `pipeline.rs`  `state.rs`  `token.rs`
  * `Cargo.toml` (edition = "2021") - `os_pipe 1.2.1`, `termion 4.0.3`, `strum = { version = "0.26.3", features = ["derive"] }`;
* [mariamikv (2025-06-10) + quotes, redirection](https://github.com/mariamikv/codecrafters-shell-rust)
  * `main.rs`, `commands.rs` - funny modularity; (?? potentially mem leaking) 'k _lifetime parameter_ use
  * `Cargo.toml` (edition = "2021") - unchanged
* [kov (2024-08-28..2025-05-23) + redirection, pipe (**both need ' '**), history](https://github.com/kov/codecrafters-shell-rust)
  * [main.rs](https://github.com/kov/codecrafters-shell-rust/blob/master/src/main.rs) - 696 lines, (??) 'r _lifetime parameter_ use
  * `Cargo.toml` (edition = "2021") + `dirs 5.0.1`, `os_pipe 1.2.1`, `rustyline 14.0`, `home =0.5.9`, `lazy_static 1.5.0`;
* [Germainch (2024-12-09) + quotes](https://github.com/Germainch/rust-shell)
  * `main.rs` + `lib/functions/{cd, echo, exit, invalid_command, pwd,type_cmd, mod}.rs` - 15 warnings
  * `Cargo.toml` (edition = "2021") + `regex 1.11.1`;
* [gooplancton (2024-07-06) - only Basic (up to step 12: cd ~)](https://github.com/gooplancton/codecrafters-shell-rust/blob/master/src/main.rs)
  * 3 files: `main.rs`, `commands.rs`, `path.rs`
  * `Cargo.toml` (edition = "2021") - unchanged

