use crate::runtime::{Runtime, apply_adverb, apply_dyadic_verb, apply_monadic_verb};
use crate::token::{Token, adverb_index, tokenize_line, verb_index};
use crate::value::Value;
use std::io::{self, BufRead, Write};

/// Evaluate an expression.
fn evaluate_expression(runtime: &mut Runtime, tokens: &[Token]) -> Value {
    match tokens {
        [] => runtime.parse_error("evaluate_expression"),
        [token] => match runtime.noun_from_token(token) {
            Value::Error => runtime.parse_error("evaluate_error"),
            value @ _ => value,
        },
        [Token::Symbol(verb), Token::Symbol(adverb), rest @ ..]
            if verb_index(*verb) != 0 && adverb_index(*adverb) != 0 =>
        {
            let verb_idx = verb_index(*verb);
            let adverb_idx = adverb_index(*adverb);
            let operand = evaluate_expression(runtime, rest);
            if operand.is_error() {
                return operand;
            }
            apply_adverb(runtime, adverb_idx, verb_idx, operand)
        }
        [Token::Symbol(verb), rest @ ..] if verb_index(*verb) != 0 => {
            let verb_idx = verb_index(*verb);
            let operand = evaluate_expression(runtime, rest);
            if operand.is_error() {
                return operand;
            }
            apply_monadic_verb(runtime, verb_idx, operand)
        }
        [Token::Global(name), Token::Colon, rest @ ..] => {
            let right_value = evaluate_expression(runtime, rest);
            if right_value.is_error() {
                return right_value;
            }
            let index = (name - b'a') as usize;
            runtime.assign_global(index, right_value)
        }
        [left_token, Token::Symbol(op), rest @ ..] => {
            let left_value = runtime.noun_from_token(left_token);
            if left_value.is_error() {
                return runtime.parse_error("evaluate_expression");
            }

            let right_value = evaluate_expression(runtime, rest);
            if right_value.is_error() {
                return right_value;
            }

            let dyadic_idx = verb_index(*op);
            if dyadic_idx == 0 {
                return runtime.domain_error("evaluate_expression");
            }

            apply_dyadic_verb(runtime, dyadic_idx, left_value, right_value)
        }
        _ => runtime.parse_error("evaluate_expression"),
    }
}

/// Process a line of k/simple code.
fn process_line(runtime: &mut Runtime, line: &str) -> bool {
    let trimmed = line.trim_end();

    if trimmed.is_empty() {
        return true;
    }

    let bytes = trimmed.as_bytes();

    // Special commands:
    // \\ - quit
    // \w - memory allocation in workspace (in bytes, by vectors only)
    // \v - list global variables (vectors only)
    if bytes.len() == 2 && bytes[0] == b'\\' {
        match bytes[1] {
            b'\\' => return false,
            b'w' => println!("{}", runtime.workspace_bytes()),
            b'v' => print!("{}", runtime),
            _ => {}
        }
        return true;
    }

    // Comments start with a slash.
    if bytes[0] == b'/' {
        return true;
    }

    // Tokenize the line.
    let tokens = match tokenize_line(trimmed) {
        Ok(tokens) => tokens,
        Err(_) => {
            runtime.parse_error("tokenize_line");
            return true;
        }
    };

    // Nothing to do.
    if tokens.is_empty() {
        return true;
    }

    let result = evaluate_expression(runtime, &tokens);

    // Assignment.
    if tokens.len() > 1 && matches!(tokens[1], Token::Colon) {
        return true;
    }

    println!("{}", result);

    true
}

/// Run a REPL.
pub fn run_repl(runtime: &mut Runtime) {
    let mut input = String::new();

    loop {
        print!("k)");

        let _ = io::stdout().flush();
        input.clear();

        if io::stdin().read_line(&mut input).unwrap_or(0) == 0 {
            break;
        }
        if !process_line(runtime, &input) {
            break;
        }
    }
}

/// Run a file containing k code.
pub fn run_batch(runtime: &mut Runtime, path: &str) {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(_) => {
            runtime.report_error("read_line", path);
            return;
        }
    };

    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        match line {
            Ok(line) => {
                if !process_line(runtime, &line) {
                    break;
                }
            }
            Err(_) => {
                runtime.report_error("read_line", path);
                break;
            }
        }
    }
}
