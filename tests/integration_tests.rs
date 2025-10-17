// tests/integration_tests.rs
use evaluate::config::EvalConfig;
use serde_json::json;

#[test]
fn test_eval_config_creation() {
    let eval = EvalConfig {
        model: "gemini:gemini-1.5-flash".to_string(),
        prompt: "What is 2+2?".to_string(),
        expected: Some("4".to_string()),
        judge_model: Some("gemini:gemini-1.5-pro".to_string()),
        criteria: None,
        tags: vec!["math".to_string()],
        metadata: None,
    };

    assert_eq!(eval.model, "gemini:gemini-1.5-flash");
    assert_eq!(eval.prompt, "What is 2+2?");
    assert_eq!(eval.expected, Some("4".to_string()));
    assert_eq!(eval.tags.len(), 1);
}

#[test]
fn test_template_rendering() {
    let eval = EvalConfig {
        model: "gemini:gemini-1.5-flash".to_string(),
        prompt: "Calculate {{num1}} + {{num2}}".to_string(),
        expected: Some("The answer is {{result}}".to_string()),
        judge_model: None,
        criteria: None,
        tags: vec![],
        metadata: Some(json!({
            "num1": "5",
            "num2": "3",
            "result": "8"
        })),
    };

    let rendered = eval.render().unwrap();
    
    assert_eq!(rendered.prompt, "Calculate 5 + 3");
    assert_eq!(rendered.expected, Some("The answer is 8".to_string()));
}
