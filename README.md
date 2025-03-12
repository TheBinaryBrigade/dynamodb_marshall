# DynamoDB Type Serializer/Deserializer


> **NOTE: Handling DynamoDB Sets**
> 
> DynamoDB has a distinct concept of sets (`Ss`, `Ns`, `Bs`), whereas JSON does not. By default, the marshalling and unmarshalling in this library treats all JSON arrays as DynamoDB lists (`AttributeValue::L`). Consequently, even if you start with a DynamoDB string set (`Ss`), once it’s converted to JSON and then back again, it will become a list of strings (`L`).


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