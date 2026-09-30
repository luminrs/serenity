//! Ori CV2 fork (F1): raw data is preserved, rows are lenient, and nothing is dropped.
//!
//! The differential comparison with stock 0.12.5 lives in `cv2-tests/`; these tests check the
//! fork on its own.
// The fixtures are `serde_json` values; with `simd_json` the raw fields are simd-json values.
#![cfg(all(feature = "model", not(feature = "simd_json")))]

#[path = "cv2_fixtures/mod.rs"]
mod fixtures;

use fixtures::{Case, Expect, Kind, ATT, MEMBER, ROLE};
use serde::Deserialize;
use serde_json::{json, Value};
use serenity::all::{
    ActionRowComponent,
    ComponentInteractionDataKind,
    ComponentType,
    CreateAttachment,
    Event,
    GatewayEvent,
    Interaction,
    InteractionResponseFlags,
    Message,
    MessageFlags,
    MessageUpdateEvent,
};

// ---------------------------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------------------------

fn message(v: &Value) -> Message {
    serde_json::from_value(v.clone()).expect("message parses")
}

fn interaction(v: &Value) -> Interaction {
    serde_json::from_value(v.clone()).expect("interaction parses")
}

/// Asserts that every raw field of a message (and of its nested messages) equals the input.
fn assert_message_raw(msg: &Message, input: &Value) {
    assert_eq!(msg.raw_components.as_ref(), input.get("components"), "Message::raw_components");
    match (&msg.referenced_message, input.get("referenced_message")) {
        (Some(referenced), Some(raw)) => assert_message_raw(referenced, raw),
        (None, None | Some(Value::Null)) => {},
        (a, b) => panic!("referenced_message mismatch: {a:?} vs {b:?}"),
    }
    let raw_snapshots = input.get("message_snapshots").and_then(Value::as_array);
    assert_eq!(msg.message_snapshots.len(), raw_snapshots.map_or(0, Vec::len));
    for (snapshot, raw) in msg.message_snapshots.iter().zip(raw_snapshots.into_iter().flatten()) {
        assert_eq!(
            snapshot.raw_components.as_ref(),
            raw["message"].get("components"),
            "MessageSnapshot::raw_components"
        );
    }
}

fn assert_interaction_raw(interaction: &Interaction, input: &Value) {
    match interaction {
        Interaction::Component(i) => {
            assert_eq!(i.data.raw_data, input["data"], "ComponentInteractionData::raw_data");
            assert_message_raw(&i.message, &input["message"]);
        },
        Interaction::Modal(i) => {
            assert_eq!(i.data.raw_data, input["data"], "ModalInteractionData::raw_data");
            if let Some(m) = &i.message {
                assert_message_raw(m, &input["message"]);
            }
        },
        _ => {},
    }
}

/// Parses a case the way the library would and checks the raw fields; `Err` if it fails.
fn parse_and_check(case: &Case) -> Result<(), String> {
    let input = &case.json;
    match case.kind {
        Kind::Message => {
            let msg: Message = serde_json::from_value(input.clone()).map_err(|e| e.to_string())?;
            assert_message_raw(&msg, input);
        },
        Kind::Interaction => {
            let i: Interaction =
                serde_json::from_value(input.clone()).map_err(|e| e.to_string())?;
            assert_interaction_raw(&i, input);
        },
        Kind::MessageUpdate => {
            let ev: MessageUpdateEvent =
                serde_json::from_value(input.clone()).map_err(|e| e.to_string())?;
            assert_eq!(ev.raw_components.as_ref(), input.get("components"));
        },
        Kind::Gateway => {
            // As the shard does it (`gateway/ws.rs`): from the frame's text.
            let text = input.to_string();
            let ev: GatewayEvent = serenity::json::from_str(&text).map_err(|e| e.to_string())?;
            let GatewayEvent::Dispatch(_, event) = ev else { return Ok(()) };
            let d = &input["d"];
            match event {
                Event::MessageCreate(e) => assert_message_raw(&e.message, d),
                Event::MessageUpdate(e) => {
                    assert_eq!(e.raw_components.as_ref(), d.get("components"));
                },
                Event::InteractionCreate(e) => assert_interaction_raw(&e.interaction, d),
                Event::Unknown(e) => return Err(format!("demoted to Event::Unknown({})", e.kind)),
                _ => {},
            }
        },
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Every fixture
// ---------------------------------------------------------------------------------------------

#[test]
fn every_fixture_parses_with_raw_data_preserved() {
    let cases = fixtures::all_cases();
    assert!(cases.len() >= 80, "only {} cases", cases.len());
    let mut failures = Vec::new();
    for case in &cases {
        let result = parse_and_check(case);
        match (&case.expect, result) {
            (Expect::BothFail, Ok(())) => failures.push(format!("{}: parsed but must fail", case.id)),
            (Expect::BothFail, Err(_)) | (_, Ok(())) => {},
            (_, Err(e)) => failures.push(format!("{}: {e}", case.id)),
        }
    }
    assert!(failures.is_empty(), "failures:\n{}", failures.join("\n"));
}

// ---------------------------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------------------------

#[test]
fn v2_message_keeps_layout_and_flag() {
    let input = fixtures::v2_welcome_message();
    let msg = message(&input);
    assert!(msg.components.is_empty(), "a Container is not a typed row");
    assert_eq!(msg.raw_components.as_ref(), Some(&input["components"]));
    assert_eq!(msg.raw_components.as_ref().unwrap()[0]["type"], 17);
    let flags = msg.flags.unwrap();
    assert!(flags.contains(MessageFlags::IS_COMPONENTS_V2));
    assert_eq!(flags.bits(), 1 << 15);
    // Serialized output is unchanged apart from the flag bit, and never has the raw field.
    let out = serde_json::to_value(&msg).unwrap();
    assert_eq!(out["flags"], 32768);
    assert!(out.get("raw_components").is_none());
    assert_eq!(out["components"], json!([]));
}

#[test]
fn top_level_row_beside_container_is_typed() {
    let msg = message(&fixtures::v2_container_plus_top_row());
    assert_eq!(msg.components.len(), 1);
    assert_eq!(msg.components[0].components.len(), 3);
    assert_eq!(msg.raw_components.unwrap().as_array().unwrap().len(), 2);
}

#[test]
fn unknown_child_in_row_is_skipped_not_fatal() {
    for input in [fixtures::bad_row_message(), fixtures::text_in_row_message()] {
        let msg = message(&input);
        assert_eq!(msg.components.len(), 1, "the row is kept");
        assert!(msg.components[0].components.is_empty(), "only the bad child is skipped");
        assert_eq!(msg.raw_components.as_ref(), Some(&input["components"]));
    }
}

#[test]
fn malformed_child_of_known_type_is_skipped() {
    let bad = json!({"type": 2, "id": 32, "label": "Broken", "custom_id": "broken"});
    let input = fixtures::base_message(json!([fixtures::row_with(bad)]));
    let msg = message(&input);
    let children = &msg.components[0].components;
    assert_eq!(children.len(), 1);
    assert!(matches!(&children[0], ActionRowComponent::Button(b) if b.label.as_deref() == Some("Keep")));
    assert_eq!(msg.raw_components.unwrap()[0]["components"][1]["custom_id"], "broken");
}

#[test]
fn malformed_top_level_entries_are_skipped() {
    let input = fixtures::base_message(json!([
        fixtures::good_row(),
        {"type": 1, "id": 40, "components": "oops"},
        {"id": 41},
        {"type": "1"},
        17
    ]));
    let msg = message(&input);
    assert_eq!(msg.components.len(), 1);
    assert_eq!(msg.raw_components.unwrap().as_array().unwrap().len(), 5);
}

#[test]
fn missing_components_key_gives_none() {
    let mut input = fixtures::plain_message(json!({}));
    input.as_object_mut().unwrap().remove("components");
    let msg = message(&input);
    assert!(msg.raw_components.is_none());
    assert!(msg.components.is_empty());
    assert!(Message::default().raw_components.is_none());
}

#[test]
fn null_components_still_fails_as_in_0_12_5() {
    let input = fixtures::plain_message(json!({"components": null}));
    assert!(serde_json::from_value::<Message>(input).is_err());
}

#[test]
fn forwarded_and_replied_v2_messages_keep_raw() {
    let fwd = fixtures::forwarded_v2();
    let msg = message(&fwd);
    let snapshot = &msg.message_snapshots[0];
    assert_eq!(snapshot.raw_components.as_ref(), Some(&fwd["message_snapshots"][0]["message"]["components"]));
    assert!(snapshot.flags.unwrap().contains(MessageFlags::IS_COMPONENTS_V2));

    let reply = fixtures::reply_to_v2();
    let msg = message(&reply);
    let referenced = msg.referenced_message.unwrap();
    assert_eq!(referenced.raw_components.as_ref(), Some(&reply["referenced_message"]["components"]));

    let bad_fwd = fixtures::forward_of(&fixtures::bad_row_message());
    let msg = message(&bad_fwd);
    assert_eq!(msg.message_snapshots[0].components.len(), 1);
    assert!(msg.message_snapshots[0].components[0].components.is_empty());
}

/// Every way to reach `Deserialize` fills the raw field: there is no inherent `deserialize` (as
/// `#[serde(remote = "Self")]` would generate) that could skip it.
#[test]
fn only_deserialize_path_fills_raw() {
    let input = fixtures::v2_welcome_message();
    let text = input.to_string();
    let expected = Some(&input["components"]);

    let a = Message::deserialize(input.clone()).unwrap();
    let b = Message::deserialize(&mut serde_json::Deserializer::from_str(&text)).unwrap();
    let c: Message = serde_json::from_str(&text).unwrap();
    let d: Message = serde_json::from_slice(text.as_bytes()).unwrap();
    let e: Message = serenity::json::from_value(input.clone()).unwrap();
    let f: Message = serenity::json::from_str(text.as_str()).unwrap();
    for m in [a, b, c, d, e, f] {
        assert_eq!(m.raw_components.as_ref(), expected);
    }

    let snap = &fixtures::forwarded_v2()["message_snapshots"][0]["message"];
    let s = serenity::all::MessageSnapshot::deserialize(snap.clone()).unwrap();
    assert_eq!(s.raw_components.as_ref(), Some(&snap["components"]));

    let u = MessageUpdateEvent::deserialize(input.clone()).unwrap();
    assert_eq!(u.raw_components.as_ref(), expected);

    let click = fixtures::button_click_on_v2();
    let data = serenity::all::ComponentInteractionData::deserialize(click["data"].clone()).unwrap();
    assert_eq!(data.raw_data, click["data"]);

    let form = fixtures::modal_label();
    let data = serenity::all::ModalInteractionData::deserialize(form["data"].clone()).unwrap();
    assert_eq!(data.raw_data, form["data"]);
}

// ---------------------------------------------------------------------------------------------
// Message updates and the cache path
// ---------------------------------------------------------------------------------------------

#[test]
fn message_update_copies_raw_into_cached_message() {
    let mut cached = message(&fixtures::plain_message(json!({})));
    assert_eq!(cached.raw_components, Some(json!([])));

    let update_input = fixtures::v2_welcome_message();
    let update: MessageUpdateEvent = serde_json::from_value(update_input.clone()).unwrap();
    assert_eq!(update.components.as_ref().map(Vec::len), Some(0));
    update.apply_to_message(&mut cached);
    assert_eq!(cached.raw_components.as_ref(), Some(&update_input["components"]));
    assert!(cached.flags.unwrap().contains(MessageFlags::IS_COMPONENTS_V2));

    // An update without `components` leaves both fields alone.
    let partial: MessageUpdateEvent = serde_json::from_value(json!({
        "id": fixtures::MSG, "channel_id": fixtures::CHANNEL, "content": "edited"
    }))
    .unwrap();
    assert!(partial.components.is_none());
    assert!(partial.raw_components.is_none());
    partial.apply_to_message(&mut cached);
    assert_eq!(cached.raw_components.as_ref(), Some(&update_input["components"]));
    assert_eq!(cached.content, "edited");

    let out = serde_json::to_value(&update).unwrap();
    assert!(out.get("raw_components").is_none());
}

// ---------------------------------------------------------------------------------------------
// Gateway
// ---------------------------------------------------------------------------------------------

fn gateway_event(frame: &Value) -> Event {
    let text = frame.to_string();
    match serenity::json::from_str::<GatewayEvent>(text.as_str()).unwrap() {
        GatewayEvent::Dispatch(_, event) => event,
        other => panic!("not a dispatch: {other:?}"),
    }
}

#[test]
fn gateway_events_are_no_longer_demoted_to_unknown() {
    let msg = fixtures::gateway("MESSAGE_CREATE", 46, fixtures::gateway_message(fixtures::bad_row_message()));
    let Event::MessageCreate(e) = gateway_event(&msg) else { panic!("C5 demoted") };
    assert_eq!(e.message.components.len(), 1);
    assert_eq!(e.message.raw_components.as_ref(), Some(&msg["d"]["components"]));

    let click = fixtures::gateway("INTERACTION_CREATE", 47, fixtures::button_click_on_bad_row());
    let Event::InteractionCreate(e) = gateway_event(&click) else { panic!("C6 demoted") };
    let Interaction::Component(i) = e.interaction else { panic!("not a click") };
    assert_eq!(i.data.custom_id, "welcome:hi");
    assert_eq!(i.message.raw_components.as_ref(), Some(&click["d"]["message"]["components"]));

    let update = fixtures::gateway("MESSAGE_UPDATE", 80, fixtures::gateway_message(fixtures::bad_row_message()));
    let Event::MessageUpdate(e) = gateway_event(&update) else { panic!("update demoted") };
    assert_eq!(e.raw_components.as_ref(), Some(&update["d"]["components"]));
}

// ---------------------------------------------------------------------------------------------
// Forms
// ---------------------------------------------------------------------------------------------

fn modal_data(v: &Value) -> serenity::all::ModalInteractionData {
    match interaction(v) {
        Interaction::Modal(m) => m.data,
        other => panic!("not a modal: {other:?}"),
    }
}

#[test]
fn label_form_keeps_every_value_in_raw_data() {
    let input = fixtures::modal_label();
    let data = modal_data(&input);
    // Typed rows are exactly as in 0.12.5.
    let kinds: Vec<_> = data.components.iter().map(|r| r.kind).collect();
    assert_eq!(kinds, [ComponentType::Unknown(10), ComponentType::Unknown(18), ComponentType::Unknown(18)]);
    assert!(data.components.iter().all(|r| r.components.is_empty()));
    // The values are in raw_data.
    assert_eq!(data.raw_data, input["data"]);
    assert_eq!(data.raw_data["components"][1]["component"]["value"], "Welcome {user}!");
    assert_eq!(data.raw_data["components"][2]["component"]["values"], json!(["classic"]));
}

#[test]
fn every_form_field_kind_is_kept() {
    for case in fixtures::form_cases() {
        let data = modal_data(&case.json);
        assert_eq!(data.raw_data, case.json["data"], "{}", case.id);
        assert!(data.raw_data["resolved"]["attachments"][ATT]["size"].is_number(), "{}", case.id);
    }
    // M3-M6 through the probe ids.
    let probe = fixtures::probe_cases();
    let get = |id: &str| probe.iter().find(|c| c.id == id).unwrap().json.clone();
    assert_eq!(modal_data(&get("M3")).raw_data["components"][0]["component"]["value"], "warm");
    assert_eq!(modal_data(&get("M4")).raw_data["components"][0]["component"]["values"], json!(["a", "b"]));
    assert_eq!(modal_data(&get("M5")).raw_data["components"][0]["component"]["value"], true);
    let m6 = modal_data(&get("M6"));
    assert_eq!(m6.raw_data["components"][0]["component"]["values"], json!([ATT]));
    assert_eq!(m6.raw_data["resolved"]["attachments"][ATT]["filename"], "welcome.png");
    assert_eq!(modal_data(&get("M2")).raw_data["resolved"]["users"][MEMBER]["username"], "newmember");
}

#[test]
fn malformed_form_rows_are_skipped_not_fatal() {
    let input = fixtures::modal(
        json!([
            {"type": 1, "id": 1, "components": [
                {"type": 4, "id": 2, "custom_id": "greeting", "value": "hi"},
                {"type": 99, "id": 3, "custom_id": "future"}
            ]},
            {"type": 1, "id": 4, "components": "oops"}
        ]),
        None,
    );
    let data = modal_data(&input);
    assert_eq!(data.components.len(), 1);
    assert_eq!(data.components[0].components.len(), 1);
    assert!(matches!(
        &data.components[0].components[0],
        ActionRowComponent::InputText(t) if t.value.as_deref() == Some("hi")
    ));
    assert_eq!(data.raw_data, input["data"]);
}

// ---------------------------------------------------------------------------------------------
// Clicks
// ---------------------------------------------------------------------------------------------

fn click_data(v: &Value) -> serenity::all::ComponentInteractionData {
    match interaction(v) {
        Interaction::Component(c) => c.data,
        other => panic!("not a click: {other:?}"),
    }
}

#[test]
fn every_click_kind_parses_with_raw_data() {
    let cases = fixtures::click_cases();
    let get = |id: &str| click_data(&cases.iter().find(|c| c.id == id).unwrap().json);

    let button = get("K-button");
    assert!(matches!(button.kind, ComponentInteractionDataKind::Button));
    assert_eq!(button.raw_data["id"], 41, "the clicked component's numeric id is kept");

    assert!(matches!(get("K-string").kind, ComponentInteractionDataKind::StringSelect { ref values } if values == &["classic"]));
    let user = get("K-user");
    assert!(matches!(user.kind, ComponentInteractionDataKind::UserSelect { .. }));
    assert_eq!(user.resolved.users.len(), 1);
    let role = get("K-role");
    assert!(matches!(role.kind, ComponentInteractionDataKind::RoleSelect { ref values } if values.len() == 2));
    assert_eq!(role.resolved.roles.len(), 2);
    assert!(matches!(get("K-mentionable").kind, ComponentInteractionDataKind::MentionableSelect { .. }));
    assert!(matches!(get("K-channel").kind, ComponentInteractionDataKind::ChannelSelect { .. }));
    let unknown = get("K-unknown");
    assert!(matches!(unknown.kind, ComponentInteractionDataKind::Unknown(30)));
    assert_eq!(unknown.raw_data["values"], json!(["x"]));

    // Serialized output never has the raw field.
    let out = serde_json::to_value(&role).unwrap();
    assert!(out.get("raw_data").is_none());
    assert_eq!(out["component_type"], 6);
}

#[test]
fn click_with_malformed_resolved_is_kept() {
    let mut bad_role = fixtures::role(ROLE, "Player");
    bad_role.as_object_mut().unwrap().remove("colors");
    let input = fixtures::click(
        json!({"custom_id": "pick:role", "component_type": 6, "id": 26, "values": [ROLE],
               "resolved": {"roles": {ROLE: bad_role.clone()}}}),
        fixtures::v2_selects_message(),
    );
    let data = click_data(&input);
    assert!(matches!(data.kind, ComponentInteractionDataKind::RoleSelect { ref values } if values.len() == 1));
    assert!(data.resolved.roles.is_empty(), "the typed resolved falls back to empty");
    assert_eq!(data.raw_data["resolved"]["roles"][ROLE], bad_role, "the entry is intact in raw_data");

    // Through the gateway too.
    let frame = fixtures::gateway("INTERACTION_CREATE", 90, input);
    assert!(matches!(gateway_event(&frame), Event::InteractionCreate(_)));
}

// ---------------------------------------------------------------------------------------------
// Flags and attachments
// ---------------------------------------------------------------------------------------------

#[test]
fn components_v2_flags() {
    assert_eq!(MessageFlags::IS_COMPONENTS_V2.bits(), 1 << 15);
    assert_eq!(InteractionResponseFlags::IS_COMPONENTS_V2.bits(), 1 << 15);
    let flags: MessageFlags = serde_json::from_value(json!(32768 | 64)).unwrap();
    assert!(flags.contains(MessageFlags::IS_COMPONENTS_V2 | MessageFlags::EPHEMERAL));
    let out = serde_json::to_value(InteractionResponseFlags::IS_COMPONENTS_V2 | InteractionResponseFlags::EPHEMERAL).unwrap();
    assert_eq!(out, 32832);
}

#[test]
fn create_attachment_id_setter() {
    let a = CreateAttachment::bytes(vec![1, 2, 3], "a.png");
    assert_eq!(serde_json::to_value(&a).unwrap()["id"], 0);
    let b = CreateAttachment::bytes(vec![4], "b.png").id(1).description("second");
    assert_eq!(serde_json::to_value(&b).unwrap(), json!({"id": 1, "filename": "b.png", "description": "second"}));
}
