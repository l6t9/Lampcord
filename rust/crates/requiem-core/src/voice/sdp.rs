use std::collections::HashSet;

pub fn build_fragment(offer_sdp: &str) -> String {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for line in offer_sdp.lines() {
        let line = line.trim();
        let lower = line.to_ascii_lowercase();
        let keep = lower.starts_with("a=ice")
            || lower.contains("a=extmap")
            || lower.contains("a=fingerprint")
            || lower.contains("opus")
            || lower.contains("vp8")
            || lower.contains("rtx");
        if keep && seen.insert(line.to_string()) {
            out.push(line.to_string());
        }
    }
    out.join("\n")
}

pub fn repair_answer(answer: &str) -> String {
    if answer.trim_start().starts_with("v=") {
        return answer.to_string();
    }

    let mut port = "9".to_string();
    let mut fingerprint = String::new();
    let mut body = String::new();

    for raw in answer.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("m=audio ") {
            port = rest.split_whitespace().next().unwrap_or("9").to_string();
        } else if line.starts_with("a=fingerprint") {
            if fingerprint.is_empty() {
                fingerprint = line.to_string();
            }
        } else {
            body.push_str(line);
            body.push_str("\r\n");
        }
    }

    format!(
        "v=0\r\no=- 0 0 IN IP4 0.0.0.0\r\ns=-\r\nt=0 0\r\nm=audio {port} UDP/TLS/RTP/SAVPF 120\r\na=mid:0\r\n{body}{fingerprint}\r\na=setup:passive\r\na=rtpmap:120 opus/48000/2\r\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fragment_keeps_only_the_allowlist() {
        let offer = "\
v=0\r
o=- 123 2 IN IP4 0.0.0.0\r
s=-\r
t=0 0\r
a=group:BUNDLE 0\r
a=msid-semantic: WMS\r
m=audio 9 UDP/TLS/RTP/SAVPF 120\r
c=IN IP4 0.0.0.0\r
a=sendrecv\r
a=mid:0\r
a=ice-ufrag:ABCD\r
a=ice-pwd:EFGHIJKLMNOP\r
a=ice-options:trickle\r
a=fingerprint:sha-256 AA:BB:CC\r
a=setup:actpass\r
a=rtpmap:120 opus/48000/2\r
a=fmtp:120 minptime=10;useinbandfec=1\r
a=rtcp-fb:120 transport-cc\r
a=extmap:1 urn:ietf:params:rtp-hdrext:sdes:mid\r
a=extmap:3 urn:ietf:params:rtp-hdrext:ssrc-audio-level\r
a=ssrc:123456 cname:foo\r
a=ssrc:123456 msid:stream track\r
a=candidate:1 1 udp 2130706431 192.168.1.5 54321 typ host\r
";
        let fragment = build_fragment(offer);

        assert!(fragment.contains("a=ice-ufrag:ABCD"));
        assert!(fragment.contains("a=ice-pwd:EFGHIJKLMNOP"));
        assert!(fragment.contains("a=fingerprint:sha-256 AA:BB:CC"));
        assert!(fragment.contains("a=rtpmap:120 opus/48000/2"));
        assert!(fragment.contains("a=extmap:1"));
        assert!(fragment.contains("a=extmap:3"));

        assert!(!fragment.contains("v=0"));
        assert!(!fragment.contains("m=audio"));
        assert!(!fragment.contains("a=mid:0"));
        assert!(!fragment.contains("a=ssrc:"));
        assert!(!fragment.contains("a=candidate:"));
        assert!(!fragment.contains("a=fmtp:120"));
    }

    #[test]
    fn fragment_dedupes() {
        let offer = "a=ice-ufrag:X\na=ice-ufrag:X\n";
        assert_eq!(build_fragment(offer), "a=ice-ufrag:X");
    }

    #[test]
    fn repairs_spacebars_minimal_answer_into_a_standard_sdp() {
        let minimal = "m=audio 50000 ICE/SDP\na=fingerprint:sha-256 AA:BB:CC\nc=IN IP4 1.2.3.4\na=rtcp:50000\na=ice-ufrag:ufrag123\na=ice-pwd:pwd123\na=fingerprint:sha-256 AA:BB:CC\na=candidate:1 1 UDP 2130706431 1.2.3.4 50000 typ host\n";
        let repaired = repair_answer(minimal);
        assert!(repaired.starts_with("v=0\r\n"));
        assert!(repaired.contains("m=audio 50000 UDP/TLS/RTP/SAVPF 120\r\n"));
        assert!(repaired.contains("a=mid:0\r\n"));
        assert!(repaired.contains("c=IN IP4 1.2.3.4\r\n"));
        assert!(repaired.contains("a=ice-ufrag:ufrag123\r\n"));
        assert!(repaired.contains("a=ice-pwd:pwd123\r\n"));
        assert!(repaired.contains("a=candidate:1 1 UDP 2130706431 1.2.3.4 50000 typ host\r\n"));
        assert!(repaired.contains("a=setup:passive\r\n"));
        assert!(repaired.contains("a=rtpmap:120 opus/48000/2\r\n"));

        assert_eq!(repaired.matches("a=fingerprint").count(), 1);
    }

    #[test]
    fn leaves_a_standard_answer_untouched() {
        let standard =
            "v=0\r\no=- 0 0 IN IP4 0.0.0.0\r\ns=-\r\nt=0 0\r\nm=audio 9 UDP/TLS/RTP/SAVPF 120\r\n";
        assert_eq!(repair_answer(standard), standard);
    }
}
