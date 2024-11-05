const EXAMPLE_AVRO: &str = include_str!("avro_schemas/example.avsc");

/// Maps env var keys to the name of the table
pub fn env_key_to_table_name(env_key: &str) -> &str {
    match env_key {
        "QUEUE_NAME_EXAMPLE" => "example",
        _ => panic!(
            "unexpected env_key: {}, env_key should be UPPERCASE and SNAKE_CASE",
            env_key
        ),
    }
}

/// Maps table names to the AVRO schema contents
pub fn table_to_avro(table_name: &str) -> &str {
    match table_name {
        "example" => EXAMPLE_AVRO,
        _ => panic!(
            "unexpected table_name: {}, table_name should be lowercase and snake_case",
            table_name
        ),
    }
}
