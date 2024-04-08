#![doc = include_str!("../README.md")]

pub mod dynamodb {
    use std::collections::HashMap;
    use std::error::Error;

    use aws_sdk_dynamodb::types::AttributeValue;
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use serde_json::{Map, Number, Value};

    pub fn marshall_t<T: Serialize>(m: &T) -> Result<AttributeValue, Box<dyn Error>> {
        let m = serde_json::to_value(m)?;
        Ok(marshall(&m))
    }

    pub fn unmarshall_t<T: DeserializeOwned>(m: &AttributeValue) -> Result<T, serde_json::Error> {
        let m = unmarshall(m);
        serde_json::from_value(m)
    }

    pub fn marshall(m: &Value) -> AttributeValue {
        match m {
            Value::Null => AttributeValue::Null(true),
            Value::Bool(b) => AttributeValue::Bool(*b),
            Value::Number(n) => AttributeValue::N(n.to_string()),
            Value::String(s) => AttributeValue::S(s.to_owned()),
            Value::Array(arr) => AttributeValue::L(arr.iter().map(marshall).collect()),
            Value::Object(o) => {
                let new_map = o
                    .iter()
                    .map(|(k, v)| (k.to_owned(), marshall(v)))
                    .collect::<HashMap<String, AttributeValue>>();

                AttributeValue::M(new_map)
            }
        }
    }

    pub fn unmarshall(m: &AttributeValue) -> Value {
        match m {
            AttributeValue::S(s) => Value::String(s.to_owned()),
            AttributeValue::B(blob) => Value::Array(
                blob.clone()
                    .into_inner()
                    .iter()
                    .map(|v| Value::Number(Number::from(*v)))
                    .collect::<Vec<Value>>(),
            ),
            AttributeValue::Bool(b) => Value::Bool(*b),
            AttributeValue::M(o) => {
                let new_map: Map<String, Value> = o
                    .iter()
                    .map(|(k, v)| (k.to_owned(), unmarshall(v)))
                    .collect();
                Value::Object(new_map)
            }
            AttributeValue::N(v) => {
                if v.contains('.') {
                    v.parse::<f64>()
                        .map_or_else(|_| Value::Null, |v| serde_json::json!(v))
                } else {
                    v.parse::<i64>()
                        .map_or_else(|_| Value::Null, |v| serde_json::json!(v))
                }
            }
            AttributeValue::L(arr) => {
                Value::Array(arr.iter().map(unmarshall).collect::<Vec<Value>>())
            }
            AttributeValue::Ns(arr) => Value::Array(
                arr.iter()
                    .map(|v| unmarshall(&AttributeValue::N(v.to_string())))
                    .collect::<Vec<Value>>(),
            ),
            AttributeValue::Bs(arr) => Value::Array(
                arr.iter()
                    .map(|v| unmarshall(&AttributeValue::B(v.to_owned())))
                    .collect::<Vec<Value>>(),
            ),
            AttributeValue::Ss(arr) => Value::Array(
                arr.iter()
                    .map(|s| Value::String(s.to_owned()))
                    .collect::<Vec<Value>>(),
            ),
            _ => Value::Null, // covers AttributeValue::Null(_) too
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};

    use crate::dynamodb;

    #[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq, Clone)]
    struct Example {
        hello: String,
        world: bool,
        a: i32,
        some: HashMap<String, String>,
        values: Vec<HashMap<String, String>>,
        other: Example2,
        others: Vec<Example2>,
        fake: Option<Example2>,
    }

    #[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq, Clone)]
    struct Example2 {
        hola: String,
        mundo: bool,
        algo: HashMap<String, String>,
        otro: u64,
    }

    #[test]
    fn it_works() {
        let example = Example {
            hello: "world".to_string(),
            world: false,
            a: 42,
            some: HashMap::from([("some".into(), "42".into()), ("value".into(), "42".into())]),
            values: vec![
                HashMap::from([("some".into(), "42".into()), ("value".into(), "42".into())]),
                HashMap::from([("some".into(), "42".into()), ("value".into(), "42".into())]),
                HashMap::from([("some".into(), "42".into()), ("value".into(), "42".into())]),
                HashMap::from([("some".into(), "42".into()), ("value".into(), "42".into())]),
                HashMap::from([("some".into(), "42".into()), ("value".into(), "42".into())]),
            ],
            other: Example2 {
                hola: "mundo".to_string(),
                mundo: true,
                algo: HashMap::from([("some".into(), "42".into()), ("value".into(), "42".into())]),
                otro: 42,
            },
            others: vec![
                Example2 {
                    hola: "mundo1".to_string(),
                    mundo: true,
                    algo: HashMap::from([
                        ("some".into(), "42".into()),
                        ("value".into(), "42".into()),
                    ]),
                    otro: 42,
                },
                Example2 {
                    hola: "mundo2".to_string(),
                    mundo: true,
                    algo: HashMap::from([
                        ("some".into(), "42".into()),
                        ("value".into(), "42".into()),
                    ]),
                    otro: 42,
                },
                Example2 {
                    hola: "mundo3".to_string(),
                    mundo: true,
                    algo: HashMap::from([
                        ("some".into(), "42".into()),
                        ("value".into(), "42".into()),
                    ]),
                    otro: 42,
                },
                Example2 {
                    hola: "mundo4".to_string(),
                    mundo: true,
                    algo: HashMap::from([
                        ("some".into(), "42".into()),
                        ("value".into(), "42".into()),
                    ]),
                    otro: 42,
                },
                Example2 {
                    hola: "mundo5".to_string(),
                    mundo: true,
                    algo: HashMap::from([
                        ("some".into(), "42".into()),
                        ("value".into(), "42".into()),
                    ]),
                    otro: 42,
                },
            ],
            fake: None,
        };

        // Clone object for later assertion
        let example_cloned = serde_json::to_value(&example)
            .expect("Failed to de example")
            .to_string();
        // println!(">> {example_cloned}");

        // Serialize example to AttributeValue
        let attr = dynamodb::marshall_t(&example).unwrap();

        // Deserialize example back into a serde json Value
        let result = dynamodb::unmarshall(&attr);

        // Check if they are equal
        assert_eq!(result.to_string(), example_cloned);
    }
}
