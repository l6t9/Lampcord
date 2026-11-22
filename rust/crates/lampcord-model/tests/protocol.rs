use serde::Deserialize;

use lampcord_model::channel::ChannelType;
use lampcord_model::gateway::{GatewayOp, GatewayPayload, Hello, Ready};
use lampcord_model::message::{Message, MessageType};
use lampcord_model::snowflake::{Snowflake, parse_snowflake, snowflake_timestamp};

#[test]
fn parses_snowflake_strings_and_integers() {
    let id = parse_snowflake("175928847299117063").expect("valid snowflake");
    assert_eq!(id, 175_928_847_299_117_063);
    assert_eq!(snowflake_timestamp(1 << 22), 1_420_070_400_001);
    assert!(parse_snowflake("not-an-id").is_err());

    #[derive(Deserialize)]
    struct Id(#[serde(with = "lampcord_model::snowflake::serde")] Snowflake);

    let value: Id = serde_json::from_str("\"123\"").expect("string id");
    assert_eq!(value.0, 123);
    let value: Id = serde_json::from_str("123").expect("numeric id");
    assert_eq!(value.0, 123);
}

#[test]
fn parses_message_create_dispatch() {
    let payload = r#"{
        "op": 0,
        "t": "MESSAGE_CREATE",
        "s": 42,
        "d": {
            "id": "123456789012345678",
            "channel_id": "234567890123456789",
            "guild_id": "345678901234567890",
            "author": {
                "id": "456789012345678901",
                "username": "lamp",
                "discriminator": "0",
                "avatar": null
            },
            "content": "hello",
            "timestamp": "2026-01-01T00:00:00.000000+00:00",
            "edited_timestamp": null,
            "tts": false,
            "mention_everyone": false,
            "mentions": [],
            "mention_roles": ["567890123456789012"],
            "attachments": [],
            "embeds": [],
            "pinned": false,
            "type": 0
        }
    }"#;

    let payload: GatewayPayload<Message> = serde_json::from_str(payload).expect("valid payload");
    assert_eq!(payload.op, GatewayOp::DISPATCH);
    assert_eq!(payload.s, Some(42));
    assert_eq!(payload.t.as_deref(), Some("MESSAGE_CREATE"));

    let message = payload.d.expect("message");
    assert_eq!(message.id, 123_456_789_012_345_678);
    assert_eq!(message.content, "hello");
    assert_eq!(message.kind, MessageType::DEFAULT);
    assert_eq!(message.mention_roles, vec![567_890_123_456_789_012]);
}

#[test]
fn parses_hello_and_ready_payloads() {
    let hello: Hello =
        serde_json::from_str(r#"{"heartbeat_interval":41250}"#).expect("valid hello");
    assert_eq!(hello.heartbeat_interval, 41_250);

    let payload = r#"{
        "op": 10,
        "d": {"heartbeat_interval": 41250}
    }"#;
    let payload: GatewayPayload<Hello> = serde_json::from_str(payload).expect("valid payload");
    assert_eq!(payload.op, GatewayOp::HELLO);
    assert_eq!(payload.d.expect("hello").heartbeat_interval, 41_250);

    let ready = r#"{
        "v": 9,
        "user": {"id": "1", "username": "lampcord", "discriminator": "0"},
        "guilds": [{"id": "2", "unavailable": true}],
        "session_id": "session-1",
        "resume_gateway_url": "wss://gateway.discord.gg/"
    }"#;
    let ready: Ready = serde_json::from_str(ready).expect("valid ready");
    assert_eq!(ready.session_id, "session-1");
    assert_eq!(ready.guilds.len(), 1);
    assert!(ready.guilds[0].unavailable);
}

#[test]
fn channel_type_helpers_cover_voice_and_threads() {
    assert!(ChannelType::GUILD_VOICE.is_voice());
    assert!(ChannelType::GUILD_STAGE_VOICE.is_voice());
    assert!(!ChannelType::GUILD_TEXT.is_voice());
    assert!(ChannelType::PUBLIC_THREAD.is_thread());
}
