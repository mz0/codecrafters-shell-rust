#[derive(Debug, PartialEq)]
pub enum Command {
    SimpleCommand(String, Vec<String>),
    PipeCommand(Vec<Command>),
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

    let mut commands = Vec::new();
    for part in parts {
        let cmd = parse_simple(&part);
        if let Command::InvalidCommand(_) = cmd {
            return cmd;
        }
        commands.push(cmd);
    }

    if commands.len() == 1 {
        commands.pop().unwrap()
    } else {
        Command::PipeCommand(commands)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_command() {
        let input = "ls -la";
        let expected = Command::SimpleCommand("ls".to_string(), vec!["-la".to_string()]);
        assert_eq!(parse(input), expected);
    }

    #[test]
    fn test_quoted_arguments() {
        let input = "echo 'hello world' \"foo bar\"";
        let expected = Command::SimpleCommand(
            "echo".to_string(),
            vec!["hello world".to_string(), "foo bar".to_string()],
        );
        assert_eq!(parse(input), expected);
    }

    #[test]
    fn test_mixed_quotes() {
        let input = "echo \"it's me\" 'said \"hello\"'";
        let expected = Command::SimpleCommand(
            "echo".to_string(),
            vec!["it's me".to_string(), "said \"hello\"".to_string()],
        );
        assert_eq!(parse(input), expected);
    }

    #[test]
    fn test_escaped_characters() {
        let input = "echo hello\\ world";
        let expected = Command::SimpleCommand(
            "echo".to_string(),
            vec!["hello world".to_string()],
        );
        assert_eq!(parse(input), expected);
    }

    #[test]
    fn test_escaped_quotes() {
        let input = "echo \\\"hello\\\"";
        let expected = Command::SimpleCommand(
            "echo".to_string(),
            vec!["\"hello\"".to_string()],
        );
        assert_eq!(parse(input), expected);
    }

    #[test]
    fn test_pipe_command() {
        let input = "cat file.txt | grep pattern";
        let expected = Command::PipeCommand(vec![
            Command::SimpleCommand(
                "cat".to_string(),
                vec!["file.txt".to_string()],
            ),
            Command::SimpleCommand(
                "grep".to_string(),
                vec!["pattern".to_string()],
            ),
        ]);
        assert_eq!(parse(input), expected);
    }

    #[test]
    fn test_multiple_pipes() {
        let input = "cat file | grep foo | wc -l";
        let expected = Command::PipeCommand(vec![
            Command::SimpleCommand(
                "cat".to_string(),
                vec!["file".to_string()],
            ),
            Command::SimpleCommand(
                "grep".to_string(),
                vec!["foo".to_string()],
            ),
            Command::SimpleCommand(
                "wc".to_string(),
                vec!["-l".to_string()],
            ),
        ]);
        assert_eq!(parse(input), expected);
    }

    #[test]
    fn test_pipe_with_quotes() {
        let input = "echo 'foo | bar' | cat";
        let expected = Command::PipeCommand(vec![
            Command::SimpleCommand(
                "echo".to_string(),
                vec!["foo | bar".to_string()],
            ),
            Command::SimpleCommand("cat".to_string(), vec![]),
        ]);
        assert_eq!(parse(input), expected);
    }

    #[test]
    fn test_unpaired_quote() {
        let input = "echo \"hello";
        match parse(input) {
            Command::InvalidCommand(msg) => assert_eq!(msg, "Unpaired quote"),
            _ => panic!("Expected InvalidCommand"),
        }
    }

    #[test]
    fn test_trailing_backslash() {
        let input = "echo hello\\";
        match parse(input) {
            Command::InvalidCommand(msg) => assert_eq!(msg, "Trailing backslash"),
            _ => panic!("Expected InvalidCommand"),
        }
    }

    #[test]
    fn test_empty_command() {
        let input = "";
        match parse(input) {
            Command::InvalidCommand(msg) => assert_eq!(msg, "Empty command"),
            _ => panic!("Expected InvalidCommand"),
        }
    }

    #[test]
    fn test_whitespace_only() {
        let input = "   ";
        match parse(input) {
            Command::InvalidCommand(msg) => assert_eq!(msg, "Empty command"),
            _ => panic!("Expected InvalidCommand"),
        }
    }
}
