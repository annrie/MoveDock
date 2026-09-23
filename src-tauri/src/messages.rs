//! Language-neutral application messages, carried through the existing string IPC contract.
//! External command output belongs in `detail` and must never be translated.
use serde_json::{json, Value};

pub fn encode(code: &str, params: Value, detail: &str) -> String {
    format!(
        "movedock-message-v1:{}",
        json!({ "code": code, "params": params, "detail": detail })
    )
}
pub fn message(code: &str) -> String {
    encode(code, json!({}), "")
}
pub fn with_params(code: &str, params: Value) -> String {
    encode(code, params, "")
}
pub fn with_detail(code: &str, detail: &str) -> String {
    encode(code, json!({}), detail)
}

pub fn with_cause(code: &str, cause: &str) -> String {
    format!(
        "movedock-message-v1:{}",
        json!({ "code": code, "params": {}, "detail": "", "cause": cause })
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_original_output_and_parameters() {
        let original = "Permission denied (publickey).\n/path/日本語/\"file\"";
        let value = encode("backend.commandFailed", json!({"code": "23"}), original);
        let decoded: Value =
            serde_json::from_str(value.strip_prefix("movedock-message-v1:").unwrap()).unwrap();
        assert_eq!(decoded["detail"], original);
        assert_eq!(decoded["params"]["code"], "23");
        assert_eq!(decoded["code"], "backend.commandFailed");
    }
}
