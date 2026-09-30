//! Ori CV2 fork (F1): the differential corpus test.
//!
//! Every document is parsed with stock serenity 0.12.5 (crates.io, renamed `serenity_stock`) and
//! with the fork (`serenity`, the parent directory):
//! - if stock parses it, the fork must parse it too, and `serde_json::to_value` of the two typed
//!   structs must be equal, except that the fork keeps bit 15 (`IS_COMPONENTS_V2`) of message
//!   flags, which stock truncates;
//! - if stock fails (or demotes a gateway event to `Event::Unknown`), the fork may succeed only
//!   for the listed malformed cases (`Expect::ForkRecovers`), and must fail otherwise;
//! - the fork's raw fields (`raw_components`, `raw_data`) equal the input.
//!
//! The corpus: the shared fixtures (`../tests/cv2_fixtures`: the probe's A, B, C and M cases,
//! every form field and select kind, ordinary traffic and malformed variants), every message,
//! interaction and gateway-event example extracted from the Discord API docs
//! (`corpus/discord-docs.json`), and serenity's own test JSON. A coverage checklist fails the test
//! if a category has no document.

#[path = "../../tests/cv2_fixtures/mod.rs"]
mod fixtures;

use std::collections::BTreeMap;

use fixtures::{Case, Expect, Kind};
use serde_json::{json, Value};

const V2_BIT: u64 = 1 << 15;

// ---------------------------------------------------------------------------------------------
// Parsing with either crate
// ---------------------------------------------------------------------------------------------

#[derive(Debug)]
#[allow(dead_code)] // The payloads are read through `Debug` in failure messages.
enum Parsed {
    /// `serde_json::to_value` of the typed result.
    Ok(Value),
    /// A gateway frame whose event was demoted to `Event::Unknown` (stock's silent drop).
    Demoted(String),
    Err(String),
}

macro_rules! parser {
    ($name:ident, $krate:ident) => {
        fn $name(kind: Kind, json: &Value) -> Parsed {
            use $krate::all::{Event, GatewayEvent, Interaction, Message, MessageUpdateEvent};

            fn out<T: serde::Serialize>(r: Result<T, impl ToString>) -> Parsed {
                match r {
                    Ok(t) => Parsed::Ok(serde_json::to_value(&t).expect("serializes")),
                    Err(e) => Parsed::Err(e.to_string()),
                }
            }

            match kind {
                Kind::Message => out(serde_json::from_value::<Message>(json.clone())),
                Kind::Interaction => out(serde_json::from_value::<Interaction>(json.clone())),
                // `to_value` cannot tell `Some(None)` from `None`, so the update is also applied
                // to a message whose fields are all set, as the cache does.
                Kind::MessageUpdate => match serde_json::from_value::<MessageUpdateEvent>(json.clone()) {
                    Ok(ev) => {
                        let mut msg: Message = serde_json::from_value(apply_base()).expect("base");
                        ev.apply_to_message(&mut msg);
                        Parsed::Ok(json!({
                            "event": serde_json::to_value(&ev).expect("serializes"),
                            "applied": serde_json::to_value(&msg).expect("serializes"),
                        }))
                    },
                    Err(e) => Parsed::Err(e.to_string()),
                },
                // As the shard does it (`gateway/ws.rs`): from the frame's text.
                Kind::Gateway => match $krate::json::from_str::<GatewayEvent>(json.to_string()) {
                    Ok(GatewayEvent::Dispatch(_, Event::Unknown(u))) => Parsed::Demoted(u.kind),
                    r => out(r),
                },
            }
        }
    };
}

/// A message with every field that a `MessageUpdateEvent` can clear set to a value.
fn apply_base() -> Value {
    fixtures::plain_message(json!({
        "webhook_id": fixtures::BOT, "application_id": fixtures::BOT, "flags": 64, "position": 3,
        "activity": {"type": 1, "party_id": "p"},
        "application": {"id": fixtures::BOT, "name": "Ori", "description": "", "icon": null, "cover_image": null},
        "message_reference": {"type": 0, "channel_id": fixtures::CHANNEL, "message_id": fixtures::MSG},
        "referenced_message": fixtures::plain_message(json!({})),
        "interaction": {"id": "1422000000000001001", "type": 2, "name": "x", "user": fixtures::user(fixtures::MEMBER, "m", false)},
        "interaction_metadata": {"id": "1422000000000001001", "type": 2, "user": fixtures::user(fixtures::MEMBER, "m", false),
                                 "authorizing_integration_owners": {}},
        "role_subscription_data": {"role_subscription_listing_id": "1", "tier_name": "t", "total_months_subscribed": 1, "is_renewal": false},
        "member": fixtures::partial_member(),
        "components": [fixtures::good_row()]
    }))
}

parser!(parse_stock, serenity_stock);
parser!(parse_fork, serenity);

// ---------------------------------------------------------------------------------------------
// The fork's raw fields
// ---------------------------------------------------------------------------------------------

mod raw {
    use serde_json::Value;
    use serenity::all::{Event, GatewayEvent, Interaction, Message, MessageUpdateEvent};

    use super::Kind;

    fn message(msg: &Message, input: &Value) -> Result<(), String> {
        if msg.raw_components.as_ref() != input.get("components") {
            return Err("Message::raw_components differs from the input".into());
        }
        if let (Some(m), Some(raw)) = (&msg.referenced_message, input.get("referenced_message")) {
            message(m, raw)?;
        }
        let raw_snapshots = input.get("message_snapshots").and_then(Value::as_array);
        for (i, snapshot) in msg.message_snapshots.iter().enumerate() {
            let raw = &raw_snapshots.ok_or("snapshots without input")?[i]["message"];
            if snapshot.raw_components.as_ref() != raw.get("components") {
                return Err("MessageSnapshot::raw_components differs from the input".into());
            }
        }
        Ok(())
    }

    fn interaction(i: &Interaction, input: &Value) -> Result<(), String> {
        match i {
            Interaction::Component(c) => {
                if c.data.raw_data != input["data"] {
                    return Err("ComponentInteractionData::raw_data differs from the input".into());
                }
                message(&c.message, &input["message"])
            },
            Interaction::Modal(m) => {
                if m.data.raw_data != input["data"] {
                    return Err("ModalInteractionData::raw_data differs from the input".into());
                }
                m.message.as_ref().map_or(Ok(()), |msg| message(msg, &input["message"]))
            },
            _ => Ok(()),
        }
    }

    pub fn check(kind: Kind, input: &Value) -> Result<(), String> {
        let e = |e: serde_json::Error| e.to_string();
        match kind {
            Kind::Message => message(&serde_json::from_value(input.clone()).map_err(e)?, input),
            Kind::Interaction => {
                interaction(&serde_json::from_value(input.clone()).map_err(e)?, input)
            },
            Kind::MessageUpdate => {
                let ev: MessageUpdateEvent = serde_json::from_value(input.clone()).map_err(e)?;
                if ev.raw_components.as_ref() == input.get("components") {
                    Ok(())
                } else {
                    Err("MessageUpdateEvent::raw_components differs from the input".into())
                }
            },
            Kind::Gateway => {
                let ev: GatewayEvent =
                    serenity::json::from_str(input.to_string()).map_err(|e| e.to_string())?;
                let GatewayEvent::Dispatch(_, event) = ev else { return Ok(()) };
                let d = &input["d"];
                match event {
                    Event::MessageCreate(ev) => message(&ev.message, d),
                    Event::MessageUpdate(ev) if ev.raw_components.as_ref() != d.get("components") => {
                        Err("MessageUpdateEvent::raw_components differs from the input".into())
                    },
                    Event::InteractionCreate(ev) => interaction(&ev.interaction, d),
                    _ => Ok(()),
                }
            },
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Comparison
// ---------------------------------------------------------------------------------------------

/// Compares stock's and the fork's typed output. A `flags` value where the fork has exactly
/// stock's bits plus bit 15 counts as equal, and is counted in `flag_bits`.
fn compare(stock: &Value, fork: &Value, path: &str, diffs: &mut Vec<String>, flag_bits: &mut usize) {
    match (stock, fork) {
        (Value::Object(s), Value::Object(f)) => {
            for key in s.keys().chain(f.keys().filter(|k| !s.contains_key(*k))) {
                let p = format!("{path}.{key}");
                match (s.get(key), f.get(key)) {
                    (Some(a), Some(b)) => {
                        if key == "flags" {
                            if let (Some(a), Some(b)) = (a.as_u64(), b.as_u64()) {
                                if a != b && b == a | V2_BIT && a & V2_BIT == 0 {
                                    *flag_bits += 1;
                                    continue;
                                }
                            }
                        }
                        compare(a, b, &p, diffs, flag_bits);
                    },
                    (a, b) => diffs.push(format!("{p}: stock {a:?} / fork {b:?}")),
                }
            }
        },
        (Value::Array(s), Value::Array(f)) if s.len() == f.len() => {
            for (i, (a, b)) in s.iter().zip(f).enumerate() {
                compare(a, b, &format!("{path}[{i}]"), diffs, flag_bits);
            }
        },
        (a, b) if a == b => {},
        (a, b) => diffs.push(format!("{path}: stock {a} / fork {b}")),
    }
}

// ---------------------------------------------------------------------------------------------
// The corpus
// ---------------------------------------------------------------------------------------------

/// Examples from the Discord API docs, wrapped into full inbound documents. The docs give request
/// bodies and interaction `data` objects; the fixtures' envelopes supply the other fields.
fn docs_cases() -> Vec<Case> {
    let corpus: Value =
        serde_json::from_str(include_str!("../corpus/discord-docs.json")).expect("corpus");
    let mut out = Vec::new();
    for doc in corpus["examples"].as_array().unwrap() {
        let id = doc["id"].as_str().unwrap();
        let what = format!("{} / {}", doc["source"].as_str().unwrap(), doc["heading"].as_str().unwrap());
        let ex = &doc["example"];
        let mk = |suffix: &str, kind: Kind, json: Value, cats: &[&'static str]| Case {
            id: format!("{id}{suffix}"),
            description: what.clone(),
            kind,
            json,
            expect: Expect::Auto,
            categories: cats.to_vec(),
        };
        match doc["class"].as_str().unwrap() {
            "gateway" => out.push(mk("", Kind::Gateway, ex.clone(), &["gateway"])),
            "message" => out.push(mk("", Kind::Message, ex.clone(), &[])),
            "message-body" => {
                let mut m = fixtures::base_message(json!([]));
                for (k, v) in ex.as_object().unwrap() {
                    m[k] = v.clone();
                }
                if ex.get("flags").is_none() {
                    m["flags"] = json!(0);
                }
                let cats: &[&'static str] =
                    if ex.get("flags") == Some(&json!(32768)) { &["v2-layout"] } else { &[] };
                out.push(mk("", Kind::Message, m.clone(), cats));
                out.push(mk(
                    "-gw",
                    Kind::Gateway,
                    fixtures::gateway("MESSAGE_CREATE", 100, fixtures::gateway_message(m)),
                    &["gateway"],
                ));
            },
            "interaction-example" => match ex["type"].as_u64().unwrap() {
                3 => out.push(mk(
                    "",
                    Kind::Interaction,
                    fixtures::click(ex["data"].clone(), fixtures::v2_selects_message()),
                    &["clicks"],
                )),
                5 => out.push(mk(
                    "",
                    Kind::Interaction,
                    fixtures::interaction_common(5, ex["data"].clone(), None),
                    &["forms"],
                )),
                // A modal as sent (callback 9): its rows, parsed as a submit.
                9 => out.push(mk(
                    "",
                    Kind::Interaction,
                    fixtures::interaction_common(
                        5,
                        json!({"custom_id": ex["data"]["custom_id"], "components": ex["data"]["components"]}),
                        None,
                    ),
                    &["forms"],
                )),
                other => panic!("{id}: unexpected interaction example type {other}"),
            },
            "role" => out.push(mk(
                "",
                Kind::Interaction,
                fixtures::click(
                    json!({"custom_id": "pick:role", "component_type": 6, "values": [ex["id"]],
                           "resolved": {"roles": {ex["id"].as_str().unwrap(): ex}}}),
                    fixtures::v2_selects_message(),
                ),
                &["clicks"],
            )),
            other => panic!("{id}: unknown class {other}"),
        }
    }
    out
}

/// serenity 0.12.5's own test JSON (component, role and command option tests), wrapped.
fn serenity_own_cases() -> Vec<Case> {
    let buttons = json!([
        {"type": 2, "style": 4, "custom_id": "hello", "label": "a", "disabled": false},
        {"type": 2, "style": 5, "url": "https://google.com", "label": "a", "disabled": false},
        {"type": 2, "style": 6, "sku_id": "1234965026943668316", "label": "a", "disabled": false}
    ]);
    let mut tagged_role = fixtures::role(fixtures::ROLE, "Booster");
    tagged_role["tags"] = json!({"bot_id": null, "integration_id": null, "premium_subscriber": null, "subscription_listing_id": null});
    let options = json!([
        {"name": "boolean", "type": 5, "value": true},
        {"name": "integer", "type": 4, "value": 1},
        {"name": "number", "type": 10, "value": 2.0},
        {"name": "string", "type": 3, "value": "foobar"},
        {"name": "subcommand_group", "type": 2, "options": [
            {"name": "subcommand", "type": 1, "options": [{"name": "channel", "type": 7, "value": fixtures::CHANNEL}]}
        ]}
    ]);
    let case = |id: &str, kind, json| Case {
        id: id.into(),
        description: "serenity 0.12.5 test JSON".into(),
        kind,
        json,
        expect: Expect::Same,
        categories: vec![],
    };
    vec![
        case("S-buttons", Kind::Message, fixtures::base_message(json!([{"type": 1, "components": buttons}]))),
        case(
            "S-role-tags",
            Kind::Interaction,
            fixtures::click(
                json!({"custom_id": "pick:role", "component_type": 6, "values": [fixtures::ROLE],
                       "resolved": {"roles": {fixtures::ROLE: tagged_role}}}),
                fixtures::v2_selects_message(),
            ),
        ),
        case(
            "S-command-options",
            Kind::Interaction,
            fixtures::interaction_common(
                2,
                json!({"id": "1100000000000000123", "name": "welcome", "type": 1, "options": options,
                       "resolved": {"channels": {fixtures::CHANNEL: fixtures::partial_channel()}}}),
                None,
            ),
        ),
    ]
}

// ---------------------------------------------------------------------------------------------
// The test
// ---------------------------------------------------------------------------------------------

#[test]
fn differential_corpus() {
    let fixture_cases = fixtures::all_cases();
    let docs = docs_cases();
    let own = serenity_own_cases();
    let counts = (fixture_cases.len(), docs.len(), own.len());
    let cases: Vec<Case> = fixture_cases.into_iter().chain(docs).chain(own).collect();

    let mut failures = Vec::new();
    let mut coverage: BTreeMap<&str, usize> = BTreeMap::new();
    let mut outcome: BTreeMap<&str, usize> = BTreeMap::new();
    let mut flag_bits_total = 0;
    let mut ids = std::collections::HashSet::new();

    for case in &cases {
        assert!(ids.insert(case.id.clone()), "duplicate case id {}", case.id);
        for c in &case.categories {
            *coverage.entry(c).or_default() += 1;
        }
        let stock = parse_stock(case.kind, &case.json);
        let fork = parse_fork(case.kind, &case.json);
        let mut fail = |msg: String| failures.push(format!("{} ({}): {msg}", case.id, case.description));

        // The fork's raw fields always equal the input when it parses.
        if let Parsed::Ok(_) = fork {
            if let Err(e) = raw::check(case.kind, &case.json) {
                fail(format!("raw check: {e}"));
            }
        }

        match (&case.expect, &stock, &fork) {
            (Expect::Same, Parsed::Ok(s), Parsed::Ok(f)) => {
                let (mut diffs, mut bits) = (Vec::new(), 0);
                compare(s, f, "$", &mut diffs, &mut bits);
                flag_bits_total += bits;
                if diffs.is_empty() {
                    *outcome.entry("both parse, typed output equal").or_default() += 1;
                } else {
                    fail(format!("typed output differs:\n    {}", diffs.join("\n    ")));
                }
            },
            (Expect::Same, s, f) => fail(format!("expected both to parse: stock {s:?} / fork {f:?}")),

            (Expect::ForkRecovers(equiv), Parsed::Err(_) | Parsed::Demoted(_), Parsed::Ok(f)) => {
                *outcome.entry("stock fails or drops, fork recovers (listed)").or_default() += 1;
                if let Some(equiv) = equiv {
                    match parse_stock(case.kind, equiv) {
                        Parsed::Ok(s) => {
                            let (mut diffs, mut bits) = (Vec::new(), 0);
                            compare(&s, f, "$", &mut diffs, &mut bits);
                            flag_bits_total += bits;
                            if !diffs.is_empty() {
                                fail(format!(
                                    "fork output differs from stock on the sanitized document:\n    {}",
                                    diffs.join("\n    ")
                                ));
                            }
                        },
                        other => fail(format!("stock must parse the sanitized document: {other:?}")),
                    }
                }
            },
            (Expect::ForkRecovers(_), s, f) => {
                fail(format!("expected stock to fail and the fork to parse: stock {s:?} / fork {f:?}"))
            },

            (Expect::BothFail, Parsed::Err(_), Parsed::Err(_)) => {
                *outcome.entry("both fail").or_default() += 1;
            },
            (Expect::BothFail, s, f) => fail(format!("expected both to fail: stock {s:?} / fork {f:?}")),

            (Expect::Auto, Parsed::Ok(s), Parsed::Ok(f)) => {
                let (mut diffs, mut bits) = (Vec::new(), 0);
                compare(s, f, "$", &mut diffs, &mut bits);
                flag_bits_total += bits;
                if diffs.is_empty() {
                    *outcome.entry("both parse, typed output equal").or_default() += 1;
                } else {
                    fail(format!("typed output differs:\n    {}", diffs.join("\n    ")));
                }
            },
            (Expect::Auto, Parsed::Err(_), Parsed::Err(_)) => {
                *outcome.entry("both fail").or_default() += 1;
            },
            (Expect::Auto, Parsed::Demoted(a), Parsed::Demoted(b)) if a == b => {
                *outcome.entry("both demote to Event::Unknown (unknown event name)").or_default() += 1;
            },
            (Expect::Auto, s, f) => fail(format!("outcomes differ: stock {s:?} / fork {f:?}")),
        }
    }

    // Coverage checklist.
    for category in fixtures::CATEGORIES {
        if coverage.get(category).copied().unwrap_or(0) == 0 {
            failures.push(format!("coverage: no document in category {category}"));
        }
    }

    println!(
        "differential corpus: {} documents ({} fixtures, {} Discord docs, {} serenity tests)",
        cases.len(),
        counts.0,
        counts.1,
        counts.2
    );
    for (k, v) in &outcome {
        println!("  {k}: {v}");
    }
    println!("  flags values where only the fork keeps bit 15: {flag_bits_total}");
    println!("  coverage: {coverage:?}");
    assert!(failures.is_empty(), "{} failures:\n{}", failures.len(), failures.join("\n"));
}

/// Every listed recovery really is a failure (or silent drop) on stock: guards the list itself.
#[test]
fn recoveries_are_real_stock_failures() {
    for case in fixtures::all_cases() {
        if let Expect::ForkRecovers(_) = case.expect {
            assert!(
                !matches!(parse_stock(case.kind, &case.json), Parsed::Ok(_)),
                "{} parses on stock; it is not a recovery",
                case.id
            );
        }
    }
}
