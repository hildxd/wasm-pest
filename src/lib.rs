use pest::error::{Error, ErrorVariant, InputLocation};
use pest::iterators::Pair;
use pest_meta::parser::{self, Rule};
use pest_meta::{optimizer, validator};
use pest_vm::Vm;
use std::collections::HashMap;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsValue, UnwrapThrowExt};

static mut VM: Option<Vm> = None;

#[wasm_bindgen]
pub fn compile_grammer(grammar: String) -> Result<bool, JsValue> {
    let result = parser::parse(Rule::grammar_rules, &grammar)
        .map_err(|error| error.renamed_rules(pest_meta::parser::rename_meta_rule));

    let pairs = match result {
        Ok(pairs) => pairs,
        // Err(error) => return vec![convert_error(error, &grammar)],
        Err(error) => return Err(JsValue::from_bool(false)),
    };

    if let Err(errors) = validator::validate_pairs(pairs.clone()) {
        // return errors
        //     .into_iter()
        //     .map(|error| convert_error(error, &grammar))
        //     .collect();
        return Err(JsValue::from_bool(false));
    }

    let ast = match parser::consume_rules(pairs) {
        Ok(ast) => ast,
        Err(errors) => {
            // return errors
            //     .into_iter()
            //     .map(|error| convert_error(error, &grammar))
            //     .collect();
            return Err(JsValue::from_bool(false));
        }
    };

    unsafe {
        VM = Some(Vm::new(optimizer::optimize(ast.clone())));
    }

    // vec![]
    Ok(true)
}

fn convert_error(error: Error<Rule>, grammar: &str) -> HashMap<String, String> {
    let message = match error.variant {
        ErrorVariant::CustomError { message } => message,
        _ => unreachable!(),
    };

    match error.location {
        InputLocation::Pos(pos) => {
            let mut map = HashMap::new();

            map.insert("from".to_owned(), line_col(pos, grammar));
            map.insert("to".to_owned(), line_col(pos, grammar));
            map.insert("message".to_owned(), format!("{}", message));

            map
        }
        InputLocation::Span((start, end)) => {
            let mut map = HashMap::new();

            map.insert("from".to_owned(), line_col(start, grammar));
            map.insert("to".to_owned(), line_col(end, grammar));
            map.insert("message".to_owned(), format!("{}", message));

            map
        }
    }
}

fn line_col(pos: usize, input: &str) -> String {
    let (line, col) = {
        let mut pos = pos;
        // Position's pos is always a UTF-8 border.
        let slice = &input[..pos];
        let mut chars = slice.chars().peekable();

        let mut line_col = (1, 1);

        while pos != 0 {
            match chars.next() {
                Some('\r') => {
                    if let Some(&'\n') = chars.peek() {
                        chars.next();

                        if pos == 1 {
                            pos -= 1;
                        } else {
                            pos -= 2;
                        }

                        line_col = (line_col.0 + 1, 1);
                    } else {
                        pos -= 1;
                        line_col = (line_col.0, line_col.1 + 1);
                    }
                }
                Some('\n') => {
                    pos -= 1;
                    line_col = (line_col.0 + 1, 1);
                }
                Some(c) => {
                    pos -= c.len_utf8();
                    line_col = (line_col.0, line_col.1 + 1);
                }
                None => unreachable!(),
            }
        }

        line_col
    };

    format!("({}, {})", line - 1, col - 1)
}

#[wasm_bindgen]
pub fn parse_input(rule: &str, input: &str) -> String {
    let vm = unsafe { VM.as_ref().expect_throw("no VM") };

    match vm.parse(rule, input) {
        Ok(pairs) => {
            let lines: Vec<_> = pairs.map(|pair| format_pair(pair, 0, true)).collect();
            let lines = lines.join("\n");
            lines
        }
        Err(error) => format!("{}", error.renamed_rules(|r| r.to_string())),
    }
}

fn format_pair(pair: Pair<&str>, indent_level: usize, is_newline: bool) -> String {
    let indent = if is_newline {
        "  ".repeat(indent_level)
    } else {
        "".to_string()
    };

    let children: Vec<_> = pair.clone().into_inner().collect();
    let len = children.len();
    let children: Vec<_> = children
        .into_iter()
        .map(|pair| {
            format_pair(
                pair,
                if len > 1 {
                    indent_level + 1
                } else {
                    indent_level
                },
                len > 1,
            )
        })
        .collect();

    let dash = if is_newline { "- " } else { "" };

    match len {
        0 => format!(
            "{}{}{}: {:?}",
            indent,
            dash,
            pair.as_rule(),
            pair.as_span().as_str()
        ),
        1 => format!("{}{}{} > {}", indent, dash, pair.as_rule(), children[0]),
        _ => format!(
            "{}{}{}\n{}",
            indent,
            dash,
            pair.as_rule(),
            children.join("\n")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let grammar = r#"
            alpha = { 'a'..'z' | 'A'..'Z' }
        "#;

        let errors = compile_grammer(grammar.to_string());
        assert_eq!(errors.unwrap(), true);
        // assert_eq!(errors., 0);

        let input = "ab";
        let output = parse_input("alpha", input);
        assert_eq!(output, "- alpha: \"a\"");
    }
}
