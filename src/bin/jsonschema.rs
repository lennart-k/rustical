use rustical::config::Config;
use schemars::{JsonSchema, generate::SchemaSettings};

fn main() {
    let generator = SchemaSettings::draft07().into_generator();
    let config_schema = generator.into_root_schema_for::<Config>();
    println!("{}", serde_json::to_string_pretty(&config_schema).unwrap());
}
