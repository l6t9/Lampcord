use requiem_plugin_host::css;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::sync::{Arc, Mutex};

use requiem_plugin_host::{
    ChannelId, EditDraft, HostServices, IncomingItem, IncomingSource, LogLevel, MessageData,
    MessageId, OutgoingDraft, OverlayUpdate, PluginHost, Status, UserData, UserId,
};

#[derive(Default)]
struct Services {
    logs: Mutex<Vec<String>>,
    secrets: Mutex<Vec<(String, String)>>,
}

impl HostServices for Services {
    fn log(&self, plugin_id: &str, level: LogLevel, message: &str) {
        self.logs
            .lock()
            .unwrap()
            .push(format!("{level:?} [{plugin_id}] {message}"));
    }
    fn overlay(&self, _plugin_id: &str, _update: OverlayUpdate) {}
    fn secret_get(
        &self,
        _plugin_id: &str,
        key: &str,
    ) -> requiem_plugin_host::HostResult<Option<String>> {
        Ok(self
            .secrets
            .lock()
            .unwrap()
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone()))
    }
    fn secret_set(
        &self,
        _plugin_id: &str,
        key: &str,
        value: &str,
    ) -> requiem_plugin_host::HostResult<()> {
        self.secrets
            .lock()
            .unwrap()
            .push((key.to_string(), value.to_string()));
        Ok(())
    }
    fn secret_delete(&self, _plugin_id: &str, key: &str) -> requiem_plugin_host::HostResult<()> {
        self.secrets.lock().unwrap().retain(|(k, _)| k != key);
        Ok(())
    }
}

fn item(message: MessageData) -> IncomingItem {
    IncomingItem {
        message,
        previous: None,
        source: IncomingSource::History,
    }
}

const ID: &str = "dev.requiem.word-filter";

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/word-filter")
}

fn temp_root(name: &str) -> PathBuf {
    let root = env::temp_dir().join(format!(
        "requiem-plugin-host-test-{name}-{}",
        process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    root
}

fn message(id: u64, content: &str) -> MessageData {
    MessageData {
        id: MessageId::new(id),
        channel_id: ChannelId::new(1),
        content: content.to_string(),
        author: UserData {
            id: UserId::new(9),
            username: "someone".into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn draft(content: &str) -> OutgoingDraft {
    OutgoingDraft {
        channel_id: ChannelId::new(1),
        content: content.to_string(),
        ..Default::default()
    }
}

fn installed(name: &str) -> (Arc<PluginHost>, Arc<Services>, PathBuf) {
    let root = temp_root(name);
    let services = Arc::new(Services::default());
    let host = PluginHost::new(&root, services.clone()).unwrap();
    assert_eq!(host.install_dir(&fixture()).unwrap(), ID);
    assert!(!host.list()[0].enabled, "a fresh install starts disabled");
    for permission in ["messages.intercept-incoming", "messages.intercept-outgoing"] {
        host.set_permission(ID, permission, true).unwrap();
    }
    host.set_enabled(ID, true).unwrap();
    (host, services, root)
}

fn set_words(host: &PluginHost, words: &str) {
    let page = host.render_view(ID, "settings", "{}").unwrap();
    let field = page
        .nodes
        .iter()
        .position(|n| n.tag == "mana:text-field")
        .expect("the page has a text field");
    let action = page
        .action(field as u32, "change")
        .expect("the field reports changes")
        .to_string();
    host.ui_action(
        ID,
        "settings",
        &action,
        Some(serde_json::Value::String(words.to_string())),
        None,
    )
    .unwrap();
}

#[test]
fn settings_page_drives_the_hooks() {
    let (host, services, root) = installed("hooks");

    let outcome = host.run_outgoing(draft("a spoiler here"));
    assert!(outcome.cancelled.is_none() && outcome.modified_by.is_empty());

    set_words(&host, "spoiler, leak");

    let page = host.render_view(ID, "settings", "{}").unwrap();
    let field = page
        .nodes
        .iter()
        .find(|n| n.tag == "mana:text-field")
        .unwrap();
    assert_eq!(
        field.props["text"], "spoiler, leak",
        "the page re-renders from the saved settings"
    );
    assert!(
        page.nodes[0].classes[0].starts_with(&css::prefix(ID)),
        "classes come back scoped: {:?}",
        page.nodes[0].classes
    );

    let outcome = host.run_outgoing(draft("a SPOILER here"));
    let cancelled = outcome.cancelled.expect("blocked word cancels the send");
    assert_eq!(cancelled.plugin, ID);
    assert!(cancelled.value.contains("blocked word"));

    let edit = host.run_edit(EditDraft {
        channel_id: ChannelId::new(1),
        message_id: MessageId::new(5),
        original_content: "x".into(),
        content: "the leak is real".into(),
        ..Default::default()
    });
    assert_eq!(edit.draft.content, "the **** is real");
    assert_eq!(edit.modified_by, vec![ID.to_string()]);

    let outcomes = host.run_incoming(vec![
        item(message(1, "hello")),
        item(message(2, "big spoiler!")),
        IncomingItem {
            message: message(3, " Leak "),
            previous: None,
            source: IncomingSource::GatewayCreate,
        },
    ]);
    assert!(outcomes[0].is_unchanged());
    assert_eq!(
        outcomes[1].display_content.as_ref().unwrap().value,
        "big ********"
    );
    assert_eq!(outcomes[1].annotations[0].value, "Blocked words hidden");
    assert!(outcomes[2].hidden.is_some());

    let page = host.render_view(ID, "settings", "{}").unwrap();
    assert!(
        page.nodes
            .iter()
            .any(|n| n.text.as_deref() == Some("1 message(s) masked so far")),
        "{:#?}",
        page.nodes
    );

    assert!(
        services
            .logs
            .lock()
            .unwrap()
            .iter()
            .any(|l| l.contains("word filter started"))
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn hooks_need_their_permissions() {
    let (host, _services, root) = installed("perms");
    set_words(&host, "spoiler");
    host.set_permission(ID, "messages.intercept-outgoing", false)
        .unwrap();
    assert!(
        host.run_outgoing(draft("spoiler")).cancelled.is_none(),
        "revoked permission: the hook doesn't run"
    );
    host.set_enabled(ID, false).unwrap();
    assert!(
        host.run_incoming(vec![item(message(1, "spoiler"))])[0].is_unchanged(),
        "disabled: nothing runs"
    );
    assert_eq!(host.list()[0].status, Status::Disabled);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn state_survives_a_restart() {
    let (host, services, root) = installed("restart");
    set_words(&host, "spoiler");
    drop(host);
    let host = PluginHost::new(&root, services).unwrap();
    let info = &host.list()[0];
    assert!(info.enabled && info.permissions.iter().all(|p| p.granted));
    assert!(
        host.run_outgoing(draft("spoiler")).cancelled.is_some(),
        "settings came back from storage"
    );
    host.remove(ID).unwrap();
    assert!(host.list().is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn stylesheet_is_scoped_and_only_for_enabled_plugins() {
    let (host, _services, root) = installed("css");
    let css = host.stylesheet();
    let view = css::view_class(ID);
    assert!(css.contains(&format!(".{view} .")), "{css}");
    host.set_enabled(ID, false).unwrap();
    assert!(host.stylesheet().is_empty());
    let _ = fs::remove_dir_all(root);
}
