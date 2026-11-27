use wreq::header::{HeaderMap, OrigHeaderMap};

const BLINK_ADDED: [&str; 4] = [
    "sec-ch-ua",
    "sec-ch-ua-mobile",
    "sec-ch-ua-platform",
    "user-agent",
];

const AFTER_MAP: [&str; 9] = [
    "accept",
    "origin",
    "sec-fetch-site",
    "sec-fetch-mode",
    "sec-fetch-dest",
    "referer",
    "accept-encoding",
    "cookie",
    "priority",
];

const SET_BEFORE_CALLER: [&str; 1] = ["content-type"];
const SET_AFTER_CALLER: [&str; 13] = [
    "x-audit-log-reason",
    "x-context-properties",
    "x-failed-requests",
    "authorization",
    "x-super-properties",
    "x-fingerprint",
    "x-installation-id",
    "accept-language",
    "x-discord-locale",
    "x-discord-timezone",
    "x-debug-options",
    "x-routing-key",
    "x-client-trace-id",
];

const RAPID_SEED: u64 = 0xbdd8_9aa9_8270_4029;
const RAPID_SECRET: [u64; 3] = [
    0x2d35_8dcc_aa6c_78a5,
    0x8bb8_4b93_962e_acc9,
    0x4b33_a62e_d433_d4a3,
];

fn rapid_mix(a: u64, b: u64) -> u64 {
    let product = u128::from(a) * u128::from(b);
    (product as u64) ^ ((product >> 64) as u64)
}

fn name_hash(name: &str) -> u32 {
    let chars: Vec<u64> = name
        .bytes()
        .map(|byte| u64::from(byte.to_ascii_lowercase()))
        .collect();
    let len = chars.len() * 2;
    let read32 = |at: usize| chars[at] | (chars[at + 1] << 16);
    let read64 = |at: usize| {
        chars[at] | (chars[at + 1] << 16) | (chars[at + 2] << 32) | (chars[at + 3] << 48)
    };

    let mut seed = RAPID_SEED;
    seed ^= rapid_mix(seed ^ RAPID_SECRET[0], RAPID_SECRET[1]) ^ len as u64;
    let (mut a, mut b);
    if len <= 16 {
        if len >= 4 {
            let last = (len - 4) / 2;
            a = (read32(0) << 32) | read32(last);
            let delta = ((len & 24) >> (len >> 3)) / 2;
            b = (read32(delta) << 32) | read32(last - delta);
        } else if len > 0 {
            a = chars[0];
            b = 0;
        } else {
            a = 0;
            b = 0;
        }
    } else {
        let (mut i, mut p) = (len, 0usize);
        if i > 48 {
            let (mut see1, mut see2) = (seed, seed);
            loop {
                seed = rapid_mix(read64(p) ^ RAPID_SECRET[0], read64(p + 4) ^ seed);
                see1 = rapid_mix(read64(p + 8) ^ RAPID_SECRET[1], read64(p + 12) ^ see1);
                see2 = rapid_mix(read64(p + 16) ^ RAPID_SECRET[2], read64(p + 20) ^ see2);
                p += 24;
                i -= 48;
                if i < 48 {
                    break;
                }
            }
            seed ^= see1 ^ see2;
        }
        if i > 16 {
            seed = rapid_mix(
                read64(p) ^ RAPID_SECRET[2],
                read64(p + 4) ^ seed ^ RAPID_SECRET[1],
            );
            if i > 32 {
                seed = rapid_mix(read64(p + 8) ^ RAPID_SECRET[2], read64(p + 12) ^ seed);
            }
        }
        a = read64(p + (i - 16) / 2);
        b = read64(p + (i - 8) / 2);
    }
    a ^= RAPID_SECRET[1];
    b ^= seed;
    let product = u128::from(a) * u128::from(b);
    let (a, b) = (product as u64, (product >> 64) as u64);
    let hash = rapid_mix(a ^ RAPID_SECRET[0] ^ len as u64, b ^ RAPID_SECRET[1]) & 0x00ff_ffff;
    if hash == 0 { 0x0080_0000 } else { hash as u32 }
}

#[derive(Default)]
struct HeaderMapTable {
    slots: Vec<Option<String>>,
    keys: usize,
}

impl HeaderMapTable {
    fn place(slots: &mut [Option<String>], name: String) {
        let mask = slots.len() - 1;
        let mut index = name_hash(&name) as usize & mask;
        let mut probes = 0;
        while slots[index].is_some() {
            probes += 1;
            index = (index + probes) & mask;
        }
        slots[index] = Some(name);
    }

    fn insert(&mut self, name: &str) {
        let name = name.to_ascii_lowercase();
        if self
            .slots
            .iter()
            .flatten()
            .any(|existing| *existing == name)
        {
            return;
        }
        if self.slots.is_empty() {
            self.slots = vec![None; 8];
        }
        Self::place(&mut self.slots, name);
        self.keys += 1;

        if self.keys * 2 >= self.slots.len() {
            let size = if self.keys * 6 < self.slots.len() * 2 {
                self.slots.len()
            } else {
                self.slots.len() * 2
            };
            let mut grown = vec![None; size];
            for name in self.slots.drain(..).flatten() {
                Self::place(&mut grown, name);
            }
            self.slots = grown;
        }
    }

    fn in_order(&self) -> Vec<String> {
        self.slots.iter().flatten().cloned().collect()
    }
}

pub fn wire_order(present: &[String], content_length: bool) -> Vec<String> {
    let has = |name: &str| present.iter().any(|n| n == name);
    let known = |name: &str| {
        SET_BEFORE_CALLER.contains(&name)
            || SET_AFTER_CALLER.contains(&name)
            || BLINK_ADDED.contains(&name)
            || AFTER_MAP.contains(&name)
            || name == "content-length"
    };

    let mut insertion: Vec<&str> = SET_BEFORE_CALLER
        .iter()
        .copied()
        .filter(|n| has(n))
        .collect();
    insertion.extend(present.iter().map(String::as_str).filter(|n| !known(n)));
    insertion.extend(SET_AFTER_CALLER.iter().copied().filter(|n| has(n)));
    let present_blink: Vec<&str> = BLINK_ADDED.iter().copied().filter(|n| has(n)).collect();
    let tail: Vec<&str> = AFTER_MAP
        .iter()
        .copied()
        .filter(|n| has(n) || *n == "cookie")
        .collect();
    order_parts(&insertion, &present_blink, &tail, content_length)
}

pub fn order_parts(
    map_names: &[&str],
    blink_added: &[&str],
    tail: &[&str],
    content_length: bool,
) -> Vec<String> {
    let mut page_map = HeaderMapTable::default();
    for name in map_names {
        page_map.insert(name);
    }
    let mut request_map = HeaderMapTable::default();
    for name in page_map.in_order() {
        request_map.insert(&name);
    }
    for name in blink_added {
        request_map.insert(name);
    }
    let mut order = Vec::new();
    if content_length {
        order.push("content-length".to_string());
    }
    order.extend(request_map.in_order());
    order.extend(tail.iter().map(|n| n.to_string()));
    order
}

pub fn order_for(headers: &HeaderMap, content_length: bool) -> OrigHeaderMap {
    let present: Vec<String> = headers
        .keys()
        .map(|name| name.as_str().to_string())
        .collect();
    let mut order = OrigHeaderMap::new();
    for name in wire_order(&present, content_length) {
        order.insert(name);
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::measured_wire_orders::NAMES;
    use super::super::measured_wire_orders::ORDERS;
    use super::super::measured_wire_orders::TAIL;

    fn names(list: &str) -> Vec<String> {
        list.split(',').map(str::to_string).collect()
    }

    const MEASURED: [(&str, bool, &str); 6] = [
        (
            "authorization,x-super-properties,accept-language,x-discord-locale,x-discord-timezone,x-debug-options",
            false,
            "sec-ch-ua-platform,authorization,x-debug-options,accept-language,sec-ch-ua,sec-ch-ua-mobile,x-discord-timezone,x-super-properties,x-discord-locale,user-agent,accept",
        ),
        (
            "content-type,authorization,x-super-properties,accept-language,x-discord-locale,x-discord-timezone,x-debug-options",
            true,
            "content-length,sec-ch-ua-platform,authorization,x-debug-options,accept-language,sec-ch-ua,sec-ch-ua-mobile,x-discord-timezone,x-super-properties,x-discord-locale,user-agent,content-type,accept",
        ),
        (
            "authorization,x-super-properties,accept-language,x-discord-locale,x-discord-timezone,x-debug-options,x-failed-requests",
            false,
            "sec-ch-ua-platform,authorization,x-debug-options,accept-language,sec-ch-ua,x-failed-requests,sec-ch-ua-mobile,x-discord-timezone,x-super-properties,x-discord-locale,user-agent,accept",
        ),
        (
            "content-type,authorization,x-super-properties,x-installation-id,accept-language,x-discord-locale,x-discord-timezone,x-debug-options",
            true,
            "content-length,sec-ch-ua-platform,authorization,x-installation-id,x-debug-options,accept-language,sec-ch-ua,sec-ch-ua-mobile,x-discord-timezone,x-super-properties,x-discord-locale,user-agent,content-type,accept",
        ),
        (
            "content-type,authorization,x-super-properties,accept-language,x-discord-locale,x-discord-timezone,x-debug-options,x-routing-key",
            true,
            "content-length,sec-ch-ua-platform,authorization,x-debug-options,accept-language,sec-ch-ua,sec-ch-ua-mobile,x-discord-timezone,x-routing-key,x-super-properties,x-discord-locale,user-agent,content-type,accept",
        ),
        (
            "content-type,authorization,x-super-properties,accept-language,x-discord-locale,x-discord-timezone,x-debug-options,x-client-trace-id",
            true,
            "content-length,sec-ch-ua-platform,authorization,x-debug-options,accept-language,sec-ch-ua,sec-ch-ua-mobile,x-discord-timezone,x-super-properties,x-client-trace-id,x-discord-locale,user-agent,content-type,accept",
        ),
    ];

    #[test]
    fn reproduces_the_orders_measured_on_the_real_client() {
        for (set, content_length, measured) in MEASURED {
            let mut present = names(set);
            present.extend(
                [
                    "sec-ch-ua",
                    "sec-ch-ua-mobile",
                    "sec-ch-ua-platform",
                    "user-agent",
                    "accept",
                ]
                .map(String::from),
            );
            let got = wire_order(&present, content_length);
            let expected: Vec<String> = names(measured);
            assert_eq!(got[..expected.len()], expected[..], "set: {set}");
            assert_eq!(
                &got[expected.len()..],
                ["cookie"],
                "nothing but the fixed tail follows: {set}"
            );
        }
    }

    #[test]
    fn reproduces_every_set_of_the_exhaustive_replay() {
        let mut checked = 0;
        for (has_content_type, has_content_length) in [(false, false), (false, true), (true, true)]
        {
            for logged_in in [true, false] {
                for bits in 0..8u8 {
                    let captcha: &[&[&str]] = if logged_in {
                        &[&[]]
                    } else {
                        &[
                            &[],
                            &["x-captcha-key"],
                            &["x-captcha-key", "x-captcha-rqtoken", "x-captcha-session-id"],
                        ]
                    };
                    for captcha in captcha {
                        let mut present: Vec<&str> = Vec::new();
                        if has_content_type {
                            present.push("content-type");
                        }
                        present.extend(captcha.iter().copied());
                        for (bit, name) in [
                            (2, "x-audit-log-reason"),
                            (1, "x-context-properties"),
                            (0, "x-failed-requests"),
                        ] {
                            if bits >> bit & 1 == 1 {
                                present.push(name);
                            }
                        }
                        if logged_in {
                            present.push("authorization");
                        }
                        present.push("x-super-properties");
                        if !logged_in {
                            present.push("x-fingerprint");
                        }
                        present.extend([
                            "accept-language",
                            "x-discord-locale",
                            "x-discord-timezone",
                            "x-debug-options",
                            "sec-ch-ua",
                            "sec-ch-ua-mobile",
                            "sec-ch-ua-platform",
                            "user-agent",
                            "accept",
                        ]);
                        let present: Vec<String> = present.iter().map(|n| n.to_string()).collect();
                        let got: Vec<String> = wire_order(&present, has_content_length)
                            .into_iter()
                            .filter(|n| !TAIL.contains(&n.as_str()))
                            .collect();
                        let mask = got.iter().fold(0u32, |mask, name| {
                            mask | 1 << NAMES.iter().position(|n| n == name).unwrap()
                        });
                        let measured = ORDERS
                            .iter()
                            .find(|(set, _)| *set == mask)
                            .unwrap_or_else(|| panic!("no measurement for {got:?}"))
                            .1;
                        let want: Vec<&str> = measured.iter().map(|i| NAMES[*i as usize]).collect();
                        assert_eq!(
                            got, want,
                            "content-type {has_content_type}, content-length {has_content_length}, logged in {logged_in}, bits {bits:03b}, captcha {captcha:?}"
                        );
                        checked += 1;
                    }
                }
            }
        }
        assert_eq!(
            checked,
            (8 * 3 + 8 * 3 * 3),
            "every combination was checked"
        );
    }

    #[test]
    fn the_fixed_tail_follows_in_the_networks_own_order() {
        let present = names(
            "authorization,x-super-properties,accept-language,x-discord-locale,x-discord-timezone,x-debug-options,sec-ch-ua,sec-ch-ua-mobile,sec-ch-ua-platform,user-agent,accept,origin,sec-fetch-site,sec-fetch-mode,sec-fetch-dest,referer,accept-encoding,priority",
        );
        let order = wire_order(&present, false);
        assert_eq!(
            order[order.len() - 8..],
            names(
                "origin,sec-fetch-site,sec-fetch-mode,sec-fetch-dest,referer,accept-encoding,cookie,priority"
            )[..]
        );
    }

    #[test]
    fn hashes_match_chromiums_for_known_names() {
        assert_eq!(name_hash("accept") & 31, 25);
        assert_eq!(name_hash("Content-Type") & 31, 27);
        assert_eq!(name_hash("x-fingerprint") & 31, 30);
        assert_eq!(name_hash("user-agent") & 63, 56);
    }
}
