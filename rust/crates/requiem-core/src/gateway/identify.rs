use serde_json::json;

use crate::properties::SuperProperties;

const CAPABILITIES: u64 = 1_734_653;

pub fn build(token: &str, super_properties: &SuperProperties) -> serde_json::Value {
    json!({
        "op": super::opcodes::IDENTIFY,
        "d": {
            "token": token,
            "capabilities": CAPABILITIES,
            "properties": super_properties.gateway_value(),

            "presence": {
                "status": "unknown",
                "since": 0,
                "activities": [],
                "afk": false,
            },
            "compress": false,
            "client_state": {
                "guild_versions": {},
            },

        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identify_has_the_shape_of_a_real_clients() {
        let payload = build("tok", &SuperProperties::desktop_like());
        let keys =
            |v: &serde_json::Value| v.as_object().unwrap().keys().cloned().collect::<Vec<_>>();
        let d = &payload["d"];
        assert_eq!(
            keys(d),
            [
                "token",
                "capabilities",
                "properties",
                "presence",
                "compress",
                "client_state"
            ]
        );
        assert_eq!(d["capabilities"], 1_734_653);
        assert_eq!(d["presence"]["status"], "unknown");
        let props = keys(&d["properties"]);
        assert_eq!(
            &props[props.len() - 3..],
            [
                "is_fast_connect",
                "gateway_connect_reasons",
                "installation_id"
            ]
        );
        let id = d["properties"]["installation_id"].as_str().unwrap();
        let (snowflake, token) = id.split_once('.').unwrap();
        assert!(snowflake.parse::<u64>().is_ok());
        assert_eq!(token.len(), 43);
    }
}
