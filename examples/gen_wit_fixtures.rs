use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use ciborium::value::Value;

use greentic_types::I18nText;
use greentic_types::cbor::canonical;
use greentic_types::schemas::component::v0_6_0::{
    ComponentQaSpec, QaMode as ComponentQaMode, Question as ComponentQuestion,
    QuestionKind as ComponentQuestionKind,
};
use greentic_types::schemas::pack::v0_6_0::{
    CapabilityDescriptor, CapabilityMetadata, ChoiceOption as PackChoiceOption, PackDescribe,
    PackInfo, PackQaSpec, QaMode as PackQaMode, Question as PackQuestion,
    QuestionKind as PackQuestionKind,
};

fn main() {
    let base = Path::new("fixtures");
    fs::create_dir_all(base.join("pack")).expect("pack fixtures dir");
    fs::create_dir_all(base.join("component")).expect("component fixtures dir");

    write_pack_describe(base.join("pack/describe_v0_6_0.cbor"));
    write_pack_qa(base.join("pack/qa_setup_v0_6_0.cbor"));
    write_component_qa(base.join("component/qa_default_v0_6_0.cbor"));
}

fn write_pack_describe(path: impl AsRef<Path>) {
    let mut units = BTreeMap::new();
    units.insert("components".to_string(), Value::Integer(1.into()));
    units.insert("flows".to_string(), Value::Integer(2.into()));

    let mut metadata = BTreeMap::new();
    metadata.insert("notes".to_string(), Value::Text("demo".to_string()));

    let mut capability_supports = BTreeMap::new();
    capability_supports.insert("mode".to_string(), Value::Text("sync".to_string()));

    let cap_meta = CapabilityMetadata {
        tags: vec!["demo".to_string()],
        supports: capability_supports,
        constraints: BTreeMap::new(),
        quality_hints: BTreeMap::new(),
        regions: vec!["us".to_string()],
        compliance: BTreeMap::new(),
        hints: BTreeMap::new(),
    };

    let describe = PackDescribe {
        info: PackInfo {
            id: "greentic.demo.pack".to_string(),
            version: "0.6.0".to_string(),
            role: "application".to_string(),
            display_name: Some(I18nText::new(
                "pack.display_name",
                Some("Demo Pack".to_string()),
            )),
        },
        provided_capabilities: vec![CapabilityDescriptor {
            capability_id: "greentic.cap.demo".to_string(),
            version_req: "^1".to_string(),
            metadata: Some(cap_meta),
        }],
        required_capabilities: Vec::new(),
        units_summary: units,
        metadata,
    };

    let bytes = canonical::to_canonical_cbor(&describe).expect("pack describe cbor");
    fs::write(path, bytes).expect("write pack describe fixture");
}

fn write_pack_qa(path: impl AsRef<Path>) {
    let question = PackQuestion {
        id: "region".to_string(),
        label: I18nText::new("pack.qa.region.label", Some("Region".to_string())),
        help: None,
        error: None,
        kind: PackQuestionKind::Choice {
            options: vec![
                PackChoiceOption {
                    value: "eu".to_string(),
                    label: I18nText::new("pack.qa.region.option.eu", Some("EU".to_string())),
                },
                PackChoiceOption {
                    value: "us".to_string(),
                    label: I18nText::new("pack.qa.region.option.us", Some("US".to_string())),
                },
            ],
        },
        required: true,
        default: Some(Value::Text("eu".to_string())),
    };

    let mut defaults = BTreeMap::new();
    defaults.insert("region".to_string(), Value::Text("eu".to_string()));

    let spec = PackQaSpec {
        mode: PackQaMode::Setup,
        title: I18nText::new("pack.qa.setup.title", Some("Setup".to_string())),
        description: Some(I18nText::new(
            "pack.qa.setup.description",
            Some("Setup questions".to_string()),
        )),
        questions: vec![question],
        defaults,
    };

    let bytes = canonical::to_canonical_cbor(&spec).expect("pack qa cbor");
    fs::write(path, bytes).expect("write pack qa fixture");
}

fn write_component_qa(path: impl AsRef<Path>) {
    let question = ComponentQuestion {
        id: "api_key".to_string(),
        label: I18nText::new("component.qa.api_key.label", Some("API key".to_string())),
        help: Some(I18nText::new(
            "component.qa.api_key.help",
            Some("Provide your API key".to_string()),
        )),
        error: None,
        kind: ComponentQuestionKind::Text,
        required: true,
        default: None,
        skip_if: None,
    };

    let spec = ComponentQaSpec {
        mode: ComponentQaMode::Default,
        title: I18nText::new("component.qa.default.title", Some("Default".to_string())),
        description: None,
        questions: vec![question],
        defaults: BTreeMap::new(),
    };

    let bytes = canonical::to_canonical_cbor(&spec).expect("component qa cbor");
    fs::write(path, bytes).expect("write component qa fixture");
}
