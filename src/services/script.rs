use rhai::module_resolvers::StaticModuleResolver;
use rhai::{Array, Dynamic, Engine, FnPtr, Scope};
use rhai::{EvalAltResult, ImmutableString, LexError, ParseError, ParseErrorType, Position};
use serde_json::json;
use std::collections::HashMap;

pub struct ScriptService {
    engine: Engine,
    module_resolver: StaticModuleResolver,
}

impl ScriptService {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        let module_resolver = StaticModuleResolver::new();

        // Configure engine for BASIC-like syntax
        engine.set_allow_anonymous_fn(true);
        engine.set_allow_looping(true);

        use rhai::{Array, Dynamic, Engine, Scope};

engine
    .register_custom_syntax(
        &[
            "FOR", "EACH", "$ident$", "IN", "$expr$", "$block$", "NEXT", "$ident$",
        ],
        true,  // We're modifying the scope by adding the loop variable
        |context, inputs| {
            // Get the iterator variable names
            let loop_var = inputs[0].get_string_value().unwrap();
            let next_var = inputs[3].get_string_value().unwrap();

            // Verify variable names match
            if loop_var != next_var {
                return Err(format!(
                    "NEXT variable '{}' doesn't match FOR EACH variable '{}'",
                    next_var, loop_var
                )
                .into());
            }

            // Evaluate the collection expression
            let collection = context.eval_expression_tree(&inputs[1])?;

            // Get the block as an expression tree
            let block = &inputs[2];

            // Convert to array
            let array: Array = if collection.is_array() {
                collection.cast()
            } else {
                return Err("FOR EACH can only iterate over arrays".into());
            };

            // Remember original scope length
            let orig_len = context.scope().len();

            for item in array {
                // Push the loop variable into the scope
                context.scope_mut().push(loop_var.to_string(), item);

                // Evaluate the block with the current scope
                match context.eval_expression_tree(block) {
                    Ok(_) => (),
                    Err(e) if e.to_string() == "EXIT FOR" => break,
                    Err(e) => {
                        // Rewind the scope before returning error
                        context.scope_mut().rewind(orig_len);
                        return Err(e);
                    }
                }

                // Remove the loop variable for next iteration
                context.scope_mut().rewind(orig_len);
            }

            Ok(Dynamic::UNIT)
        },
    )
    .unwrap();

        // Register EXIT FOR
        engine
            .register_custom_syntax(&["EXIT", "FOR"], false, |_context, _inputs| {
                Err("EXIT FOR".into())
            })
            .unwrap();

        // FIND command: FIND "table", "filter"
        engine
            .register_custom_syntax(
                &["FIND", "$expr$", ",", "$expr$"],
                false, // Expression, not statement
                |context, inputs| {
                    let table_name = context.eval_expression_tree(&inputs[0])?;
                    let filter = context.eval_expression_tree(&inputs[1])?;

                    let table_str = table_name.to_string();
                    let filter_str = filter.to_string();

                    let result = json!({
                        "command": "find",
                        "table": table_str,
                        "filter": filter_str,
                        "results": []
                    });
                    println!("SET executed: {}", result.to_string());
                    Ok(Dynamic::from(result.to_string()))
                },
            )
            .unwrap();

        // SET command: SET "table", "key", "value"
        engine
            .register_custom_syntax(
                &["SET", "$expr$", ",", "$expr$", ",", "$expr$"],
                true, // Statement
                |context, inputs| {
                    let table_name = context.eval_expression_tree(&inputs[0])?;
                    let key_value = context.eval_expression_tree(&inputs[1])?;
                    let value = context.eval_expression_tree(&inputs[2])?;

                    let table_str = table_name.to_string();
                    let key_str = key_value.to_string();
                    let value_str = value.to_string();

                    let result = json!({
                        "command": "set",
                        "status": "success",
                        "table": table_str,
                        "key": key_str,
                        "value": value_str
                    });
                    println!("SET executed: {}", result.to_string());
                    Ok(Dynamic::UNIT)
                },
            )
            .unwrap();

        // GET command: GET "url"
        engine
            .register_custom_syntax(
                &["GET", "$expr$"],
                false, // Expression, not statement
                |context, inputs| {
                    let url = context.eval_expression_tree(&inputs[0])?;
                    let url_str = url.to_string();

                    println!("GET executed: {}", url_str.to_string());
                    Ok(format!("Content from {}", url_str).into())
                },
            )
            .unwrap();

        // CREATE SITE command: CREATE SITE "name", "company", "website", "template", "prompt"
        engine
            .register_custom_syntax(
                &[
                    "CREATE", "SITE", "$expr$", ",", "$expr$", ",", "$expr$", ",", "$expr$", ",",
                    "$expr$",
                ],
                true, // Statement
                |context, inputs| {
                    if inputs.len() < 5 {
                        return Err("Not enough arguments for CREATE SITE".into());
                    }

                    let name = context.eval_expression_tree(&inputs[0])?;
                    let company = context.eval_expression_tree(&inputs[1])?;
                    let website = context.eval_expression_tree(&inputs[2])?;
                    let template = context.eval_expression_tree(&inputs[3])?;
                    let prompt = context.eval_expression_tree(&inputs[4])?;

                    let result = json!({
                        "command": "create_site",
                        "name": name.to_string(),
                        "company": company.to_string(),
                        "website": website.to_string(),
                        "template": template.to_string(),
                        "prompt": prompt.to_string()
                    });
                    println!("CREATE SITE executed: {}", result.to_string());
                    Ok(Dynamic::UNIT)
                },
            )
            .unwrap();

        // CREATE DRAFT command: CREATE DRAFT "to", "subject", "body"
        engine
            .register_custom_syntax(
                &["CREATE", "DRAFT", "$expr$", ",", "$expr$", ",", "$expr$"],
                true, // Statement
                |context, inputs| {
                    if inputs.len() < 3 {
                        return Err("Not enough arguments for CREATE DRAFT".into());
                    }

                    let to = context.eval_expression_tree(&inputs[0])?;
                    let subject = context.eval_expression_tree(&inputs[1])?;
                    let body = context.eval_expression_tree(&inputs[2])?;

                    let result = json!({
                        "command": "create_draft",
                        "to": to.to_string(),
                        "subject": subject.to_string(),
                        "body": body.to_string()
                    });
                    println!("CREATE DRAFT executed: {}", result.to_string());
                    Ok(Dynamic::UNIT)
                },
            )
            .unwrap();

        // PRINT command
        engine
            .register_custom_syntax(
                &["PRINT", "$expr$"],
                true, // Statement
                |context, inputs| {
                    let value = context.eval_expression_tree(&inputs[0])?;
                    println!("{}", value);
                    Ok(Dynamic::UNIT)
                },
            )
            .unwrap();

        // Register web service functions
        engine.register_fn("web_get", |url: &str| format!("Response from {}", url));

        ScriptService {
            engine,
            module_resolver,
        }
    }

    fn preprocess_basic_script(&self, script: &str) -> String {
        let mut result = String::new();
        let mut for_stack: Vec<usize> = Vec::new();
        let mut current_indent = 0;

        for line in script.lines() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("REM") {
                result.push_str(line);
                result.push('\n');
                continue;
            }

            // Handle FOR EACH start
            if trimmed.starts_with("FOR EACH") {
                for_stack.push(current_indent);
                result.push_str(&" ".repeat(current_indent));
                result.push_str(trimmed);
                result.push_str("{\n");
                current_indent += 4;
                result.push_str(&" ".repeat(current_indent));
                result.push('\n');
                continue;
            }

            // Handle NEXT
            if trimmed.starts_with("NEXT") {
                if let Some(expected_indent) = for_stack.pop() {
                    if (current_indent - 4) != expected_indent {
                        panic!("NEXT without matching FOR EACH");
                    }
                    current_indent = current_indent - 4;
                    result.push_str(&" ".repeat(current_indent));
                    result.push_str("}\n");
                    result.push_str(&" ".repeat(current_indent));
                    result.push_str(trimmed);
                    result.push(';'); 
                    result.push('\n'); 
                    continue;
                } else {
                    panic!("NEXT without matching FOR EACH");
                }
            }

            // Handle EXIT FOR
            if trimmed == "EXIT FOR" {
                result.push_str(&" ".repeat(current_indent));
                result.push_str(trimmed);
                result.push('\n');
                continue;
            }

            // Handle regular lines - no semicolons added for BASIC-style commands
            result.push_str(&" ".repeat(current_indent));

            let basic_commands = [
                "SET", "CREATE", "PRINT", "FOR", "FIND", "GET", "EXIT", "IF", "THEN", "ELSE",
                "END IF", "WHILE", "WEND", "DO", "LOOP",
            ];

            let is_basic_command = basic_commands.iter().any(|&cmd| trimmed.starts_with(cmd));
            let is_control_flow = trimmed.starts_with("IF")
                || trimmed.starts_with("ELSE")
                || trimmed.starts_with("END IF");

            if is_basic_command || !for_stack.is_empty() || is_control_flow {
                // Don'ta add semicolons for BASIC-style commands or inside blocks
                result.push_str(trimmed);
            } else {
                // Add semicolons only for non-BASIC statements
                result.push_str(trimmed);
                if !trimmed.ends_with(';') && !trimmed.ends_with('{') && !trimmed.ends_with('}') {
                    result.push(';');
                }
            }
            result.push('\n');
        }

        if !for_stack.is_empty() {
            panic!("Unclosed FOR EACH loop");
        }

        result
    }
    
    /// Preprocesses BASIC-style script to handle semicolon-free syntax
    pub fn compile(&self, script: &str) -> Result<rhai::AST, Box<EvalAltResult>> {
        let processed_script = self.preprocess_basic_script(script);
        match self.engine.compile(&processed_script) {
            Ok(ast) => Ok(ast),
            Err(parse_error) => Err(Box::new(EvalAltResult::from(parse_error))),
        }
    }

    pub fn run(&self, ast: &rhai::AST) -> Result<Dynamic, Box<EvalAltResult>> {
        self.engine.eval_ast(ast)
    }

    pub fn call_web_service(
        &self,
        endpoint: &str,
        data: HashMap<String, String>,
    ) -> Result<String, Box<EvalAltResult>> {
        Ok(format!("Called {} with {:?}", endpoint, data))
    }

    /// Execute a BASIC-style script without semicolons
    pub fn execute_basic_script(&self, script: &str) -> Result<Dynamic, Box<EvalAltResult>> {
        let processed = self.preprocess_basic_script(script);
        let ast = self.engine.compile(&processed)?;
        self.run(&ast)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_script_without_semicolons() {
        let service = ScriptService::new();

        // Test BASIC-style script without semicolons
        let script = r#"
json = FIND "users", "name=John"
SET "users", "name=John", "age=30"
text = GET "example.com"
CREATE SITE "mysite", "My Company", "mycompany.com", "basic", "Create a professional site"
CREATE DRAFT "client@example.com", "Project Update", "Here's the latest update..."
PRINT "Script completed successfully"
        "#;

        let result = service.execute_basic_script(script);
        assert!(result.is_ok());
    }

    #[test]
    fn test_preprocessing() {
        let service = ScriptService::new();

        let script = r#"
json = FIND "users", "name=John"
SET "users", "name=John", "age=30"
let x = 42
PRINT x
if x > 10 {
    PRINT "Large number"
}
        "#;

        let processed = service.preprocess_basic_script(script);

        // Should add semicolons to regular statements but not custom commands
        assert!(processed.contains("let x = 42;"));
        assert!(processed.contains("json = FIND"));
        assert!(!processed.contains("SET \"users\""));
        assert!(!processed.contains("PRINT \"Large number\";")); // Inside block shouldn't get semicolon
    }

    #[test]
    fn test_individual_commands() {
        let service = ScriptService::new();

        let commands = vec![
            r#"SET "users", "name=John", "age=30""#,
            r#"CREATE SITE "mysite", "My Company", "mycompany.com", "basic", "Create a professional site""#,
            r#"CREATE DRAFT "client@example.com", "Project Update", "Here's the latest update...""#,
            r#"PRINT "Hello, World!""#,
        ];

        for cmd in commands {
            let result = service.execute_basic_script(cmd);
            assert!(result.is_ok(), "Command '{}' failed", cmd);
        }
    }

    #[test]
    fn test_block_statements() {
        let service = ScriptService::new();

        let script = r#"
if true {
    PRINT "Inside block"
    PRINT "Another statement"
}
        "#;

        let result = service.execute_basic_script(script);
        assert!(result.is_ok());
    }
}
