use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use requiem_plugin_host::{
    ChannelId, HostServices, IncomingItem, IncomingSource, LogLevel, MessageData, MessageId,
    OutgoingDraft, OverlayUpdate, PluginHost, Status, budget,
};

#[derive(Default)]
struct Services {
    logs: Mutex<Vec<String>>,
}

impl HostServices for Services {
    fn log(&self, _plugin_id: &str, _level: LogLevel, message: &str) {
        self.logs.lock().unwrap().push(message.to_string());
    }
    fn overlay(&self, _plugin_id: &str, _update: OverlayUpdate) {}
    fn secret_get(&self, _: &str, _: &str) -> requiem_plugin_host::HostResult<Option<String>> {
        Ok(None)
    }
    fn secret_set(&self, _: &str, _: &str, _: &str) -> requiem_plugin_host::HostResult<()> {
        Ok(())
    }
    fn secret_delete(&self, _: &str, _: &str) -> requiem_plugin_host::HostResult<()> {
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

const ID: &str = "dev.requiem.misbehaving";

fn host(name: &str) -> (Arc<PluginHost>, Arc<Services>, PathBuf) {
    let root = env::temp_dir().join(format!(
        "requiem-plugin-host-test-{name}-{}",
        process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let services = Arc::new(Services::default());
    let host = PluginHost::new(&root, services.clone()).unwrap();
    host.install_dir(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/misbehaving"))
        .unwrap();
    host.set_permission(ID, "messages.intercept-incoming", true)
        .unwrap();
    host.set_permission(ID, "messages.intercept-outgoing", true)
        .unwrap();
    host.set_enabled(ID, true).unwrap();

    let _ = host.render_view(ID, "settings", "{}");
    (host, services, root)
}

#[test]
fn a_panic_is_a_pass_and_the_plugin_recovers() {
    let (host, services, root) = host("panic");
    let draft = OutgoingDraft {
        channel_id: ChannelId::new(1),
        content: "hi".into(),
        ..Default::default()
    };
    let outcome = host.run_outgoing(draft.clone());
    assert!(outcome.cancelled.is_none());
    assert_eq!(outcome.draft, draft);
    assert!(
        services
            .logs
            .lock()
            .unwrap()
            .iter()
            .any(|l| l.contains("crashed")),
        "{:?}",
        services.logs.lock().unwrap()
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn an_endless_loop_is_cut_off_at_the_budget() {
    let (host, _services, root) = host("loop");
    let started = Instant::now();
    let outcomes = host.run_incoming(vec![item(MessageData {
        id: MessageId::new(1),
        content: "x".into(),
        ..Default::default()
    })]);
    let took = started.elapsed();
    assert!(outcomes[0].is_unchanged());
    assert!(took < budget::INCOMING_BATCH * 10, "took {took:?}");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn running_out_of_memory_is_contained() {
    let (host, _services, root) = host("oom");
    assert!(host.render_view(ID, "settings", "{}").is_err());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn repeated_failures_suspend_the_plugin_until_re_enabled() {
    let (host, _services, root) = host("strikes");
    let message = || {
        vec![item(MessageData {
            id: MessageId::new(1),
            ..Default::default()
        })]
    };
    for _ in 0..3 {
        host.run_incoming(message());
    }
    assert!(
        matches!(host.list()[0].status, Status::Suspended(_)),
        "{:?}",
        host.list()[0].status
    );
    let started = Instant::now();
    host.run_incoming(message());
    assert!(
        started.elapsed() < budget::INCOMING_BATCH,
        "a suspended plugin isn't called at all"
    );
    host.set_enabled(ID, true).unwrap();
    assert!(!matches!(host.list()[0].status, Status::Suspended(_)));
    let _ = fs::remove_dir_all(root);
}
