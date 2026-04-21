use greentic_types::messaging::extensions::ext_keys;

/// These values appear in serialized envelopes across process and repo
/// boundaries. Changing any of them is a breaking wire-format change that
/// must coordinate with every consumer of `ChannelMessageEnvelope.extensions`.
#[test]
fn well_known_extension_keys_are_stable() {
    assert_eq!(ext_keys::ADAPTIVE_CARD, "adaptive_card");
    assert_eq!(ext_keys::CHANNEL_DATA, "channel_data");
    assert_eq!(ext_keys::ENTITIES, "entities");
    assert_eq!(ext_keys::ATTACHMENTS, "attachments");
    assert_eq!(ext_keys::RAG, "rag");
    assert_eq!(ext_keys::SUGGESTED_ACTIONS, "suggested_actions");
    assert_eq!(ext_keys::SPEAK, "speak");
    assert_eq!(ext_keys::INPUT_HINT, "input_hint");
}
