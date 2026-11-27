use std::path::{Path, PathBuf};

use apply::HelperArgs;
use requiem_updater::apply::{self, Journal};
use std::env;
use std::fs;
use std::process;

fn temp(name: &str) -> PathBuf {
    let d = env::temp_dir().join(format!("update-apply-{name}-{}", process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn write(dir: &Path, rel: &str, bytes: &[u8]) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

fn read(dir: &Path, rel: &str) -> Vec<u8> {
    fs::read(dir.join(rel)).unwrap()
}

fn layout(name: &str) -> (PathBuf, PathBuf) {
    let base = temp(name);
    let install = base.join("app");
    let staged = install.join(".update").join("2.0.0");
    write(&install, "app.exe", b"old exe");
    write(&install, "codec.dll", b"old dll");
    write(&install, "data/app.db", b"user data");
    write(&staged, "app.exe", b"new exe");
    write(&staged, "codec.dll", b"new dll");
    write(&staged, "extra/new.txt", b"brand new");
    (install, staged)
}

#[test]
fn a_swap_replaces_files_keeps_the_old_ones_aside_and_leaves_user_data_alone() {
    let (install, staged) = layout("swap");
    let journal = apply::swap(&install, &staged, "2.0.0").unwrap();
    assert_eq!(read(&install, "app.exe"), b"new exe");
    assert_eq!(read(&install, "codec.dll"), b"new dll");
    assert_eq!(read(&install, "extra/new.txt"), b"brand new");
    assert_eq!(read(&install, "app.exe.old"), b"old exe");
    assert_eq!(read(&install, "data/app.db"), b"user data");

    apply::commit(&install, &journal);
    assert!(!install.join("app.exe.old").exists() && !install.join("codec.dll.old").exists());
    assert_eq!(read(&install, "app.exe"), b"new exe");
}

#[test]
fn a_rollback_restores_every_file_and_removes_the_new_ones() {
    let (install, staged) = layout("rollback");
    let journal = apply::swap(&install, &staged, "2.0.0").unwrap();
    apply::rollback(&install, &journal).unwrap();
    assert_eq!(read(&install, "app.exe"), b"old exe");
    assert_eq!(read(&install, "codec.dll"), b"old dll");
    assert!(!install.join("extra/new.txt").exists());
    assert!(!install.join("app.exe.old").exists());
    assert_eq!(read(&install, "data/app.db"), b"user data");
}

#[test]
fn a_failure_part_way_undoes_what_was_already_swapped() {
    let (install, staged) = layout("partial");

    write(&install, "sub", b"i am a file");
    write(&staged, "sub/z.txt", b"z");
    let err = apply::swap(&install, &staged, "2.0.0").unwrap_err();
    assert!(!err.to_string().is_empty());
    assert_eq!(
        read(&install, "app.exe"),
        b"old exe",
        "files swapped before the failure are restored"
    );
    assert_eq!(read(&install, "codec.dll"), b"old dll");
    assert_eq!(read(&install, "sub"), b"i am a file");
    assert!(!install.join("app.exe.old").exists());
    assert_eq!(read(&install, "data/app.db"), b"user data");
}

#[test]
fn a_stuck_old_file_gets_a_numbered_name_instead_of_failing() {
    let (install, staged) = layout("stuck");

    fs::create_dir_all(install.join("app.exe.old")).unwrap();
    write(&install, "app.exe.old/keep", b"x");
    let journal = apply::swap(&install, &staged, "2.0.0").unwrap();
    let entry = journal
        .entries
        .iter()
        .find(|e| e.target == Path::new("app.exe"))
        .unwrap();
    assert_eq!(entry.old.as_deref(), Some(Path::new("app.exe.old1")));
    apply::rollback(&install, &journal).unwrap();
    assert_eq!(read(&install, "app.exe"), b"old exe");
}

#[test]
fn helper_arguments_round_trip() {
    let args: Vec<String> = ["C:/app", "C:/app/.update/2.0.0", "2.0.0", "app.exe"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let parsed = apply::helper_args_from(&args).unwrap();
    assert_eq!(parsed.version, "2.0.0");
    assert_eq!(parsed.root(), Path::new("C:/app/.update"));
    assert_eq!(parsed.app_args, ["--post-update"]);
    assert!(apply::helper_args_from(&args[..3]).is_none());
}

#[test]
fn a_report_is_read_once_and_a_pending_journal_is_committed_by_the_app() {
    let (install, staged) = layout("pending");
    let root = install.join(".update");
    let journal = apply::swap(&install, &staged, "2.0.0").unwrap();
    fs::write(
        root.join("pending.json"),
        serde_json::to_vec(&journal).unwrap(),
    )
    .unwrap();
    apply::commit_pending(&install, &root);
    assert!(!install.join("app.exe.old").exists());
    assert!(!root.join("pending.json").exists());

    fs::write(
        root.join("rolled-back.json"),
        br#"{"version":"2.0.0","reason":"x"}"#,
    )
    .unwrap();
    assert_eq!(apply::take_report(&root).unwrap().version, "2.0.0");
    assert!(apply::take_report(&root).is_none());
    let _: Option<Journal> = None;
}

#[cfg(windows)]
mod running_processes {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    use super::*;

    fn fake_app(dir: &Path, name: &str, extra_byte: bool) {
        let cmd = Path::new(&env::var("SystemRoot").unwrap())
            .join("System32")
            .join("cmd.exe");
        fs::create_dir_all(dir).unwrap();
        let mut bytes = fs::read(cmd).unwrap();
        if extra_byte {
            bytes.push(0);
        }
        fs::write(dir.join(name), bytes).unwrap();
    }

    fn size(dir: &Path, name: &str) -> u64 {
        fs::metadata(dir.join(name)).unwrap().len()
    }

    fn sleeper(dir: &Path, name: &str, pings: u32) -> process::Child {
        Command::new(dir.join(name))
            .args(["/c", &format!("ping -n {pings} 127.0.0.1 > nul")])
            .stdout(Stdio::null())
            .spawn()
            .unwrap()
    }

    fn args_for(install: &Path, staged: &Path, app_args: Vec<String>) -> HelperArgs {
        let mut args = HelperArgs::new(
            install.to_path_buf(),
            staged.to_path_buf(),
            "2.0.0".into(),
            "app.exe".into(),
        );
        args.app_args = app_args;
        args.exit_timeout = Duration::from_secs(20);
        args.health_timeout = Duration::from_secs(20);
        args
    }

    fn setup(name: &str) -> (PathBuf, PathBuf, u64, u64) {
        let install = temp(name).join("app");
        let staged = install.join(".update").join("2.0.0");
        fake_app(&install, "app.exe", false);
        fake_app(&staged, "app.exe", true);
        write(&install, "data/app.db", b"user data");
        (
            install.clone(),
            staged.clone(),
            size(&install, "app.exe"),
            size(&staged, "app.exe"),
        )
    }

    #[test]
    fn a_running_executable_is_busy_until_it_exits() {
        let (install, _, _, _) = setup("busy");
        let mut child = sleeper(&install, "app.exe", 4);
        let exe = install.join("app.exe");
        assert!(
            !apply::wait_until_free(&exe, Duration::from_millis(500)),
            "a running executable cannot be written"
        );
        let started = Instant::now();
        assert!(apply::wait_until_free(&exe, Duration::from_secs(20)));
        assert!(started.elapsed() > Duration::from_millis(500));
        let _ = child.wait();
    }

    #[test]
    fn a_healthy_new_version_is_kept_and_the_old_files_are_removed() {
        let (install, staged, _old, new) = setup("healthy");
        let root = install.join(".update");
        let healthy = root.join("healthy");
        let script = format!(
            "echo ok > {} & ping -n 3 127.0.0.1 > nul",
            healthy.display()
        );
        let code = apply::run_helper(&args_for(&install, &staged, vec!["/c".into(), script]));
        assert_eq!(code, 0);
        assert_eq!(size(&install, "app.exe"), new);
        assert!(!install.join("app.exe.old").exists());
        assert!(!root.join("pending.json").exists());
        assert_eq!(read(&install, "data/app.db"), b"user data");
        assert!(apply::take_report(&root).is_none());
    }

    #[test]
    fn a_new_version_that_dies_at_once_is_rolled_back_and_reported() {
        let (install, staged, old, _new) = setup("crash");
        let root = install.join(".update");
        let code = apply::run_helper(&args_for(
            &install,
            &staged,
            vec!["/c".into(), "exit 3".into()],
        ));
        assert_eq!(code, 4);
        assert_eq!(size(&install, "app.exe"), old, "the old version is back");
        assert!(!install.join("app.exe.old").exists());
        let report = apply::take_report(&root).expect("a report is left for the updater");
        assert_eq!(report.version, "2.0.0");
        assert!(report.reason.contains("exited"), "{}", report.reason);
        assert_eq!(read(&install, "data/app.db"), b"user data");
    }

    #[test]
    fn the_helper_waits_for_the_old_app_to_exit_before_swapping() {
        let (install, staged, _old, new) = setup("waits");
        let root = install.join(".update");
        let healthy = root.join("healthy");
        let mut old_app = sleeper(&install, "app.exe", 4);
        let script = format!(
            "echo ok > {} & ping -n 3 127.0.0.1 > nul",
            healthy.display()
        );
        let started = Instant::now();
        let code = apply::run_helper(&args_for(&install, &staged, vec!["/c".into(), script]));
        assert_eq!(code, 0);
        assert!(
            started.elapsed() > Duration::from_secs(2),
            "it did not wait for the old app"
        );
        assert_eq!(size(&install, "app.exe"), new);
        let _ = old_app.wait();
    }

    #[test]
    fn an_old_app_that_never_exits_leaves_the_install_untouched() {
        let (install, staged, old, _new) = setup("never");
        let mut old_app = sleeper(&install, "app.exe", 12);
        let mut args = args_for(&install, &staged, vec![]);
        args.exit_timeout = Duration::from_secs(1);
        assert_eq!(apply::run_helper(&args), 2);
        assert_eq!(size(&install, "app.exe"), old);
        let _ = old_app.kill();
        let _ = old_app.wait();
    }
}
