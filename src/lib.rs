#![doc = include_str!("../README.md")]

pub mod dynamodb {
    use std::collections::HashMap;
    use std::error::Error;

    use aws_sdk_dynamodb::types::AttributeValue;
    use serde::Serialize;
    use serde::de::DeserializeOwned;
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
                    v.parse::<f64>().map_or_else(
                        |_| serde_json::json!(v),
                        |parsed_float| serde_json::json!(parsed_float),
                    )
                } else {
                    v.parse::<i64>().map_or_else(
                        |_| serde_json::json!(v),
                        |parsed_int| serde_json::json!(parsed_int),
                    )
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

    use aws_sdk_dynamodb::types::AttributeValue;
    use serde::{Deserialize, Serialize};
    use serde_json::{Value as JsonValue, json};

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

    #[test]
    fn test_simple_struct() {
        #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
        struct SimpleStruct {
            name: String,
            age: u32,
        }

        let example = SimpleStruct {
            name: "Alice".to_owned(),
            age: 30,
        };

        let as_json = serde_json::to_value(&example).unwrap().to_string();
        let attr = dynamodb::marshall_t(&example).expect("Failed to marshall SimpleStruct");
        let round_trip = dynamodb::unmarshall(&attr);

        // Compare JSON strings
        assert_eq!(round_trip.to_string(), as_json);
    }

    #[test]
    fn test_optional_values_some() {
        #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
        struct WithOption {
            id: u32,
            label: Option<String>,
        }

        // Here, label is Some
        let example = WithOption {
            id: 999,
            label: Some("Hello Option".to_owned()),
        };

        let as_json = serde_json::to_value(&example).unwrap().to_string();
        let attr = dynamodb::marshall_t(&example).expect("Failed to marshall WithOption");
        let round_trip = dynamodb::unmarshall(&attr);

        assert_eq!(round_trip.to_string(), as_json);
    }

    #[test]
    fn test_optional_values_none() {
        #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
        struct WithOption {
            id: u32,
            label: Option<String>,
        }

        // Here, label is None
        let example = WithOption {
            id: 999,
            label: None,
        };

        let as_json = serde_json::to_value(&example).unwrap().to_string();
        let attr = dynamodb::marshall_t(&example).expect("Failed to marshall WithOption");
        let round_trip = dynamodb::unmarshall(&attr);

        assert_eq!(round_trip.to_string(), as_json);
    }

    #[test]
    fn test_floating_numbers() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Floats {
            value1: f64,
            value2: f32,
        }

        let example = Floats {
            value1: 123.456,
            value2: 78.9,
        };

        let as_json = serde_json::to_value(&example).unwrap().to_string();
        let attr = dynamodb::marshall_t(&example).expect("Failed to marshall Floats");
        let round_trip = dynamodb::unmarshall(&attr);

        assert_eq!(round_trip.to_string(), as_json);
    }

    #[test]
    fn test_byte_array() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct BinaryData {
            data: Vec<u8>,
        }

        let example = BinaryData {
            data: vec![1, 2, 3, 4, 255],
        };

        let as_json = serde_json::to_value(&example).unwrap().to_string();
        let attr = dynamodb::marshall_t(&example).expect("Failed to marshall BinaryData");
        let round_trip = dynamodb::unmarshall(&attr);

        // Check if the JSON matches.
        // The byte array is stored as a JSON array of numbers, e.g., [1,2,3,4,255].
        assert_eq!(round_trip.to_string(), as_json);
    }

    #[test]
    fn test_large_integers() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct LargeIntegers {
            big_positive: i64,
            big_negative: i64,
        }

        let example = LargeIntegers {
            big_positive: 9_223_372_036_854_775_807,  // i64::MAX
            big_negative: -9_223_372_036_854_775_808, // i64::MIN
        };

        let as_json = serde_json::to_value(&example).unwrap().to_string();
        let attr = dynamodb::marshall_t(&example).expect("Failed to marshall LargeIntegers");
        let round_trip = dynamodb::unmarshall(&attr);

        assert_eq!(round_trip.to_string(), as_json);
    }

    #[test]
    fn test_empty_collections() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct EmptyStruct {
            empty_vec: Vec<i32>,
            empty_map: HashMap<String, String>,
        }

        let data = EmptyStruct {
            empty_vec: vec![],
            empty_map: HashMap::new(),
        };
        let json_str = serde_json::to_value(&data).unwrap().to_string();

        let attr = dynamodb::marshall_t(&data).expect("Failed to marshall empty collections");
        let round_trip = dynamodb::unmarshall(&attr);

        assert_eq!(round_trip.to_string(), json_str);
    }

    #[test]
    fn test_dynamodb_sets() {
        let input = AttributeValue::Ss(vec![
            "apple".to_owned(),
            "banana".to_owned(),
            "cherry".to_owned(),
        ]);

        // Convert from AttributeValue to serde_json::Value
        let as_json = dynamodb::unmarshall(&input);
        // Should be an array of strings ["apple", "banana", "cherry"]
        assert_eq!(
            as_json,
            JsonValue::Array(vec![
                JsonValue::String("apple".to_string()),
                JsonValue::String("banana".to_string()),
                JsonValue::String("cherry".to_string()),
            ])
        );

        // Convert back to AttributeValue -- lossy :(
        let rem = dynamodb::marshall(&as_json);
        assert_eq!(
            rem,
            AttributeValue::L(vec![
                AttributeValue::S("apple".to_owned()),
                AttributeValue::S("banana".to_owned()),
                AttributeValue::S("cherry".to_owned()),
            ])
        );
    }

    #[test]
    fn test_unparseable_numbers() {
        let input = AttributeValue::N("123abc".to_string());

        // Dynamically convert to JSON Value
        let json_val = dynamodb::unmarshall(&input);
        // We expect it to remain a string in JSON, because it can't parse as i64 or f64
        assert_eq!(json_val, JsonValue::String("123abc".to_string()));

        // Converting back should yield the original `AttributeValue::S("123abc")`
        // Because when we marshall that JSON string, we get an S-attribute.
        let rem = dynamodb::marshall(&json_val);
        assert_eq!(rem, AttributeValue::S("123abc".to_string()));
    }

    #[test]
    fn test_out_of_range_integers() {
        // This number is larger than i64::MAX
        let input = AttributeValue::N("999999999999999999999".to_string());

        let json_val = dynamodb::unmarshall(&input);
        // Fallback is string in JSON since it can't parse into i64/f64
        assert_eq!(
            json_val,
            JsonValue::String("999999999999999999999".to_string())
        );

        // Round-trip
        let rem = dynamodb::marshall(&json_val);
        // Should become S("999999999999999999999") on re-marshall
        assert_eq!(rem, AttributeValue::S("999999999999999999999".to_string()));
    }

    #[test]
    fn test_nested_optional_fields() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Inner {
            value: Option<String>,
        }

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Outer {
            inner: Option<Inner>,
        }

        let example = Outer {
            inner: Some(Inner {
                value: None, // The inner struct is present, but the field is None
            }),
        };

        let json_str = serde_json::to_value(&example).unwrap().to_string();
        let attr = dynamodb::marshall_t(&example).expect("Failed to marshall nested optional");
        let round_trip = dynamodb::unmarshall(&attr);

        assert_eq!(round_trip.to_string(), json_str);
    }

    #[test]
    fn test_mix_null_and_valid() {
        let json_val = json!({
            "someField": null,
            "otherField": 42,
            "arrayField": [null, 1, true, "string"],
        });

        let attr = dynamodb::marshall(&json_val);
        let round_trip = dynamodb::unmarshall(&attr);

        assert_eq!(round_trip, json_val);
    }

    #[test]
    fn test_booleans_in_arrays_and_maps() {
        let json_val = json!({
            "trueVal": true,
            "falseVal": false,
            "mixedArray": [true, false, 123, "hello"],
        });

        let attr = dynamodb::marshall(&json_val);
        let round_trip = dynamodb::unmarshall(&attr);

        assert_eq!(round_trip, json_val);
    }

    #[test]
    fn test_special_strings() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct SpecialStrings {
            regular: String,
            emoji: String,
            unicode: String,
        }

        let data = SpecialStrings {
            regular: "Hello World!".to_owned(),
            emoji: "😀🔥".to_owned(),
            unicode: "こんにちは世界".to_owned(),
        };

        let json_str = serde_json::to_value(&data).unwrap().to_string();
        let attr = dynamodb::marshall_t(&data).expect("Failed to marshall special strings");
        let round_trip = dynamodb::unmarshall(&attr);

        assert_eq!(round_trip.to_string(), json_str);
    }

    #[test]
    fn test_mixed_array_data_types() {
        let json_val = json!(["hello", 123, true, null, {"nested": "object"}]);

        let attr = dynamodb::marshall(&json_val);
        let round_trip = dynamodb::unmarshall(&attr);

        assert_eq!(round_trip, json_val);
    }

    #[test]
    fn test_deeply_nested() {
        // 3-level nested JSON
        let json_val = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "value": 999
                    }
                }
            }
        });

        let attr = dynamodb::marshall(&json_val);
        let round_trip = dynamodb::unmarshall(&attr);
        assert_eq!(round_trip, json_val);
    }

    #[test]
    fn test_zero_values() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Zeros {
            int_zero: i32,
            float_zero: f64,
            empty_str: String,
        }

        let data = Zeros {
            int_zero: 0,
            float_zero: 0.0,
            empty_str: "".to_owned(),
        };

        let json_str = serde_json::to_value(&data).unwrap().to_string();
        let attr = dynamodb::marshall_t(&data).expect("Failed to marshall zeros");
        let round_trip = dynamodb::unmarshall(&attr);

        assert_eq!(round_trip.to_string(), json_str);
    }

    #[test]
    fn test_enums() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        enum SimpleEnum {
            UnitVariant,
            NewTypeVariant(i32),
            StructVariant { x: String, y: bool },
        }

        let data = SimpleEnum::StructVariant {
            x: "example".to_string(),
            y: true,
        };

        let json_str = serde_json::to_value(&data).unwrap().to_string();
        let attr = dynamodb::marshall_t(&data).expect("Failed to marshall enum");
        let round_trip = dynamodb::unmarshall(&attr);

        assert_eq!(round_trip.to_string(), json_str);
    }
}
