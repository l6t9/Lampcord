use base64::engine::general_purpose;
use requiem_core::properties::SuperProperties;
use requiem_core::properties;
use requiem_core::rest::RequestExtras;
use requiem_core::rest;
use std::path::Path;
use wreq::header;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::util::{BOLD, DIM, RESET, bad, hint, ok, step, warn};
use serde_json::Map;
use std::env;
use std::fs;
use std::process::Command;

const REFERENCE_REPO: &str = "https://github.com/dolfies/discord.py-self";

const REFERENCE_PY: &str = r#"
import asyncio, json, sys
src, ours_path, out_path, echo_base, pin = sys.argv[1:6]
sys.path.insert(0, src)
import curl_cffi
from discord.http import HTTPClient, Route
from discord.tracking import HeadersContext

ours = json.load(open(ours_path, encoding='utf-8'))

class Context(HeadersContext):
    @property
    def impersonate(self):
        return pin or super().impersonate

async def main():
    ctx = Context(
        super_properties=ours['super_properties'],
        platform='Windows',
        browser_type='electron',
        browser_major_version=int(ours['chrome_version'].split('.')[0]),
    )
    http = HTTPClient(loop=asyncio.get_running_loop(), headers_context=ctx)
    http.token = 'local-echo-placeholder'
    Route.BASE = echo_base
    captures = {}
    captures['get'] = await http.request(Route('GET', '/all'))
    captures['post_json'] = await http.request(Route('POST', '/all'), json={'content': 'x', 'nonce': '1', 'tts': False})
    json.dump({
        'client': 'discord.py-self',
        'impersonate': ctx.impersonate,
        'curl_cffi': curl_cffi.__version__,
        'captures': captures,
    }, open(out_path, 'w', encoding='utf-8'), indent=2)

asyncio.run(main())
"#;

pub async fn run(
    root: &Path,
    impersonate: Option<String>,
    echo: String,
    update: bool,
) -> Result<i32> {
    let work = root.join(".cache").join("fingerprint");
    fs::create_dir_all(&work)?;
    let checkout = work.join("discord.py-self");
    let venv = work.join("venv");
    let py = if cfg!(windows) {
        venv.join("Scripts").join("python.exe")
    } else {
        venv.join("bin").join("python")
    };

    step("Reference: discord.py-self");
    if !checkout.exists() {
        run_tool(
            "git",
            &[
                "clone",
                "--depth",
                "1",
                REFERENCE_REPO,
                &checkout.to_string_lossy(),
            ],
            None,
        )?;
    } else if update {
        run_tool("git", &["pull", "--ff-only"], Some(&checkout))?;
    }
    if !py.exists() {
        run_tool("python", &["-m", "venv", &venv.to_string_lossy()], None)?;
        run_tool(
            &py.to_string_lossy(),
            &[
                "-m",
                "pip",
                "install",
                "-q",
                "-r",
                &checkout.join("requirements.txt").to_string_lossy(),
            ],
            None,
        )?;
    }
    let sha =
        capture("git", &["rev-parse", "--short", "HEAD"], Some(&checkout)).unwrap_or_default();
    ok(&format!("discord.py-self {}", sha.trim()));

    step("Ours: sending through build_client + api_headers");
    let ours_path = work.join("ours.json");

    if let Some(major) = impersonate
        .as_deref()
        .map(|i| {
            i.chars()
                .filter(char::is_ascii_digit)
                .take(3)
                .collect::<String>()
        })
        .filter(|m| !m.is_empty())
    {
        env::set_var("FINGERPRINT_CHROME_MAJOR", major);
    }
    capture_ours(&ours_path, &format!("{echo}/all")).await?;

    step("Reference: sending through discord.py-self's HTTPClient");
    let script = work.join("reference.py");
    fs::write(&script, REFERENCE_PY)?;
    let theirs_path = work.join("discord.py-self.json");
    run_tool(
        &py.to_string_lossy(),
        &[
            "-I",
            &script.to_string_lossy(),
            &checkout.to_string_lossy(),
            &ours_path.to_string_lossy(),
            &theirs_path.to_string_lossy(),
            &echo,
            impersonate.as_deref().unwrap_or(""),
        ],
        None,
    )?;

    let ours: Value = serde_json::from_slice(&fs::read(&ours_path)?)?;
    let theirs: Value = serde_json::from_slice(&fs::read(&theirs_path)?)?;
    println!(
        "  {DIM}ours: {} ({}) · reference: curl_cffi {} impersonating {}{RESET}",
        ours["tls_profile"].as_str().unwrap_or("?"),
        ours["chrome_version"].as_str().unwrap_or("?"),
        theirs["curl_cffi"].as_str().unwrap_or("?"),
        theirs["impersonate"].as_str().unwrap_or("?")
    );

    let mut failures = 0;
    for request in ["get", "post_json"] {
        step(&format!("Comparing {request}"));
        failures += compare(&ours["captures"][request], &theirs["captures"][request]);
    }
    println!();
    if failures == 0 {
        ok("identical on every compared field");
        warn(
            "this proves parity with discord.py-self only, not with Discord's own desktop client (see `cargo requiem fingerprint --help`)",
        );
        Ok(0)
    } else {
        bad(&format!(
            "{failures} difference(s); captures kept in {}",
            work.display()
        ));
        Ok(1)
    }
}

fn run_tool(program: &str, args: &[&str], cwd: Option<&Path>) -> Result<()> {
    let mut command = Command::new(program);
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let status = command
        .status()
        .with_context(|| format!("starting {program}"))?;
    if !status.success() {
        bail!("{program} {} failed", args.join(" "));
    }
    Ok(())
}

async fn capture_ours(out: &Path, url: &str) -> Result<()> {
    if let Some(major) = env::var("FINGERPRINT_CHROME_MAJOR")
        .ok()
        .and_then(|v| v.parse().ok())
    {
        properties::pin_chromium_major(major);
    }
    let client = rest::build_client().context("building the client")?;
    let xsp = SuperProperties::desktop_like();
    let token = "local-echo-placeholder";
    let mut captures = Map::new();
    let json = || Some(serde_json::json!({"content": "x", "nonce": "1", "tts": false}));
    for (name, method, body) in [
        ("get", wreq::Method::GET, None),
        ("post_json", wreq::Method::POST, json()),
        ("patch_json", wreq::Method::PATCH, json()),
        ("put_empty", wreq::Method::PUT, None),
        ("post_empty", wreq::Method::POST, None),
        ("delete", wreq::Method::DELETE, None),
    ] {
        let (mut headers, order) = rest::request_headers(
            "https://discord.com",
            &xsp,
            &method,
            token,
            body.is_some(),
            &RequestExtras::default(),
        );
        if let Ok(cookie) = env::var("FINGERPRINT_COOKIE") {
            headers.insert(
                header::COOKIE,
                cookie.parse().context("cookie header")?,
            );
        }
        let mut request = client
            .request(method, url)
            .headers(headers)
            .orig_headers(order);
        if let Some(body) = &body {
            request = request.json(body);
        }
        let text = request
            .send()
            .await
            .context("fingerprint request")?
            .text()
            .await
            .context("fingerprint body")?;
        captures.insert(
            name.to_string(),
            serde_json::from_str(&text).unwrap_or(Value::String(text)),
        );
    }
    let doc = serde_json::json!({
        "client": "requiem",
        "tls_profile": format!("{:?}", properties::emulation_profile()),
        "user_agent": properties::user_agent(),
        "chrome_version": properties::chrome_version(),
        "super_properties": serde_json::from_slice::<Value>(
            &base64::Engine::decode(
                &general_purpose::STANDARD,
                xsp.to_header_value(),
            )
            .context("super properties are not base64")?,
        )
        .context("super properties are not JSON")?,
        "captures": captures,
    });
    fs::write(out, serde_json::to_string_pretty(&doc)?)
        .with_context(|| format!("writing {}", out.display()))?;
    ok(&format!("wrote {}", out.display()));
    Ok(())
}

fn capture(program: &str, args: &[&str], cwd: Option<&Path>) -> Result<String> {
    let mut command = Command::new(program);
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    Ok(String::from_utf8_lossy(&command.output()?.stdout).into_owned())
}

const VOLATILE: &[&str] = &[
    "ip",
    "client_random",
    "session_id",
    "ja3",
    "ja3_hash",
    "tcpip",
    "donate",
    "id",
    "date",
    "time",
    "timestamp",
    "ticket",
    "psk",
    "binder",
    "binders",
    "identities",
];

fn normalise(value: &Value) -> Value {
    let mut root = Map::new();
    for key in ["http_version", "method", "tls", "http2"] {
        if let Some(v) = value.get(key) {
            root.insert(key.to_string(), clean(v, key));
        }
    }
    Value::Object(root)
}

fn clean(value: &Value, key: &str) -> Value {
    match value {
        Value::Object(map) if key == "shared_keys" || key == "data" => Value::Object(
            map.iter()
                .map(|(k, _)| (grease(k), Value::String(String::new())))
                .collect(),
        ),
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(k, _)| !VOLATILE.contains(&k.as_str()))
                .map(|(k, v)| (grease(k), clean(v, k)))
                .collect(),
        ),

        Value::String(s) if key == "data" => Value::String(format!("{} hex chars", s.len())),
        Value::Array(items) => {
            let mut items: Vec<Value> = items
                .iter()
                .map(|v| {
                    let mut v = clean(v, key);

                    if v.get("name")
                        .and_then(Value::as_str)
                        .is_some_and(|n| n.contains("EncryptedClientHello"))
                    {
                        if let Some(map) = v.as_object_mut() {
                            map.remove("data");
                        }
                    }
                    v
                })
                .collect();
            if key == "extensions" {
                items.sort_by_key(|v| {
                    v.get("name")
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| v.to_string())
                });
            }
            Value::Array(items)
        }
        Value::String(s) if s.contains("GREASE") => Value::String("GREASE".into()),
        other => other.clone(),
    }
}

fn grease(key: &str) -> String {
    if key.contains("GREASE") {
        "GREASE".into()
    } else {
        key.to_string()
    }
}

fn compare(ours: &Value, theirs: &Value) -> usize {
    let (a, b) = (normalise(ours), normalise(theirs));
    let mut diffs = Vec::new();
    diff("", &a, &b, &mut diffs);

    let (oh, th) = (header_lines(ours), header_lines(theirs));
    if oh == th {
        ok(&format!(
            "headers: {} lines identical (order, case, values)",
            oh.len()
        ));
    } else {
        bad("headers differ (ours, then reference), in wire order");
        for i in 0..oh.len().max(th.len()) {
            let (o, t) = (
                oh.get(i).map(String::as_str).unwrap_or("-"),
                th.get(i).map(String::as_str).unwrap_or("-"),
            );
            if o != t {
                println!("      {BOLD}{i:>2}{RESET} {o}\n         {t}");
            }
        }
    }
    let header_diff = usize::from(oh != th);
    diffs.retain(|d| !d.contains("headers"));
    if diffs.is_empty() {
        ok(
            "TLS + HTTP/2 (ja4, ja4_r, peetprint, ciphers, extensions, SETTINGS, WINDOW_UPDATE, priority, pseudo-header order) identical",
        );
    }
    for d in &diffs {
        bad(d);
    }
    if !diffs.is_empty() {
        hint(
            "a TLS difference with a different Chromium major is a profile gap, not a bug: pin the reference with --impersonate",
        );
    }
    diffs.len() + header_diff
}

fn header_lines(value: &Value) -> Vec<String> {
    value["http2"]["sent_frames"]
        .as_array()
        .into_iter()
        .flatten()
        .find_map(|f| f.get("headers").and_then(|h| h.as_array()))
        .map(|h| {
            h.iter()
                .filter_map(|l| l.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn diff(path: &str, a: &Value, b: &Value, out: &mut Vec<String>) {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            for key in x.keys().chain(y.keys().filter(|k| !x.contains_key(*k))) {
                let p = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                side(&p, x.get(key), y.get(key), out);
            }
        }
        (Value::Array(x), Value::Array(y)) => {
            for i in 0..x.len().max(y.len()) {
                side(&format!("{path}[{i}]"), x.get(i), y.get(i), out);
            }
        }
        _ if a != b => out.push(format!(
            "{path}: ours {} != reference {}",
            short(a),
            short(b)
        )),
        _ => {}
    }
}

fn side(path: &str, ours: Option<&Value>, theirs: Option<&Value>, out: &mut Vec<String>) {
    match (ours, theirs) {
        (Some(l), Some(r)) => diff(path, l, r, out),
        (Some(l), None) => out.push(format!("{path}: only ours = {}", short(l))),
        (None, Some(r)) => out.push(format!("{path}: only reference = {}", short(r))),
        (None, None) => {}
    }
}

fn short(v: &Value) -> String {
    let s = v.to_string();
    if s.chars().count() > 120 {
        format!("{}...", s.chars().take(120).collect::<String>())
    } else {
        s
    }
}
