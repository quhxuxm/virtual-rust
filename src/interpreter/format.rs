//! Format string processing (`{}`, `{:?}`, `{:.2}`, `{0}`, etc.).

use crate::interpreter::error::RuntimeError;
use crate::interpreter::value::Value;

/// Processes a Rust-style format string with positional arguments.
///
/// Supported placeholders:
/// - `{}` — display the next argument
/// - `{0}`, `{1}` — indexed arguments
/// - `{:?}` — debug format
/// - `{:.N}` — float precision
/// - `{{` / `}}` — escaped braces
pub fn format_string(fmt: &str, args: &[Value]) -> Result<String, RuntimeError> {
    let mut result = String::new();
    let mut arg_index = 0;
    let mut chars = fmt.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '{' => format_placeholder(&mut chars, &mut result, args, &mut arg_index),
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                result.push('}');
            }
            _ => result.push(c),
        }
    }

    Ok(result)
}

/// Parses and formats a single `{...}` placeholder.
fn format_placeholder(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    result: &mut String,
    args: &[Value],
    arg_index: &mut usize,
) {
    if chars.peek() == Some(&'{') {
        // Escaped brace: {{
        chars.next();
        result.push('{');
        return;
    }

    if chars.peek() == Some(&'}') {
        // Simple `{}` — use next positional argument
        chars.next();
        if *arg_index < args.len() {
            result.push_str(&format!("{}", args[*arg_index]));
            *arg_index += 1;
        } else {
            result.push_str("{}");
        }
        return;
    }

    // Collect the content between { and }
    let mut placeholder = String::new();
    while let Some(&c) = chars.peek() {
        if c == '}' {
            chars.next();
            break;
        }
        placeholder.push(c);
        chars.next();
    }

    if let Some(spec) = placeholder.strip_prefix(':') {
        // Format specifier: {:?}, {:.2}, etc.
        format_with_spec(spec, result, args, arg_index);
    } else if let Ok(idx) = placeholder.parse::<usize>() {
        // Indexed: {0}, {1}, etc.
        if idx < args.len() {
            result.push_str(&format!("{}", args[idx]));
        }
    } else {
        // Named parameter — fall back to positional
        if *arg_index < args.len() {
            result.push_str(&format!("{}", args[*arg_index]));
            *arg_index += 1;
        } else {
            result.push('{');
            result.push_str(&placeholder);
            result.push('}');
        }
    }
}

/// Applies a format specifier (the part after `:` inside `{:...}`).
fn format_with_spec(spec: &str, result: &mut String, args: &[Value], arg_index: &mut usize) {
    if *arg_index >= args.len() {
        return;
    }

    let arg = &args[*arg_index];
    *arg_index += 1;

    if spec == "?" {
        // Debug format
        result.push_str(&arg.debug_fmt());
    } else if let Some(prec_str) = spec.strip_prefix('.') {
        // Precision format: {:.2}
        if let Ok(precision) = prec_str.parse::<usize>() {
            if let Value::Float(n) = arg {
                result.push_str(&format!("{n:.prec$}", prec = precision));
            } else {
                result.push_str(&format!("{arg}"));
            }
        } else {
            result.push_str(&format!("{arg}"));
        }
    } else {
        result.push_str(&format!("{arg}"));
    }
}
