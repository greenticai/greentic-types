use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use greentic_types::cbor::canonical;
use greentic_types::schemas::common::schema_ir::{AdditionalProperties, SchemaIr};
use greentic_types::schemas::component::v0_6_0::{
    ComponentDescribe, ComponentInfo, ComponentOperation, ComponentRunInput, ComponentRunOutput,
    schema_hash,
};

fn main() {
    let input_schema = SchemaIr::Object {
        properties: BTreeMap::from([(
            "prompt".to_string(),
            SchemaIr::String {
                min_len: Some(1),
                max_len: None,
                regex: None,
                format: None,
            },
        )]),
        required: vec!["prompt".to_string()],
        additional: AdditionalProperties::Forbid,
    };

    let output_schema = SchemaIr::Object {
        properties: BTreeMap::from([(
            "result".to_string(),
            SchemaIr::String {
                min_len: Some(1),
                max_len: None,
                regex: None,
                format: None,
            },
        )]),
        required: vec!["result".to_string()],
        additional: AdditionalProperties::Forbid,
    };

    let config_schema = SchemaIr::Object {
        properties: BTreeMap::from([(
            "api_key".to_string(),
            SchemaIr::String {
                min_len: Some(1),
                max_len: None,
                regex: None,
                format: None,
            },
        )]),
        required: vec!["api_key".to_string()],
        additional: AdditionalProperties::Forbid,
    };

    let schema_hash =
        schema_hash(&input_schema, &output_schema, &config_schema).expect("schema hash");

    let describe = ComponentDescribe {
        info: ComponentInfo {
            id: "greentic.example.component".to_string(),
            version: "0.1.0".to_string(),
            role: "tool".to_string(),
            display_name: None,
        },
        provided_capabilities: Vec::new(),
        required_capabilities: Vec::new(),
        metadata: BTreeMap::new(),
        operations: vec![ComponentOperation {
            id: "run".to_string(),
            display_name: None,
            input: ComponentRunInput {
                schema: input_schema,
            },
            output: ComponentRunOutput {
                schema: output_schema,
            },
            defaults: BTreeMap::new(),
            redactions: Vec::new(),
            constraints: BTreeMap::new(),
            schema_hash,
        }],
        config_schema,
        supports: Vec::new(),
        capabilities: None,
        profiles: None,
        configurators: None,
        resources: None,
        secret_requirements: Vec::new(),
    };

    let bytes = canonical::to_canonical_cbor_allow_floats(&describe).expect("canonical encode");
    let path = Path::new("fixtures/component/describe_v0_6_0.cbor");
    fs::write(path, bytes).expect("write fixture");
}
