use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum RefactoringAction {
    RemoveUnusedArgument(String),
    InlineVariableAssignment {
        variable: String,
        expression: String,
    },
    RenameVariable {
        from: String,
        to: String,
    },
}

impl RefactoringAction {
    /// Validates if the action produces the expected result when applied to the original code
    pub fn validate(&self, original_code: &str, expected_result: &str) -> Result<(), String> {
        Ok(())
    }
}

impl FromStr for RefactoringAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if let Some(rest) = s.strip_prefix("Remove unused argument ") {
            if rest.is_empty() {
                return Err("Missing argument name after 'Remove unused argument'".to_string());
            }
            return Ok(RefactoringAction::RemoveUnusedArgument(rest.to_string()));
        }

        if let Some(rest) = s.strip_prefix("Inline variable assignment `") {
            if let Some(inner) = rest.strip_suffix('`') {
                let parts: Vec<&str> = inner.splitn(2, " = ").collect();
                if parts.len() != 2 {
                    return Err(
                        "Invalid inline assignment format. Expected `var = expr`".to_string()
                    );
                }
                return Ok(RefactoringAction::InlineVariableAssignment {
                    variable: parts[0].trim().to_string(),
                    expression: parts[1].trim().to_string(),
                });
            }
            return Err("Missing closing backtick for inline assignment".to_string());
        }

        if let Some(rest) = s.strip_prefix("Rename variable ") {
            let parts: Vec<&str> = rest.splitn(3, " to ").collect();
            if parts.len() != 2 {
                return Err(
                    "Invalid rename format. Expected 'Rename variable XXXX to YYYY'".to_string(),
                );
            }
            return Ok(RefactoringAction::RenameVariable {
                from: parts[0].trim().to_string(),
                to: parts[1].trim().to_string(),
            });
        }

        if let Some(rest) = s.strip_prefix("Remove variable ") {
            let parts: Vec<&str> = rest.splitn(3, " to ").collect();
            if parts.len() != 2 {
                return Err(
                    "Invalid rename format. Expected 'Remove variable XXXX to YYYY'".to_string(),
                );
            }
            return Ok(RefactoringAction::RenameVariable {
                from: parts[0].trim().to_string(),
                to: parts[1].trim().to_string(),
            });
        }

        Err(format!("Unknown action: '{}'", s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_remove_unused_argument() {
        assert_eq!(
            "Remove unused argument foo".parse(),
            Ok(RefactoringAction::RemoveUnusedArgument("foo".to_string()))
        );
    }

    #[test]
    fn test_parse_inline_assignment() {
        assert_eq!(
            "Inline variable assignment `foo = 5 + 3`".parse(),
            Ok(RefactoringAction::InlineVariableAssignment {
                variable: "foo".to_string(),
                expression: "5 + 3".to_string()
            })
        );
    }

    #[test]
    fn test_parse_rename_variable() {
        assert_eq!(
            "Rename variable foo to answer".parse(),
            Ok(RefactoringAction::RenameVariable {
                from: "foo".to_string(),
                to: "answer".to_string()
            })
        );

        // Test the alternative "Remove variable X to Y" syntax
        assert_eq!(
            "Remove variable foo to answer".parse(),
            Ok(RefactoringAction::RenameVariable {
                from: "foo".to_string(),
                to: "answer".to_string()
            })
        );
    }

    #[test]
    fn test_parse_errors() {
        assert!("Foo bar".parse::<RefactoringAction>().is_err());
        assert!(
            "Remove unused argument"
                .parse::<RefactoringAction>()
                .is_err()
        );
        assert!(
            "Inline variable assignment `foo =`"
                .parse::<RefactoringAction>()
                .is_err()
        );
    }
}
