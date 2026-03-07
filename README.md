# DynamoDB Type Serializer/Deserializer


> **NOTE: Handling DynamoDB Sets**
>
> DynamoDB has a distinct concept of sets (`Ss`, `Ns`, `Bs`), whereas JSON does not. By default, the marshalling and unmarshalling in this library treats all JSON arrays as DynamoDB lists (`AttributeValue::L`). Consequently, even if you start with a DynamoDB string set (`Ss`), once it’s converted to JSON and then back again, it will become a list of strings (`L`).

> **NOTE: Large Numbers**
>
> DynamoDB’s `N` type supports arbitrary-precision decimals, but JSON does not. During `unmarshall`, numbers are parsed as `i64` first, then `f64`. Values outside both ranges (e.g. numbers with more than ~15 significant digits) fall back to a JSON string. On re-marshalling that string becomes `AttributeValue::S`, not `AttributeValue::N`. If you need to preserve large or high-precision numbers, store them as strings in DynamoDB.

> **NOTE: Binary Data (`B` / `Bs`)**
>
> `AttributeValue::B` has no direct JSON equivalent. This library unmarshals binary blobs as a JSON array of byte values (`[1, 2, 3, ...]`). Re-marshalling that array produces `AttributeValue::L` (a list), not `AttributeValue::B`, so the roundtrip is lossy. `Vec<u8>` fields on Rust structs are serialized by serde_json as a number array and follow the same path; they are never stored as `AttributeValue::B`.


## Simple Example for `Value` -> `AttributeValue`
```rust
use aws_sdk_dynamodb::types::AttributeValue;
use serde_json::{Value, json};
use dynamodb_marshall::dynamodb;

fn main() {
    let input: Value = json!({
        "hello": "world",
        "n": 42,
        "some": {
            "deep": {
                "value": 42
            },
        },
    });

    // transform `Value` into a DynamoDB `AttributeValue`
    let value: AttributeValue = dynamodb::marshall(&input);
    // M({"hello": S("world"), "some": M({"deep": M({"value": N("42")})}), "n": N("42")})
    
    // ... upload value into dynamodb / do stuff

    // transform DynamoDB `AttributeValue` into a `Value`
    let original: Value = dynamodb::unmarshall(&value);
    // Object {"hello": String("world"), "n": Number(42), "some": Object {"deep": Object {"value": Number(42)}}}

    // Compare unmarshalled and input
    assert_eq!(
        input,
        original
    );
}
```

## For `struct` that derive from `Serialize, Deserialize`

```rust
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use aws_sdk_dynamodb::types::AttributeValue;
use dynamodb_marshall::dynamodb;

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq, Clone)]
struct Example {
    hello: String,
    world: bool,
    something: HashMap<String, String>,
    other: u64,
}

fn main() {
    let example = Example::default();
    //                                                         V may fail
    let value: AttributeValue = dynamodb::marshall_t(&example).unwrap();
    
    // Turn back to the struct                                            V may fail
    let same_example: Example = dynamodb::unmarshall_t::<Example>(&value).unwrap();
    
    assert_eq!(
        example,
        same_example,
    );
}
```