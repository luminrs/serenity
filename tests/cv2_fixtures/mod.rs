//! Ori CV2 fork (F1): payload fixtures shared by `tests/cv2_raw.rs` and the differential corpus
//! test in `cv2-tests/`.
//!
//! The A, B, C and M cases are the Components V2 probe fixtures (ori-bot plan, "probe-output"),
//! shaped after the Discord components reference and after what the API returns for a Components
//! V2 message (ids populated, media resolved). The other cases add every select kind, every form
//! field kind, ordinary message traffic for the coverage checklist, and hand-made malformed
//! variants that prove the lenient skips. Only `serde_json` is used, so both crates can include
//! this file with `#[path]`.
#![allow(dead_code)]

use serde_json::{json, Value};

pub const GUILD: &str = "1100000000000000001";
pub const CHANNEL: &str = "1100000000000000333";
pub const BOT: &str = "1100000000000000999";
pub const MEMBER: &str = "1100000000000002222";
pub const MSG: &str = "1422000000000000777";
pub const ATT: &str = "1422000000000000778";
pub const ROLE: &str = "1100000000000000555";
pub const ROLE2: &str = "1100000000000000556";

/// What a document is parsed as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// `Message`, as returned by REST.
    Message,
    /// `Interaction`, as received by an interactions endpoint or inside `INTERACTION_CREATE`.
    Interaction,
    /// `MessageUpdateEvent`, the `d` of `MESSAGE_UPDATE`.
    MessageUpdate,
    /// A raw gateway frame, parsed as `GatewayEvent` the way the shard does.
    Gateway,
}

/// What stock 0.12.5 and the fork must do with a document.
#[derive(Clone, Debug, PartialEq)]
pub enum Expect {
    /// Both parse, and the typed output is equal (apart from bit 15 of message flags).
    Same,
    /// Stock fails (or, for gateway frames, demotes the event to `Event::Unknown`); the fork
    /// parses. If a value is given, the fork's typed output must equal stock's typed output for
    /// that value, which is the same document with the malformed part removed by hand.
    ForkRecovers(Option<Value>),
    /// Both fail.
    BothFail,
    /// The same outcome as stock, whatever it is: both parse with equal typed output, both fail,
    /// or both demote the gateway event to `Event::Unknown`. Used for the Discord docs examples,
    /// which include outbound-only gateway payloads.
    Auto,
}

#[derive(Clone, Debug)]
pub struct Case {
    pub id: String,
    pub description: String,
    pub kind: Kind,
    pub json: Value,
    pub expect: Expect,
    /// Coverage checklist categories (see `CATEGORIES`).
    pub categories: Vec<&'static str>,
}

/// Every category must have at least one document in the corpus.
pub const CATEGORIES: &[&str] = &[
    "v2-layout",
    "embeds",
    "polls",
    "stickers",
    "forwards",
    "replies",
    "thread-starters",
    "interaction-metadata",
    "attachment-flags",
    "forms",
    "clicks",
    "gateway",
    "malformed",
];

fn case(
    id: &str,
    description: &str,
    kind: Kind,
    json: Value,
    expect: Expect,
    categories: &[&'static str],
) -> Case {
    Case {
        id: id.to_string(),
        description: description.to_string(),
        kind,
        json,
        expect,
        categories: categories.to_vec(),
    }
}

// ---------------------------------------------------------------------------------------------
// Building blocks (from the probe)
// ---------------------------------------------------------------------------------------------

pub fn user(id: &str, name: &str, bot: bool) -> Value {
    json!({
        "id": id, "username": name, "discriminator": "0", "global_name": name,
        "avatar": null, "bot": bot, "public_flags": 0, "avatar_decoration_data": null,
        "clan": null, "primary_guild": null, "collectibles": null, "banner": null, "accent_color": null
    })
}

pub fn member(id: &str, name: &str) -> Value {
    json!({
        "user": user(id, name, false),
        "roles": [], "joined_at": "2026-09-29T15:04:05.123000+00:00", "deaf": false, "mute": false,
        "flags": 0, "pending": false, "permissions": "2248473465835073", "nick": null, "avatar": null,
        "banner": null, "premium_since": null, "communication_disabled_until": null,
        "unusual_dm_activity_until": null
    })
}

pub fn attachment() -> Value {
    json!({
        "id": ATT, "filename": "welcome.png", "size": 183422,
        "url": format!("https://cdn.discordapp.com/attachments/{CHANNEL}/{ATT}/welcome.png?ex=66f9&is=66f7&hm=abc&"),
        "proxy_url": format!("https://media.discordapp.net/attachments/{CHANNEL}/{ATT}/welcome.png?ex=66f9&is=66f7&hm=abc&"),
        "width": 1100, "height": 500, "content_type": "image/png",
        "placeholder": "ZvcJDQJ4h3eIeHiHh3eIiHd4Z4Z4hwgAAAAA", "placeholder_version": 1,
        "content_scan_version": 2, "flags": 0
    })
}

/// Unfurled media item as the API returns it for `attachment://welcome.png`.
pub fn media_item() -> Value {
    json!({
        "id": ATT,
        "url": format!("https://cdn.discordapp.com/attachments/{CHANNEL}/{ATT}/welcome.png?ex=66f9&is=66f7&hm=abc&"),
        "proxy_url": format!("https://media.discordapp.net/attachments/{CHANNEL}/{ATT}/welcome.png?ex=66f9&is=66f7&hm=abc&"),
        "width": 1100, "height": 500, "content_type": "image/png",
        "placeholder": "ZvcJDQJ4h3eIeHiHh3eIiHd4Z4Z4hwgAAAAA", "placeholder_version": 1,
        "loading_state": 2, "flags": 0, "attachment_id": ATT, "content_scan_metadata": {"version": 2, "flags": 0}
    })
}

pub fn role(id: &str, name: &str) -> Value {
    json!({
        "id": id, "name": name, "color": 7419530,
        "colors": {"primary_color": 7419530, "secondary_color": null, "tertiary_color": null},
        "hoist": true, "icon": null, "unicode_emoji": null, "position": 3,
        "permissions": "2249596494938111", "managed": false, "mentionable": true,
        "description": null, "flags": 0
    })
}

pub fn partial_channel() -> Value {
    json!({"id": CHANNEL, "name": "welcome", "type": 0, "permissions": "2248473465835073",
           "parent_id": null, "last_message_id": MSG, "nsfw": false, "position": 3,
           "rate_limit_per_user": 0, "topic": null, "flags": 0, "guild_id": GUILD})
}

pub fn partial_member() -> Value {
    json!({"roles": [], "joined_at": "2026-09-29T15:04:05.123000+00:00", "flags": 0,
           "pending": false, "permissions": "2248473465835073", "nick": null, "avatar": null,
           "banner": null, "premium_since": null, "communication_disabled_until": null})
}

pub fn link_row(id_base: u64, with_custom: bool) -> Value {
    let mut buttons = vec![
        json!({"type": 2, "id": id_base + 1, "style": 5, "label": "Pick your roles",
               "url": format!("https://discord.com/channels/{GUILD}/1100000000000000444")}),
        json!({"type": 2, "id": id_base + 2, "style": 5, "label": "Read the rules",
               "url": format!("https://discord.com/channels/{GUILD}/1100000000000000555")}),
    ];
    if with_custom {
        buttons.push(json!({"type": 2, "id": id_base + 3, "style": 2, "label": "Say hi",
                            "custom_id": "welcome:hi:1100000000000002222"}));
    }
    json!({"type": 1, "id": id_base, "components": buttons})
}

/// The M2 welcome: one Container with greeting text, media gallery (card) and a row of link buttons.
pub fn welcome_container(with_custom: bool) -> Value {
    json!({
        "type": 17, "id": 1, "accent_color": 5793266, "spoiler": false,
        "components": [
            {"type": 10, "id": 2, "content": format!("## Welcome to Ori HQ, <@{MEMBER}>!\nYou're member #1,204.")},
            {"type": 12, "id": 3, "items": [
                {"media": media_item(), "description": "Welcome card for newmember", "spoiler": false}
            ]},
            link_row(4, with_custom)
        ]
    })
}

pub fn base_message(components: Value) -> Value {
    json!({
        "id": MSG, "channel_id": CHANNEL, "guild_id": GUILD,
        "author": user(BOT, "Ori", true),
        "content": "", "timestamp": "2026-09-29T15:04:05.123000+00:00", "edited_timestamp": null,
        "tts": false, "mention_everyone": false, "mentions": [], "mention_roles": [],
        "attachments": [attachment()], "embeds": [], "pinned": false, "type": 0,
        "flags": 32768, "components": components, "application_id": BOT, "webhook_id": null
    })
}

/// A message as Discord returns it from POST /channels/{id}/messages for the M2 welcome.
pub fn v2_welcome_message() -> Value {
    base_message(json!([welcome_container(false)]))
}

pub fn v2_welcome_message_custom() -> Value {
    base_message(json!([welcome_container(true)]))
}

/// Container followed by a top-level (legacy) action row.
pub fn v2_container_plus_top_row() -> Value {
    base_message(json!([welcome_container(false), link_row(20, true)]))
}

/// Every V2 layout/content type at top level: section+thumbnail, section+button accessory,
/// separator, file, text display, media gallery.
pub fn v2_kitchen_sink() -> Value {
    base_message(json!([
        {"type": 9, "id": 1, "components": [
            {"type": 10, "id": 2, "content": "# Real Game v7.3"},
            {"type": 10, "id": 3, "content": "Patch notes"}
        ], "accessory": {"type": 11, "id": 4, "media": media_item(), "description": null, "spoiler": false}},
        {"type": 9, "id": 5, "components": [{"type": 10, "id": 6, "content": "Get roles"}],
         "accessory": {"type": 2, "id": 7, "style": 1, "label": "Roles", "custom_id": "roles:open"}},
        {"type": 14, "id": 8, "divider": true, "spacing": 1},
        {"type": 13, "id": 9, "file": media_item(), "spoiler": false, "name": "welcome.png", "size": 183422},
        {"type": 10, "id": 10, "content": "Standalone text"},
        {"type": 12, "id": 11, "items": [{"media": media_item(), "description": null, "spoiler": false}]}
    ]))
}

/// Every select kind in rows inside a container, plus a top-level row of the same selects.
pub fn v2_selects_message() -> Value {
    let selects = json!([
        {"type": 1, "id": 21, "components": [{"type": 3, "id": 22, "custom_id": "pick:string",
            "options": [{"label": "Classic", "value": "classic", "description": null, "emoji": null, "default": false}],
            "placeholder": "Style", "min_values": 1, "max_values": 1, "disabled": false}]},
        {"type": 1, "id": 23, "components": [{"type": 5, "id": 24, "custom_id": "pick:user", "min_values": 1, "max_values": 1}]},
        {"type": 1, "id": 25, "components": [{"type": 6, "id": 26, "custom_id": "pick:role", "min_values": 1, "max_values": 2}]},
        {"type": 1, "id": 27, "components": [{"type": 7, "id": 28, "custom_id": "pick:mentionable"}]},
        {"type": 1, "id": 29, "components": [{"type": 8, "id": 30, "custom_id": "pick:channel", "channel_types": [0, 5]}]}
    ]);
    let mut container_children = vec![json!({"type": 10, "id": 20, "content": "Settings"})];
    container_children.extend(selects.as_array().unwrap().iter().cloned());
    let mut top = vec![json!({"type": 17, "id": 19, "components": container_children})];
    top.push(json!({"type": 1, "id": 40, "components": [
        {"type": 2, "id": 41, "style": 1, "label": "Save", "custom_id": "pick:save"}
    ]}));
    base_message(Value::Array(top))
}

/// Synthetic: an unknown/new component type INSIDE a top-level action row.
pub fn bad_row_message() -> Value {
    base_message(json!([
        {"type": 1, "id": 1, "components": [{"type": 99, "id": 2, "custom_id": "future"}]}
    ]))
}

/// Synthetic: a text display inside a top-level action row (invalid for Discord).
pub fn text_in_row_message() -> Value {
    base_message(json!([
        {"type": 1, "id": 1, "components": [{"type": 10, "id": 2, "content": "hi"}]}
    ]))
}

/// A top-level row whose children are one good button and one malformed child (`bad`).
pub fn row_with(bad: Value) -> Value {
    json!({"type": 1, "id": 30, "components": [
        {"type": 2, "id": 31, "style": 1, "label": "Keep", "custom_id": "keep"},
        bad
    ]})
}

pub fn good_row() -> Value {
    json!({"type": 1, "id": 30, "components": [
        {"type": 2, "id": 31, "style": 1, "label": "Keep", "custom_id": "keep"}
    ]})
}

/// Forward (message_snapshots) of a message.
pub fn forward_of(snap: &Value) -> Value {
    json!({
        "id": "1422000000000000900", "channel_id": CHANNEL, "guild_id": GUILD,
        "author": user(MEMBER, "newmember", false),
        "content": "", "timestamp": "2026-09-29T15:05:05.123000+00:00", "edited_timestamp": null,
        "tts": false, "mention_everyone": false, "mentions": [], "mention_roles": [],
        "attachments": [], "embeds": [], "pinned": false, "type": 0, "flags": 16384, "components": [],
        "message_reference": {"type": 1, "channel_id": CHANNEL, "message_id": MSG, "guild_id": GUILD},
        "message_snapshots": [{"message": {
            "content": snap["content"], "timestamp": snap["timestamp"], "edited_timestamp": null, "mentions": [],
            "mention_roles": [], "attachments": snap["attachments"], "embeds": snap["embeds"], "type": 0,
            "flags": snap["flags"], "components": snap["components"], "sticker_items": []
        }}]
    })
}

/// Forward (message_snapshots) of the V2 welcome by a user.
pub fn forwarded_v2() -> Value {
    forward_of(&v2_welcome_message())
}

/// Reply to a message (referenced_message is that message).
pub fn reply_to(referenced: Value) -> Value {
    json!({
        "id": "1422000000000000901", "channel_id": CHANNEL, "guild_id": GUILD,
        "author": user(MEMBER, "newmember", false),
        "content": "thanks!", "timestamp": "2026-09-29T15:05:05.123000+00:00", "edited_timestamp": null,
        "tts": false, "mention_everyone": false, "mentions": [user(BOT, "Ori", true)], "mention_roles": [],
        "attachments": [], "embeds": [], "pinned": false, "type": 19, "flags": 0, "components": [],
        "message_reference": {"type": 0, "channel_id": CHANNEL, "message_id": MSG, "guild_id": GUILD},
        "referenced_message": referenced
    })
}

/// Reply to the V2 welcome (referenced_message is the V2 message).
pub fn reply_to_v2() -> Value {
    reply_to(v2_welcome_message())
}

pub fn interaction_common(kind: u8, data: Value, message: Option<Value>) -> Value {
    let mut v = json!({
        "id": "1422000000000001000", "application_id": BOT, "type": kind, "data": data,
        "guild_id": GUILD,
        "channel": {"id": CHANNEL, "type": 0, "name": "welcome", "guild_id": GUILD, "flags": 0,
                    "permissions": "2248473465835073", "parent_id": null, "position": 3, "nsfw": false,
                    "topic": null, "last_message_id": MSG, "rate_limit_per_user": 0},
        "channel_id": CHANNEL, "member": member(MEMBER, "newmember"),
        "token": "aW50ZXJhY3Rpb246MTQyMjAwMDAwMDAwMDAwMTAwMDpmYWtl", "version": 1,
        "app_permissions": "2248473465835073", "locale": "en-US", "guild_locale": "en-US",
        "entitlements": [], "authorizing_integration_owners": {"0": GUILD}, "context": 0,
        "attachment_size_limit": 10485760
    });
    if let Some(m) = message {
        v["message"] = m;
    }
    v
}

pub fn click(data: Value, message: Value) -> Value {
    interaction_common(3, data, Some(message))
}

/// Button click (legacy button inside an action row inside the container) on the V2 welcome.
pub fn button_click_on_v2() -> Value {
    click(
        json!({"custom_id": "welcome:hi:1100000000000002222", "component_type": 2, "id": 7}),
        v2_welcome_message_custom(),
    )
}

pub fn button_click_on_v2_top_row() -> Value {
    click(
        json!({"custom_id": "welcome:hi:1100000000000002222", "component_type": 2, "id": 23}),
        v2_container_plus_top_row(),
    )
}

pub fn button_click_on_section_accessory() -> Value {
    click(json!({"custom_id": "roles:open", "component_type": 2, "id": 7}), v2_kitchen_sink())
}

pub fn button_click_on_bad_row() -> Value {
    click(json!({"custom_id": "welcome:hi", "component_type": 2, "id": 2}), bad_row_message())
}

/// A modal submit with the given `data.components` and optional `data.resolved`.
pub fn modal(components: Value, resolved: Option<Value>) -> Value {
    let mut data = json!({"custom_id": "welcome:edit", "components": components});
    if let Some(r) = resolved {
        data["resolved"] = r;
    }
    interaction_common(5, data, None)
}

/// Legacy modal submit: action row + text input.
pub fn modal_legacy() -> Value {
    modal(
        json!([{"type": 1, "id": 1, "components": [
            {"type": 4, "id": 2, "custom_id": "greeting", "value": "Welcome {user}!"}
        ]}]),
        None,
    )
}

/// New modal submit: Label (18) wrapping a text input and a string select, plus a Text Display.
pub fn modal_label() -> Value {
    modal(
        json!([
            {"type": 10, "id": 1},
            {"type": 18, "id": 2, "component": {"type": 4, "id": 3, "custom_id": "greeting", "value": "Welcome {user}!"}},
            {"type": 18, "id": 4, "component": {"type": 3, "id": 5, "custom_id": "style", "values": ["classic"]}}
        ]),
        None,
    )
}

pub fn label(id: u64, component: Value) -> Value {
    json!({"type": 18, "id": id, "component": component})
}

pub fn gateway(t: &str, s: u64, d: Value) -> Value {
    json!({"op": 0, "s": s, "t": t, "d": d})
}

pub fn gateway_message(msg: Value) -> Value {
    let mut m = msg;
    m["member"] = json!({"roles": [], "joined_at": "2026-09-01T00:00:00+00:00", "deaf": false, "mute": false,
                          "flags": 0, "pending": false, "premium_since": null, "nick": null, "avatar": null,
                          "banner": null, "communication_disabled_until": null});
    m["nonce"] = json!(null);
    m
}

/// A plain (non-V2) message with the given extra fields merged in.
pub fn plain_message(extra: Value) -> Value {
    let mut m = json!({
        "id": "1422000000000000950", "channel_id": CHANNEL, "guild_id": GUILD,
        "author": user(MEMBER, "newmember", false),
        "content": "hello", "timestamp": "2026-09-29T16:00:00.000000+00:00", "edited_timestamp": null,
        "tts": false, "mention_everyone": false, "mentions": [], "mention_roles": [],
        "attachments": [], "embeds": [], "pinned": false, "type": 0, "flags": 0, "components": []
    });
    for (k, v) in extra.as_object().unwrap() {
        m[k] = v.clone();
    }
    m
}

// ---------------------------------------------------------------------------------------------
// The cases
// ---------------------------------------------------------------------------------------------

fn same(id: &str, d: &str, kind: Kind, json: Value, cats: &[&'static str]) -> Case {
    case(id, d, kind, json, Expect::Same, cats)
}

fn recovers(id: &str, d: &str, kind: Kind, json: Value, equiv: Option<Value>) -> Case {
    case(id, d, kind, json, Expect::ForkRecovers(equiv), &["malformed"])
}

/// A copy of `v` with `f` applied.
pub fn with(v: &Value, f: impl FnOnce(&mut Value)) -> Value {
    let mut v = v.clone();
    f(&mut v);
    v
}

pub fn probe_cases() -> Vec<Case> {
    use Kind::*;

    let label_text = label(2, json!({"type": 4, "id": 3, "custom_id": "greeting", "value": "Welcome {user}!"}));
    let label_select = label(4, json!({"type": 3, "id": 5, "custom_id": "style", "values": ["classic"]}));
    let radio = label(1, json!({"type": 21, "id": 2, "custom_id": "tone", "value": "warm"}));

    vec![
        same("A1", "V2 welcome Container[Text, Gallery, Row[link,link]]", Message, v2_welcome_message(), &["v2-layout"]),
        same("A2", "A1 + custom_id button in container row", Message, v2_welcome_message_custom(), &["v2-layout"]),
        same("A3", "Container + top-level Row", Message, v2_container_plus_top_row(), &["v2-layout"]),
        same("A4", "kitchen sink: Section+Thumb, Section+Button, Sep, File, Text, Gallery", Message, v2_kitchen_sink(), &["v2-layout"]),
        same("A5", "forward (message_snapshots) of V2 welcome", Message, forwarded_v2(), &["v2-layout", "forwards"]),
        same("A6", "reply whose referenced_message is V2", Message, reply_to_v2(), &["v2-layout", "replies"]),
        recovers("A7", "SYNTHETIC top-level Row containing type 99", Message, bad_row_message(),
            Some(base_message(json!([{"type": 1, "id": 1, "components": []}])))),
        recovers("A8", "SYNTHETIC top-level Row containing TextDisplay", Message, text_in_row_message(),
            Some(base_message(json!([{"type": 1, "id": 1, "components": []}])))),
        same("B1", "click: button in row inside container", Interaction, button_click_on_v2(), &["clicks", "v2-layout"]),
        same("B2", "click: button in top-level row of V2 msg", Interaction, button_click_on_v2_top_row(), &["clicks", "v2-layout"]),
        same("B3", "click: Section accessory button", Interaction, button_click_on_section_accessory(), &["clicks", "v2-layout"]),
        recovers("B4", "SYNTHETIC click on msg with type 99 in row", Interaction, button_click_on_bad_row(),
            Some(click(json!({"custom_id": "welcome:hi", "component_type": 2, "id": 2}),
                base_message(json!([{"type": 1, "id": 1, "components": []}]))))),
        same("B5", "modal: legacy Row[TextInput]", Interaction, modal_legacy(), &["forms"]),
        same("B6", "modal: TextDisplay + Label[TextInput] + Label[StringSelect]", Interaction, modal_label(), &["forms"]),
        same("M1", "modal: Label[TextInput] + Label[StringSelect] (no TextDisplay)", Interaction,
            modal(json!([label_text, label_select]), None), &["forms"]),
        same("M2", "modal: Label[UserSelect] + resolved.users", Interaction,
            modal(json!([label(1, json!({"type": 5, "id": 2, "custom_id": "who", "values": [MEMBER]}))]),
                Some(json!({"users": {MEMBER: user(MEMBER, "newmember", false)},
                            "members": {MEMBER: partial_member()}}))), &["forms"]),
        same("M3", "modal: Label[RadioGroup] value", Interaction, modal(json!([radio.clone()]), None), &["forms"]),
        same("M4", "modal: Label[CheckboxGroup] values", Interaction,
            modal(json!([label(1, json!({"type": 22, "id": 2, "custom_id": "opts", "values": ["a", "b"]}))]), None), &["forms"]),
        same("M5", "modal: Label[Checkbox] value", Interaction,
            modal(json!([label(1, json!({"type": 23, "id": 2, "custom_id": "ok", "value": true}))]), None), &["forms"]),
        same("M6", "modal: Label[FileUpload] + resolved.attachments", Interaction,
            modal(json!([label(1, json!({"type": 19, "id": 2, "custom_id": "img", "values": [ATT]}))]),
                Some(json!({"attachments": {ATT: attachment()}}))), &["forms"]),
        same("C1", "GW MESSAGE_CREATE V2 welcome", Gateway, gateway("MESSAGE_CREATE", 42, gateway_message(v2_welcome_message())), &["gateway", "v2-layout"]),
        same("C2", "GW MESSAGE_CREATE kitchen sink", Gateway, gateway("MESSAGE_CREATE", 43, gateway_message(v2_kitchen_sink())), &["gateway", "v2-layout"]),
        same("C3", "GW MESSAGE_UPDATE V2 welcome", Gateway, gateway("MESSAGE_UPDATE", 44, gateway_message(v2_welcome_message())), &["gateway", "v2-layout"]),
        same("C4", "GW INTERACTION_CREATE click on V2 welcome", Gateway, gateway("INTERACTION_CREATE", 45, button_click_on_v2()), &["gateway", "clicks"]),
        recovers("C5", "GW SYNTHETIC MESSAGE_CREATE type 99 in row", Gateway,
            gateway("MESSAGE_CREATE", 46, gateway_message(bad_row_message())), None),
        recovers("C6", "GW SYNTHETIC INTERACTION_CREATE click on that", Gateway,
            gateway("INTERACTION_CREATE", 47, button_click_on_bad_row()), None),
        same("C7", "GW INTERACTION_CREATE modal TextDisplay+Label", Gateway, gateway("INTERACTION_CREATE", 48, modal_label()), &["gateway", "forms"]),
        same("C8", "GW INTERACTION_CREATE modal Label[RadioGroup]", Gateway,
            gateway("INTERACTION_CREATE", 49, modal(json!([radio]), None)), &["gateway", "forms"]),
    ]
}

/// Every form field kind, and every select kind, inside Labels.
pub fn form_cases() -> Vec<Case> {
    use Kind::*;

    let text_display = json!({"type": 10, "id": 1});
    let fields = vec![
        ("F-text", label(2, json!({"type": 4, "id": 3, "custom_id": "t", "value": "hello"}))),
        ("F-string", label(2, json!({"type": 3, "id": 3, "custom_id": "s", "values": ["a", "b"]}))),
        ("F-user", label(2, json!({"type": 5, "id": 3, "custom_id": "u", "values": [MEMBER]}))),
        ("F-role", label(2, json!({"type": 6, "id": 3, "custom_id": "r", "values": [ROLE]}))),
        ("F-mentionable", label(2, json!({"type": 7, "id": 3, "custom_id": "m", "values": [ROLE, MEMBER]}))),
        ("F-channel", label(2, json!({"type": 8, "id": 3, "custom_id": "c", "values": [CHANNEL]}))),
        ("F-upload", label(2, json!({"type": 19, "id": 3, "custom_id": "f", "values": [ATT]}))),
        ("F-radio", label(2, json!({"type": 21, "id": 3, "custom_id": "g", "value": "warm"}))),
        ("F-radio-empty", label(2, json!({"type": 21, "id": 3, "custom_id": "g", "value": null}))),
        ("F-checkbox-group", label(2, json!({"type": 22, "id": 3, "custom_id": "cg", "values": []}))),
        ("F-checkbox", label(2, json!({"type": 23, "id": 3, "custom_id": "cb", "value": false}))),
        ("F-unknown", label(2, json!({"type": 99, "id": 3, "custom_id": "future", "value": {"x": 1}}))),
    ];
    let resolved = json!({
        "users": {MEMBER: user(MEMBER, "newmember", false)},
        "members": {MEMBER: partial_member()},
        "roles": {ROLE: role(ROLE, "Player")},
        "channels": {CHANNEL: partial_channel()},
        "attachments": {ATT: attachment()}
    });
    let mut out: Vec<Case> = fields
        .into_iter()
        .map(|(id, field)| {
            same(id, "modal: Text Display + one Label field", Interaction,
                modal(json!([text_display.clone(), field]), Some(resolved.clone())), &["forms"])
        })
        .collect();
    out.push(same("F-mixed", "modal: legacy row + Labels + text display", Interaction,
        modal(json!([
            {"type": 1, "id": 1, "components": [{"type": 4, "id": 2, "custom_id": "legacy", "value": "x"}]},
            text_display,
            label(3, json!({"type": 21, "id": 4, "custom_id": "g", "value": "cool"})),
            label(5, json!({"type": 19, "id": 6, "custom_id": "f", "values": [ATT]}))
        ]), Some(resolved)), &["forms"]));
    out
}

/// A click of every kind, on a V2 message.
pub fn click_cases() -> Vec<Case> {
    use Kind::*;

    let msg = v2_selects_message();
    let resolved_users = json!({"users": {MEMBER: user(MEMBER, "newmember", false)},
                                "members": {MEMBER: partial_member()}});
    let resolved_roles = json!({"roles": {ROLE: role(ROLE, "Player"), ROLE2: role(ROLE2, "Viewer")}});
    let resolved_channels = json!({"channels": {CHANNEL: partial_channel()}});
    let mut resolved_mentionable = resolved_users.clone();
    resolved_mentionable["roles"] = json!({ROLE: role(ROLE, "Player")});

    vec![
        same("K-button", "click: button on a V2 message", Interaction,
            click(json!({"custom_id": "pick:save", "component_type": 2, "id": 41}), msg.clone()), &["clicks"]),
        same("K-string", "click: string select in a container", Interaction,
            click(json!({"custom_id": "pick:string", "component_type": 3, "id": 22, "values": ["classic"]}), msg.clone()), &["clicks"]),
        same("K-user", "click: user select + resolved users/members", Interaction,
            click(json!({"custom_id": "pick:user", "component_type": 5, "id": 24, "values": [MEMBER],
                         "resolved": resolved_users}), msg.clone()), &["clicks"]),
        same("K-role", "click: role select + resolved roles", Interaction,
            click(json!({"custom_id": "pick:role", "component_type": 6, "id": 26, "values": [ROLE, ROLE2],
                         "resolved": resolved_roles.clone()}), msg.clone()), &["clicks"]),
        same("K-mentionable", "click: mentionable select + resolved", Interaction,
            click(json!({"custom_id": "pick:mentionable", "component_type": 7, "id": 28, "values": [MEMBER, ROLE],
                         "resolved": resolved_mentionable}), msg.clone()), &["clicks"]),
        same("K-channel", "click: channel select + resolved channels", Interaction,
            click(json!({"custom_id": "pick:channel", "component_type": 8, "id": 30, "values": [CHANNEL],
                         "resolved": resolved_channels}), msg.clone()), &["clicks"]),
        same("K-unknown", "click: unknown component type 30", Interaction,
            click(json!({"custom_id": "pick:future", "component_type": 30, "id": 50, "values": ["x"]}), msg.clone()), &["clicks"]),
        same("K-forwarded", "click on a message that forwards a V2 message", Interaction,
            click(json!({"custom_id": "keep", "component_type": 2}), forwarded_v2()), &["clicks", "forwards"]),
        same("K-gw-role", "GW INTERACTION_CREATE role select click", Gateway,
            gateway("INTERACTION_CREATE", 60, click(json!({"custom_id": "pick:role", "component_type": 6, "id": 26,
                "values": [ROLE], "resolved": {"roles": {ROLE: role(ROLE, "Player")}}}), msg)), &["gateway", "clicks"]),
    ]
}

/// Ordinary (non-V2) traffic for the coverage checklist, shaped after the Discord message docs.
pub fn ordinary_cases() -> Vec<Case> {
    use Kind::*;

    let embed = json!({
        "type": "rich", "title": "Patch notes", "description": "Fixed things", "url": "https://example.com/notes",
        "timestamp": "2026-09-29T16:00:00.000000+00:00", "color": 5793266,
        "footer": {"text": "Ori", "icon_url": "https://cdn.discordapp.com/embed/avatars/0.png"},
        "image": {"url": "https://example.com/i.png", "proxy_url": "https://images-ext-1.discordapp.net/i.png", "width": 640, "height": 360, "flags": 0},
        "thumbnail": {"url": "https://example.com/t.png", "width": 80, "height": 80},
        "author": {"name": "Ori", "url": "https://example.com", "icon_url": "https://example.com/a.png"},
        "fields": [{"name": "Level", "value": "7", "inline": true}, {"name": "XP", "value": "1,204", "inline": true}],
        "content_scan_version": 2
    });
    let link_embed = json!({"type": "link", "url": "https://example.com", "title": "Example",
        "provider": {"name": "Example"}, "description": "A link",
        "video": {"url": "https://example.com/v.mp4", "width": 1280, "height": 720}});
    let poll = json!({
        "question": {"text": "Best layout?"},
        "answers": [
            {"answer_id": 1, "poll_media": {"text": "Classic", "emoji": {"id": null, "name": "\u{1f3a8}"}}},
            {"answer_id": 2, "poll_media": {"text": "Minimal", "emoji": {"id": "1100000000000000777", "name": "ori", "animated": false}}}
        ],
        "expiry": "2026-10-01T16:00:00.000000+00:00", "allow_multiselect": false, "layout_type": 1,
        "results": {"is_finalized": false, "answer_counts": [{"id": 1, "count": 3, "me_voted": true}]}
    });
    let voice_attachment = json!({
        "id": "1422000000000000960", "filename": "voice-message.ogg", "size": 21000,
        "url": "https://cdn.discordapp.com/attachments/1/2/voice-message.ogg",
        "proxy_url": "https://media.discordapp.net/attachments/1/2/voice-message.ogg",
        "content_type": "audio/ogg", "duration_secs": 3.2, "waveform": "AAAAAAAAAA==", "flags": 8192
    });
    let mut flagged = attachment();
    flagged["flags"] = json!(4 | 8 | 32);
    flagged["ephemeral"] = json!(true);
    flagged["title"] = json!("welcome");
    let thread = json!({
        "id": "1422000000000000950", "guild_id": GUILD, "parent_id": CHANNEL, "owner_id": MEMBER,
        "type": 11, "name": "hello thread", "last_message_id": null, "rate_limit_per_user": 0,
        "flags": 0, "message_count": 0, "member_count": 1, "total_message_sent": 0,
        "thread_metadata": {"archived": false, "auto_archive_duration": 1440,
                            "archive_timestamp": "2026-09-29T16:00:00.000000+00:00", "locked": false,
                            "create_timestamp": "2026-09-29T16:00:00.000000+00:00"},
        "member": {"id": "1422000000000000950", "user_id": MEMBER, "join_timestamp": "2026-09-29T16:00:00.000000+00:00", "flags": 1}
    });
    let interaction_metadata = json!({
        "id": "1422000000000001001", "type": 2, "user": user(MEMBER, "newmember", false),
        "authorizing_integration_owners": {"0": GUILD}, "name": "welcome preview", "command_type": 1
    });
    let component_metadata = json!({
        "id": "1422000000000001002", "type": 3, "user": user(MEMBER, "newmember", false),
        "authorizing_integration_owners": {"0": GUILD}, "interacted_message_id": MSG
    });

    vec![
        same("O-embeds", "message with rich and link embeds", Message,
            plain_message(json!({"embeds": [embed.clone(), link_embed]})), &["embeds"]),
        same("O-embeds-v2-reply", "reply to a V2 message, with an embed", Message,
            with(&reply_to_v2(), |m| m["embeds"] = json!([embed.clone()])), &["embeds", "replies"]),
        same("O-poll", "message with a poll", Message, plain_message(json!({"poll": poll, "content": ""})), &["polls"]),
        same("O-stickers", "message with sticker items", Message,
            plain_message(json!({"sticker_items": [{"id": "1100000000000000888", "name": "wave", "format_type": 1}], "content": ""})),
            &["stickers"]),
        same("O-forward-plain", "forward of an ordinary message with an embed", Message,
            forward_of(&plain_message(json!({"embeds": [embed]}))), &["forwards"]),
        same("O-reply-plain", "reply to an ordinary message", Message, reply_to(plain_message(json!({}))), &["replies"]),
        same("O-thread-starter", "thread starter message (type 21)", Message,
            plain_message(json!({"type": 21, "content": "", "message_reference": {"type": 0, "channel_id": CHANNEL, "message_id": MSG, "guild_id": GUILD},
                                  "referenced_message": plain_message(json!({"flags": 32}))})), &["thread-starters"]),
        same("O-has-thread", "message that started a thread", Message,
            plain_message(json!({"flags": 32, "thread": thread})), &["thread-starters"]),
        same("O-interaction-metadata", "slash command response with interaction_metadata and interaction", Message,
            plain_message(json!({"author": user(BOT, "Ori", true), "application_id": BOT, "webhook_id": BOT, "type": 20,
                                  "interaction_metadata": interaction_metadata,
                                  "interaction": {"id": "1422000000000001001", "type": 2, "name": "welcome preview",
                                                  "user": user(MEMBER, "newmember", false)}})), &["interaction-metadata"]),
        same("O-component-metadata", "V2 update after a click, with component interaction_metadata", Message,
            with(&v2_welcome_message(), |m| m["interaction_metadata"] = component_metadata), &["interaction-metadata", "v2-layout"]),
        same("O-attachment-flags", "attachments with flags (spoiler/remix) and a voice message", Message,
            plain_message(json!({"attachments": [flagged, voice_attachment], "flags": 8192})), &["attachment-flags"]),
        same("O-gw-create-embed", "GW MESSAGE_CREATE with an embed and a poll", Gateway,
            gateway("MESSAGE_CREATE", 70, gateway_message(plain_message(json!({"poll": {
                "question": {"text": "?"}, "answers": [{"answer_id": 1, "poll_media": {"text": "yes"}}],
                "expiry": null, "allow_multiselect": true, "layout_type": 1}})))), &["gateway", "polls"]),
        same("O-gw-update-partial", "GW MESSAGE_UPDATE without components", Gateway,
            gateway("MESSAGE_UPDATE", 71, json!({"id": MSG, "channel_id": CHANNEL, "guild_id": GUILD,
                "content": "edited", "edited_timestamp": "2026-09-29T16:10:00.000000+00:00"})), &["gateway"]),
        same("O-update-partial", "MESSAGE_UPDATE d without components", MessageUpdate,
            json!({"id": MSG, "channel_id": CHANNEL, "content": "edited", "flags": 32768,
                   "edited_timestamp": "2026-09-29T16:10:00.000000+00:00"}), &[]),
        same("O-update-v2", "MESSAGE_UPDATE d of the V2 welcome", MessageUpdate, v2_welcome_message(), &["v2-layout"]),
        same("O-update-null-flags", "MESSAGE_UPDATE d with null flags and webhook", MessageUpdate,
            json!({"id": MSG, "channel_id": CHANNEL, "flags": null, "webhook_id": null, "components": []}), &[]),
        same("O-update-all-null", "MESSAGE_UPDATE d with every clearable field null", MessageUpdate,
            json!({"id": MSG, "channel_id": CHANNEL, "webhook_id": null, "activity": null, "application": null,
                   "application_id": null, "message_reference": null, "flags": null, "referenced_message": null,
                   "interaction": null, "interaction_metadata": null, "thread": null, "position": null,
                   "role_subscription_data": null, "member": null}), &[]),
        case("O-no-components", "message without a components key", Message,
            with(&plain_message(json!({})), |m| { m.as_object_mut().unwrap().remove("components"); }),
            Expect::Same, &[]),
        case("X-missing-id", "message without an id (both fail)", Message,
            with(&plain_message(json!({})), |m| { m.as_object_mut().unwrap().remove("id"); }),
            Expect::BothFail, &[]),
        case("X-null-components", "message with components: null (both fail, as in 0.12.5)", Message,
            plain_message(json!({"components": null})), Expect::BothFail, &[]),
        case("X-click-no-custom-id", "click without custom_id (both fail)", Interaction,
            click(json!({"component_type": 2}), plain_message(json!({}))), Expect::BothFail, &[]),
        case("X-select-no-values", "string select click without values (both fail)", Interaction,
            click(json!({"custom_id": "s", "component_type": 3}), plain_message(json!({}))), Expect::BothFail, &[]),
        case("X-modal-no-components", "modal submit without components (both fail)", Interaction,
            interaction_common(5, json!({"custom_id": "m"}), None), Expect::BothFail, &[]),
    ]
}

/// Hand-made malformed variants: the fork skips only the malformed part and keeps the event.
pub fn malformed_cases() -> Vec<Case> {
    use Kind::*;

    let bad_children = vec![
        ("R-button-no-style", json!({"type": 2, "id": 32, "label": "Broken", "custom_id": "broken"})),
        ("R-nested-row", json!({"type": 1, "id": 32, "components": []})),
        ("R-select-bad-options", json!({"type": 3, "id": 32, "custom_id": "s", "options": "none"})),
        ("R-child-no-type", json!({"id": 32, "custom_id": "x"})),
        ("R-child-not-object", json!("button")),
        ("R-section-in-row", json!({"type": 9, "id": 32, "components": []})),
    ];
    let mut out = Vec::new();
    for (id, bad) in bad_children {
        out.push(recovers(id, "row with one good child and one malformed child", Message,
            base_message(json!([welcome_container(true), row_with(bad)])),
            Some(base_message(json!([welcome_container(true), good_row()])))));
    }

    let top_level_bad = vec![
        ("T-row-components-string", json!({"type": 1, "id": 40, "components": "oops"})),
        ("T-no-type", json!({"id": 40, "components": []})),
        ("T-type-string", json!({"type": "1", "id": 40, "components": []})),
        ("T-not-object", json!(17)),
    ];
    for (id, bad) in top_level_bad {
        out.push(recovers(id, "malformed top-level entry beside a good row", Message,
            base_message(json!([good_row(), bad])), Some(base_message(json!([good_row()])))));
    }

    // In message snapshots, referenced messages, updates and clicks.
    out.push(recovers("S-forward-bad-row", "forward of a message with type 99 in a row", Message,
        forward_of(&bad_row_message()),
        Some(forward_of(&base_message(json!([{"type": 1, "id": 1, "components": []}]))))));
    out.push(recovers("S-reply-bad-row", "reply whose referenced message has type 99 in a row", Message,
        reply_to(bad_row_message()),
        Some(reply_to(base_message(json!([{"type": 1, "id": 1, "components": []}]))))));
    out.push(recovers("S-update-bad-row", "MESSAGE_UPDATE d with type 10 in a row", MessageUpdate,
        text_in_row_message(), Some(base_message(json!([{"type": 1, "id": 1, "components": []}])))));
    out.push(recovers("S-gw-update-bad-row", "GW MESSAGE_UPDATE with type 99 in a row", Gateway,
        gateway("MESSAGE_UPDATE", 80, gateway_message(bad_row_message())), None));
    out.push(recovers("S-gw-forward-bad-row", "GW MESSAGE_CREATE forward of a bad-row message", Gateway,
        gateway("MESSAGE_CREATE", 81, gateway_message(forward_of(&bad_row_message()))), None));

    // Form rows.
    out.push(recovers("S-modal-row-bad-child", "legacy modal row with a type 99 child", Interaction,
        modal(json!([{"type": 1, "id": 1, "components": [
            {"type": 4, "id": 2, "custom_id": "greeting", "value": "hi"},
            {"type": 99, "id": 3, "custom_id": "future"}
        ]}]), None),
        Some(modal(json!([{"type": 1, "id": 1, "components": [
            {"type": 4, "id": 2, "custom_id": "greeting", "value": "hi"}
        ]}]), None))));
    out.push(recovers("S-modal-bad-row", "modal with a row whose components is a string", Interaction,
        modal(json!([label(1, json!({"type": 21, "id": 2, "custom_id": "g", "value": "x"})),
                     {"type": 1, "id": 3, "components": "oops"}]), None),
        Some(modal(json!([label(1, json!({"type": 21, "id": 2, "custom_id": "g", "value": "x"}))]), None))));

    // Malformed resolved on clicks.
    let mut role_no_colors = role(ROLE, "Player");
    role_no_colors.as_object_mut().unwrap().remove("colors");
    let mut att_no_size = attachment();
    att_no_size.as_object_mut().unwrap().remove("size");
    att_no_size.as_object_mut().unwrap().remove("proxy_url");
    let msg = v2_selects_message();
    for (id, data) in [
        ("Q-role-no-colors", json!({"custom_id": "pick:role", "component_type": 6, "id": 26,
            "values": [ROLE, ROLE2], "resolved": {"roles": {ROLE: role_no_colors, ROLE2: role(ROLE2, "Viewer")}}})),
        ("Q-attachment-no-size", json!({"custom_id": "pick:string", "component_type": 3, "id": 22,
            "values": ["classic"], "resolved": {"attachments": {ATT: att_no_size}}})),
        ("Q-resolved-null", json!({"custom_id": "pick:user", "component_type": 5, "id": 24,
            "values": [MEMBER], "resolved": null})),
    ] {
        let mut sanitized = data.clone();
        sanitized.as_object_mut().unwrap().remove("resolved");
        out.push(recovers(id, "click whose resolved fails its typed parse", Interaction,
            click(data, msg.clone()), Some(click(sanitized, msg.clone()))));
    }
    out.push(recovers("Q-gw-role-no-colors", "GW INTERACTION_CREATE role click, role without colors", Gateway,
        gateway("INTERACTION_CREATE", 82, click(json!({"custom_id": "pick:role", "component_type": 6,
            "values": [ROLE], "resolved": {"roles": {ROLE: {"id": ROLE, "name": "Player", "color": 0,
            "hoist": false, "position": 1, "permissions": "0", "managed": false, "mentionable": false}}}}),
            v2_selects_message())), None));
    out
}

pub fn all_cases() -> Vec<Case> {
    let mut out = probe_cases();
    out.extend(form_cases());
    out.extend(click_cases());
    out.extend(ordinary_cases());
    out.extend(malformed_cases());
    out
}
