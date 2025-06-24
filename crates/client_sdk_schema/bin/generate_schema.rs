use client_sdk_schema::Schema;
use schemars::schema_for;
use std::fs::File;
use std::io::Write;

fn main() {
    let schema = schema_for!(Schema);
    let output = serde_json::to_string_pretty(&schema).unwrap();

    let mut file = File::create("schema.json").unwrap();
    file.write_all(output.as_bytes()).unwrap();
}
