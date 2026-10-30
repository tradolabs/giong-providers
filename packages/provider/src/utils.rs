use serde::Serialize;
use serde_json::Value;

pub fn matches<S: Serialize, D: Serialize>(expected: &S, actual: &D) -> bool {
    let expected = serde_json::to_value(expected).unwrap();
    let actual = serde_json::to_value(actual).unwrap();

    fn eq(expected: &Value, actual: &Value) -> bool {
        match (expected, actual) {
            (Value::Object(expected), Value::Object(actual)) => expected
                .iter()
                .all(|(key, value)| actual.get(key).is_some_and(|item| eq(value, item))),
            (Value::Array(expected), Value::Array(actual)) => {
                expected.len() == actual.len() && expected.iter().zip(actual).all(|(a, b)| eq(a, b))
            }
            _ => expected == actual,
        }
    }

    eq(&expected, &actual)
}
