#[derive(Debug, PartialEq)]
pub enum Command {
    SimpleCommand(String, Vec<String>),
    PipeCommand(Box<Command>, Box<Command>),
    InvalidCommand(String),
}

pub fn parse(s: &str) -> Command {
    let s = s.trim();
    if s.is_empty() {
        return Command::InvalidCommand("Empty command".to_string());
    }

    let parts = match split_by_pipe(s) {
        Ok(p) => p,
        Err(e) => return Command::InvalidCommand(e),
    };

    if parts.is_empty() {
        return Command::InvalidCommand("Empty command".to_string());
    }

    let mut iter = parts.into_iter();
    let first = iter.next().unwrap();
    let mut cmd = parse_simple(&first);

    // If the first part is invalid, return it immediately
    if let Command::InvalidCommand(_) = cmd {
        return cmd;
    }

    for part in iter {
        let next_cmd = parse_simple(&part);
        if let Command::InvalidCommand(_) = next_cmd {
            return next_cmd;
        }
        cmd = Command::PipeCommand(Box::new(cmd), Box::new(next_cmd));
    }

    cmd
}

fn parse_simple(s: &str) -> Command {
    let tokens = match tokenize(s) {
        Ok(t) => t,
        Err(e) => return Command::InvalidCommand(e),
    };

    if tokens.is_empty() {
        // This might happen if s was just whitespace
        return Command::InvalidCommand("Empty command".to_string());
    }

    let cmd = tokens[0].clone();
    let args = tokens[1..].to_vec();
    Command::SimpleCommand(cmd, args)
}

fn split_by_pipe(s: &str) -> Result<Vec<String>, String> {
    let mut parts = Vec::new();
    let mut current_part = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    for c in s.chars() {
        if escaped {
            current_part.push(c);
            escaped = false;
            continue;
        }

        match c {
            '\\' => {
                escaped = true;
                current_part.push(c);
            }
            '\'' => {
                if !in_double_quote {
                    in_single_quote = !in_single_quote;
                }
                current_part.push(c);
            }
            '"' => {
                if !in_single_quote {
                    in_double_quote = !in_double_quote;
                }
                current_part.push(c);
            }
            '|' => {
                if !in_single_quote && !in_double_quote {
                    parts.push(current_part);
                    current_part = String::new();
                } else {
                    current_part.push(c);
                }
            }
            _ => {
                current_part.push(c);
            }
        }
    }

    if escaped {
        return Err("Trailing backslash".to_string());
    }
    if in_single_quote || in_double_quote {
        return Err("Unpaired quote".to_string());
    }

    parts.push(current_part);
    Ok(parts)
}

fn tokenize(s: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    for c in s.chars() {
        if escaped {
            current_token.push(c);
            escaped = false;
            continue;
        }

        match c {
            '\\' => {
                if in_single_quote {
                    current_token.push(c);
                } else {
                    escaped = true;
                }
            }
            '\'' => {
                if in_double_quote {
                    current_token.push(c);
                } else {
                    in_single_quote = !in_single_quote;
                }
            }
            '"' => {
                if in_single_quote {
                    current_token.push(c);
                } else {
                    in_double_quote = !in_double_quote;
                }
            }
            ' ' | '\t' | '\n' | '\r' => {
                if in_single_quote || in_double_quote {
                    current_token.push(c);
                } else if !current_token.is_empty() {
                    tokens.push(current_token);
                    current_token = String::new();
                }
            }
            _ => {
                current_token.push(c);
            }
        }
    }

    if escaped {
        return Err("Trailing backslash".to_string());
    }
    if in_single_quote || in_double_quote {
        return Err("Unpaired quote".to_string());
    }

    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    Ok(tokens)
}
