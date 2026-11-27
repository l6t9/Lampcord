use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;

use requiem_plugin_host::{HostServices, OverlayUpdate, PluginHost};

struct Services;

impl HostServices for Services {
    fn overlay(&self, _plugin_id: &str, _update: OverlayUpdate) {}
}

const ID: &str = "dev.requiem.word-filter";
const INCOMING: &str = "messages.intercept-incoming";
const OUTGOING: &str = "messages.intercept-outgoing";

fn temp(name: &str) -> PathBuf {
    let dir =
        env::temp_dir().join(format!("requiem-plugin-host-{name}-{}", process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn package(name: &str) -> PathBuf {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/word-filter");
    let dir = temp(name).join("package");
    fs::create_dir_all(&dir).unwrap();
    for file in ["plugin.wasm", "styles.css"] {
        fs::copy(fixture.join(file), dir.join(file)).unwrap();
    }
    let manifest = fs::read_to_string(fixture.join("plugin.toml")).unwrap();
    let kept: Vec<&str> = manifest
        .lines()
        .filter(|line| {
            !line.starts_with(&format!("\"{OUTGOING}\"")) && *line != "[optional-permissions]"
        })
        .collect();
    let optional = format!(
        "[optional-permissions]
\"{OUTGOING}\" = \"Stops blocked words from being sent\"
\"clipboard.write\" = \"Copies a word\"

[hooks]"
    );
    let manifest = kept
        .join(
            "
",
        )
        .replace("[hooks]", &optional);
    fs::write(dir.join("plugin.toml"), manifest).unwrap();
    dir
}

fn host(name: &str) -> (Arc<PluginHost>, PathBuf) {
    let root = temp(name);
    (PluginHost::new(&root, Arc::new(Services)).unwrap(), root)
}

fn granted(host: &PluginHost) -> Vec<(String, bool, bool)> {
    host.list()[0]
        .permissions
        .iter()
        .map(|p| (p.name.clone(), p.required, p.granted))
        .collect()
}

#[test]
fn preview_lists_required_first_and_grants_only_those() {
    let (host, root) = host("preview");
    let preview = host.inspect_dir(&package("preview-pkg")).unwrap();
    assert_eq!(preview.id, ID);
    assert!(preview.installed_version.is_none());
    let rows: Vec<_> = preview
        .permissions
        .iter()
        .map(|p| (p.name.as_str(), p.required, p.granted))
        .collect();
    assert_eq!(
        rows,
        [
            (INCOMING, true, true),
            ("clipboard.write", false, false),
            (OUTGOING, false, false)
        ]
    );
    assert!(host.list().is_empty(), "previewing installs nothing");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn install_grants_required_plus_the_chosen_optional_ones() {
    let (host, root) = host("install");
    host.install_dir_with(
        &package("install-pkg"),
        &[OUTGOING.to_string(), "bogus".to_string()],
        true,
    )
    .unwrap();
    assert_eq!(
        granted(&host),
        [
            (INCOMING.to_string(), true, true),
            ("clipboard.write".to_string(), false, false),
            (OUTGOING.to_string(), false, true)
        ]
    );
    assert!(
        host.list()[0].enabled,
        "the dialog's Install switches the plugin on"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn plain_install_still_starts_off_with_only_required_grants() {
    let (host, root) = host("plain");
    host.install_dir(&package("plain-pkg")).unwrap();
    assert!(!host.list()[0].enabled);
    assert_eq!(granted(&host)[0], (INCOMING.to_string(), true, true));
    assert!(!granted(&host)[1].2 && !granted(&host)[2].2);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn required_permissions_cannot_be_revoked_but_optional_ones_can() {
    let (host, root) = host("revoke");
    host.install_dir(&package("revoke-pkg")).unwrap();
    assert!(host.set_permission(ID, INCOMING, false).is_err());
    assert!(host.set_permission(ID, INCOMING, true).is_ok());
    host.set_permission(ID, "clipboard.write", true).unwrap();
    assert!(granted(&host)[1].2);
    host.set_permission(ID, "clipboard.write", false).unwrap();
    assert!(!granted(&host)[1].2);
    assert!(
        host.set_permission(ID, "guilds.read", true).is_err(),
        "not asked for"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn an_update_keeps_choices_and_the_preview_shows_them() {
    let (host, root) = host("update");
    let dir = package("update-pkg");
    host.install_dir_with(&dir, &["clipboard.write".to_string()], false)
        .unwrap();
    let preview = host.inspect_dir(&dir).unwrap();
    assert_eq!(preview.installed_version.as_deref(), Some("0.1.0"));
    assert!(
        preview
            .permissions
            .iter()
            .find(|p| p.name == "clipboard.write")
            .unwrap()
            .granted,
        "already allowed"
    );
    assert!(
        !preview
            .permissions
            .iter()
            .find(|p| p.name == OUTGOING)
            .unwrap()
            .granted
    );
    host.install_dir(&dir).unwrap();
    assert!(
        granted(&host)[1].2,
        "an update keeps the grants that still apply"
    );
    let _ = fs::remove_dir_all(root);
}
