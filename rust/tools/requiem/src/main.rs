#![allow(clippy::print_stdout, clippy::print_stderr)]

use commands::asio;
use commands::assets;
use commands::cargo;
use commands::desktop;
use commands::doctor;
use commands::emoji;
use commands::fingerprint;
use commands::plugin;
use commands::release;
use std::process;

mod commands;
mod env;
mod util;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "requiem",
    version,
    about = "requiem dev CLI",
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Doctor,

    /// Download the Steinberg ASIO SDK into vendor/ASIOSDK.
    Asio {
        /// Replace an SDK that is already there.
        #[arg(long)]
        force: bool,
    },

    Cargo {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    Build {
        #[arg(short, long, default_value = "requiem")]
        package: String,
        #[arg(long)]
        release: bool,

        #[arg(last = true)]
        extra: Vec<String>,
    },

    Run {
        #[arg(long)]
        release: bool,
        #[arg(last = true)]
        app_args: Vec<String>,
    },

    Test {
        #[arg(short, long)]
        package: Option<String>,
        #[arg(last = true)]
        extra: Vec<String>,
    },

    Desktop {
        #[arg(long, default_value = "stable")]
        channel: String,

        #[arg(long)]
        force: bool,

        #[arg(long)]
        check: bool,

        #[arg(long)]
        out: Option<PathBuf>,
    },

    Assets {
        #[command(subcommand)]
        command: AssetsCmd,
    },

    Plugin {
        #[command(subcommand)]
        command: PluginCmd,
    },

    Fingerprint {
        #[arg(long)]
        impersonate: Option<String>,

        #[arg(long, default_value = "https://tls.peet.ws/api")]
        echo: String,

        #[arg(long)]
        update: bool,
    },

    Emoji {
        #[command(subcommand)]
        command: EmojiCmd,
    },

    Release {
        #[command(subcommand)]
        command: ReleaseCmd,
    },
}

#[derive(Subcommand)]
enum ReleaseCmd {
    Keygen {
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        out: PathBuf,
    },

    Sign {
        zip: PathBuf,

        #[arg(long)]
        release_version: String,
        #[arg(long, default_value = "stable")]
        channel: String,
        #[arg(long, default_value = "windows-x64")]
        target: String,

        #[arg(long)]
        key_id: String,

        #[arg(long = "base")]
        bases: Vec<String>,

        #[arg(long = "artifact-url")]
        artifact_urls: Vec<String>,

        #[arg(long)]
        serial: Option<u64>,
        #[arg(long, default_value_t = 14)]
        valid_days: u64,
        #[arg(long)]
        notes_url: Option<String>,

        #[arg(long = "revoke")]
        revoked: Vec<String>,
        #[arg(long)]
        out: PathBuf,
    },

    Resign {
        input: PathBuf,

        #[arg(long)]
        key_id: String,
        #[arg(long, default_value_t = 14)]
        valid_days: u64,
        #[arg(long)]
        out: PathBuf,
    },

    Identity {
        input: PathBuf,

        #[arg(long)]
        key_id: String,
        #[arg(long)]
        out: PathBuf,
    },

    Publish {
        dir: PathBuf,

        #[arg(long)]
        base: String,

        #[arg(long)]
        token_env: Option<String>,

        #[arg(long, default_value_t = 50 * 1024 * 1024)]
        part_size: u64,
    },

    KeysJson {
        #[arg(long)]
        out: PathBuf,
    },

    Verify {
        file: PathBuf,

        #[arg(long)]
        key_id: String,

        #[arg(long)]
        public: String,
    },
}

#[derive(Subcommand)]
enum PluginCmd {
    New {
        name: String,

        #[arg(long)]
        id: Option<String>,

        #[arg(long)]
        dir: Option<PathBuf>,
    },

    Build {
        dir: Option<PathBuf>,

        #[arg(long)]
        debug: bool,
    },
}

#[derive(Subcommand)]
enum EmojiCmd {
    Vendors,

    Build {
        vendor: String,

        #[arg(long)]
        out: Option<PathBuf>,

        #[arg(long)]
        force: bool,
    },

    Manifest {
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        all: bool,
    },
}

#[derive(Subcommand)]
enum AssetsCmd {
    Resolve {
        #[arg(long)]
        base: Option<PathBuf>,

        #[arg(long)]
        fresh: bool,

        #[arg(long)]
        out: Option<PathBuf>,

        #[arg(long, default_value = "latest")]
        build: String,

        #[arg(long, default_value = "same")]
        client: String,

        #[arg(long)]
        refresh: Vec<String>,

        #[arg(long)]
        cache: Option<PathBuf>,
    },

    Show {
        path: Option<PathBuf>,
    },

    Find {
        d_prefix: String,

        #[arg(long)]
        cache: Option<PathBuf>,
    },

    Inventory {
        section: Option<String>,

        #[arg(long)]
        cache: Option<PathBuf>,
    },

    Svg {
        export: String,
        #[arg(long)]
        cache: Option<PathBuf>,
    },

    Check {
        path: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() {
    let code = match run().await {
        Ok(code) => code,
        Err(err) => {
            util::bad(&format!("{err:#}"));
            1
        }
    };
    process::exit(code);
}

async fn run() -> Result<i32> {
    let cli = Cli::parse();

    let cli = match cli.command {
        Cmd::Release { command } => {
            return match command {
                ReleaseCmd::Keygen { id, out } => release::keygen(id.as_deref(), &out),
                ReleaseCmd::Sign {
                    zip,
                    release_version,
                    channel,
                    target,
                    key_id,
                    bases,
                    artifact_urls,
                    serial,
                    valid_days,
                    notes_url,
                    revoked,
                    out,
                } => release::sign(release::SignArgs {
                    zip,
                    version: release_version,
                    channel,
                    target,
                    key_id,
                    bases,
                    artifact_urls,
                    serial,
                    valid_days,
                    notes_url,
                    revoked,
                    out,
                }),
                ReleaseCmd::Resign {
                    input,
                    key_id,
                    valid_days,
                    out,
                } => release::resign(&input, &key_id, valid_days, &out),
                ReleaseCmd::Identity { input, key_id, out } => {
                    release::identity(input, &key_id, &out)
                }
                ReleaseCmd::Publish {
                    dir,
                    base,
                    token_env,
                    part_size,
                } => release::publish(&dir, &base, token_env.as_deref(), part_size).await,
                ReleaseCmd::KeysJson { out } => release::keys_json(&out),
                ReleaseCmd::Verify {
                    file,
                    key_id,
                    public,
                } => release::verify(&file, &key_id, &public),
            };
        }
        _ => cli,
    };
    let root = util::workspace_root()?;
    match cli.command {
        Cmd::Doctor => doctor::run(&root).await,
        Cmd::Asio { force } => asio::run(&root, force).await,
        Cmd::Cargo { args } => cargo::cargo(&root, args).await,
        Cmd::Build {
            package,
            release,
            extra,
        } => cargo::build(&root, package, release, extra).await,
        Cmd::Run { release, app_args } => cargo::run(&root, release, app_args).await,
        Cmd::Test { package, extra } => cargo::test(&root, package, extra).await,
        Cmd::Desktop {
            channel,
            force,
            check,
            out,
        } => desktop::run(&root, &channel, force, check, out.as_deref()).await,
        Cmd::Fingerprint {
            impersonate,
            echo,
            update,
        } => fingerprint::run(&root, impersonate, echo, update).await,
        Cmd::Release { .. } => unreachable!("handled above"),
        Cmd::Plugin { command } => match command {
            PluginCmd::New { name, id, dir } => plugin::new(&root, &name, id, dir),
            PluginCmd::Build { dir, debug } => plugin::build(&root, dir, debug).await,
        },
        Cmd::Emoji { command } => match command {
            EmojiCmd::Vendors => emoji::vendors().await,
            EmojiCmd::Build { vendor, out, force } => {
                emoji::build(&root, &vendor, out, force).await
            }
            EmojiCmd::Manifest { out, all } => emoji::manifest(&root, out, all).await,
        },
        Cmd::Assets { command } => match command {
            AssetsCmd::Resolve {
                base,
                fresh,
                out,
                build,
                client,
                refresh,
                cache,
            } => assets::resolve(&root, base, fresh, out, build, client, refresh, cache).await,
            AssetsCmd::Show { path } => assets::show(&root, path),
            AssetsCmd::Find { d_prefix, cache } => assets::find(&root, d_prefix, cache),
            AssetsCmd::Inventory { section, cache } => assets::inventory(&root, cache, section),
            AssetsCmd::Svg { export, cache } => assets::svg(&root, export, cache),
            AssetsCmd::Check { path } => assets::check(&root, path).await,
        },
    }
}
