use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use requiem_plugin_host::{
    ChannelData, ChannelId, GuildData, GuildId, HostError, HostResult, HostServices, MemberData,
    OverlayUpdate, PluginHost, PresenceData, ProfileData, RelationshipData, RoleData, RoleId,
    UserData, UserId, ViewTree,
};

const ID: &str = "dev.requiem.server-info";
const GUILD: u64 = 1_000_000_000_000_000_000;
const OWNER: u64 = 42;

const ALICE: u64 = 101;
const BOB: u64 = 102;
const CAROL: u64 = 103;
const DAVE: u64 = 201;
const ERIN: u64 = 202;

#[derive(Default)]
struct Client {
    surfaces: Mutex<Vec<(String, String, String, String)>>,
    clipboard: Mutex<Vec<String>>,
    toasts: Mutex<Vec<(String, String)>>,
    profile_lookups: Mutex<Vec<UserId>>,
    opened_media: Mutex<Vec<String>>,
}

fn user(id: u64, name: &str) -> UserData {
    UserData {
        id: UserId::new(id),
        username: name.to_lowercase(),
        global_name: Some(name.to_string()),
        avatar_url: Some(format!("https://cdn.example/avatars/{id}/a.png")),
        ..Default::default()
    }
}

fn member(id: u64, name: &str) -> MemberData {
    MemberData {
        guild_id: GuildId::new(GUILD),
        user: user(id, name),
        ..Default::default()
    }
}

impl HostServices for Client {
    fn overlay(&self, _plugin_id: &str, _update: OverlayUpdate) {}

    fn guild(&self, id: GuildId) -> Option<GuildData> {
        (id.get() == GUILD).then(|| GuildData {
            id,
            name: "Black Mesa Research".to_string(),
            icon_url: Some("https://cdn.example/icons/1/icon.png".to_string()),
            banner_url: Some("https://cdn.example/banners/1/banner.png".to_string()),
            owner_id: UserId::new(OWNER),
            member_count: Some(1204),
            online_count: Some(312),
            features: vec![
                "COMMUNITY".to_string(),
                "VERIFIED".to_string(),
                "PARTNERED".to_string(),
            ],
            description: Some("Science, mostly.".to_string()),
            boost_tier: 2,
            boost_count: 14,
            splash_url: Some("https://cdn.example/splashes/1/splash.png".to_string()),
            discovery_splash_url: None,
            emojis: vec![
                (
                    "wave".to_string(),
                    "https://cdn.example/emojis/9.png".to_string(),
                ),
                (
                    "party".to_string(),
                    "https://cdn.example/emojis/10.png".to_string(),
                ),
            ],
        })
    }

    fn channels(&self, guild_id: GuildId) -> Vec<ChannelData> {
        let make = |id, kind: &str| ChannelData {
            id: ChannelId::new(id),
            guild_id: Some(guild_id),
            name: Some(format!("c{id}")),
            kind: kind.to_string(),
            ..Default::default()
        };
        vec![
            make(1, "text"),
            make(2, "text"),
            make(3, "voice"),
            make(4, "category"),
        ]
    }

    fn roles(&self, guild_id: GuildId) -> Vec<RoleData> {
        let role = |id, name: &str, position, color, permissions: &[&str]| RoleData {
            id: RoleId::new(id),
            guild_id,
            name: name.to_string(),
            position,
            color,
            permissions: permissions.iter().map(|p| p.to_string()).collect(),
            ..Default::default()
        };

        vec![
            role(
                guild_id.get(),
                "@everyone",
                0,
                None,
                &["view-channel", "send-messages"],
            ),
            role(
                7,
                "Scientist",
                2,
                Some(0x3498db),
                &[
                    "kick-members",
                    "add-reactions",
                    "view-channel",
                    "manage-messages",
                    "send-messages",
                    "embed-links",
                ],
            ),
            role(8, "Guard", 3, None, &["administrator", "kick-members"]),
        ]
    }

    fn member(&self, _guild_id: GuildId, user_id: UserId) -> Option<MemberData> {
        match user_id.get() {
            OWNER => Some(member(OWNER, "Gordon")),
            ALICE => Some(member(ALICE, "Alice")),
            DAVE => Some(member(DAVE, "Dave")),
            _ => None,
        }
    }

    fn relationships(&self) -> Vec<RelationshipData> {
        let relationship = |id, name: &str, kind: &str| RelationshipData {
            user: user(id, name),
            kind: kind.to_string(),
            ..Default::default()
        };
        vec![
            relationship(ALICE, "Alice", "friend"),
            relationship(BOB, "Bob", "friend"),
            relationship(CAROL, "Carol", "friend"),
            relationship(DAVE, "Dave", "blocked"),
            relationship(ERIN, "Erin", "blocked"),
            relationship(300, "Frank", "incoming-request"),
        ]
    }

    fn presence(&self, user_id: UserId) -> Option<PresenceData> {
        (user_id.get() == ALICE).then(|| PresenceData {
            user_id,
            status: "online".to_string(),
            ..Default::default()
        })
    }

    fn fetch_profile(&self, user_id: UserId, guild_id: Option<GuildId>) -> HostResult<ProfileData> {
        self.profile_lookups.lock().unwrap().push(user_id);
        match user_id.get() {
            BOB => Ok(ProfileData {
                user: user(BOB, "Bob"),
                member: Some(MemberData {
                    guild_id: guild_id.unwrap(),
                    ..member(BOB, "Bob")
                }),
                ..Default::default()
            }),
            CAROL | ERIN => Ok(ProfileData {
                user: user(user_id.get(), "Someone"),
                member: None,
                ..Default::default()
            }),
            _ => Err(HostError::not_found("no such user")),
        }
    }

    fn open_media(&self, url: &str) -> HostResult<()> {
        self.opened_media.lock().unwrap().push(url.to_string());
        Ok(())
    }

    fn open_surface(
        &self,
        _plugin: &str,
        surface: &str,
        view_id: &str,
        title: &str,
        extra: &str,
    ) -> HostResult<()> {
        self.surfaces.lock().unwrap().push((
            surface.to_string(),
            view_id.to_string(),
            title.to_string(),
            extra.to_string(),
        ));
        Ok(())
    }

    fn copy_to_clipboard(&self, _plugin: &str, text: &str) -> HostResult<()> {
        self.clipboard.lock().unwrap().push(text.to_string());
        Ok(())
    }

    fn toast(&self, _plugin: &str, text: &str, kind: &str) {
        self.toasts
            .lock()
            .unwrap()
            .push((text.to_string(), kind.to_string()));
    }
}

fn package() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/server-info")
}

fn host(name: &str, optional: &[&str]) -> (Arc<PluginHost>, Arc<Client>, PathBuf) {
    let root =
        env::temp_dir().join(format!("requiem-plugin-host-{name}-{}", process::id()));
    let _ = fs::remove_dir_all(&root);
    let client = Arc::new(Client::default());
    let host = PluginHost::new(&root, client.clone()).unwrap();
    let optional: Vec<String> = optional.iter().map(|p| p.to_string()).collect();
    host.install_dir_with(&package(), &optional, true).unwrap();
    (host, client, root)
}

fn wait_until(what: &str, mut done: impl FnMut() -> bool) {
    for _ in 0..200 {
        if done() {
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("timed out waiting for {what}");
}

fn started(host: &PluginHost) {
    wait_until("the plugin to register its menu entry", || {
        host.registrations()
            .iter()
            .any(|(id, r)| id == ID && !r.menu_items.is_empty())
    });
}

fn slot_items(host: &PluginHost) -> usize {
    host.registrations()
        .into_iter()
        .find(|(id, _)| id == ID)
        .map_or(0, |(_, r)| r.slot_items.len())
}

fn view_id() -> String {
    format!("guild:{GUILD}")
}

fn open(host: &PluginHost) {
    host.menu_item(
        ID,
        "guild",
        "server-info",
        &format!(r#"{{"guild-id":"{GUILD}"}}"#),
        None,
    )
    .unwrap();
}

fn render(host: &PluginHost) -> ViewTree {
    host.render_view(ID, &view_id(), r#"{"surface":"modal"}"#)
        .unwrap()
}

fn texts(tree: &ViewTree) -> Vec<String> {
    tree.nodes.iter().filter_map(|n| n.text.clone()).collect()
}

fn count(tree: &ViewTree, tag: &str) -> usize {
    tree.nodes.iter().filter(|n| n.tag == tag).count()
}

fn subtree_texts(tree: &ViewTree, node: u32) -> Vec<String> {
    let mut out: Vec<String> = tree.node(node).text.iter().cloned().collect();
    for child in tree.children(node) {
        out.extend(subtree_texts(tree, child));
    }
    out
}

fn tab_texts(tree: &ViewTree, id: &str) -> Vec<String> {
    let tab = tree
        .nodes
        .iter()
        .position(|n| n.tag == "mana:tab" && n.props.get("id").and_then(|v| v.as_str()) == Some(id))
        .unwrap_or_else(|| panic!("no tab {id}"));
    subtree_texts(tree, tab as u32)
}

fn clickable_with_text(tree: &ViewTree, text: &str) -> u32 {
    let mut node = tree
        .nodes
        .iter()
        .position(|n| n.text.as_deref() == Some(text))
        .unwrap_or_else(|| panic!("no node says {text:?}")) as u32;
    loop {
        if tree.action(node, "click").is_some() {
            return node;
        }
        node = tree
            .node(node)
            .parent
            .unwrap_or_else(|| panic!("nothing clickable around {text:?}"));
    }
}

fn tab_label(tree: &ViewTree, id: &str) -> String {
    tree.nodes
        .iter()
        .find(|n| n.tag == "mana:tab" && n.props.get("id").and_then(|v| v.as_str()) == Some(id))
        .and_then(|n| n.props.get("label"))
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string()
}

#[test]
fn the_menu_entry_opens_the_profile_as_a_modal() {
    let (host, client, root) = host("menu", &[]);
    started(&host);
    let registrations = host.registrations();
    let (_, registered) = registrations.iter().find(|(id, _)| id == ID).unwrap();
    let item = registered
        .menu_items
        .iter()
        .find(|i| i.menu == "guild")
        .expect("a guild menu item");
    assert_eq!(
        (item.label.as_str(), item.icon.as_deref()),
        ("Server Info", Some("CircleInformationIcon"))
    );

    open(&host);
    assert_eq!(
        client.surfaces.lock().unwrap().as_slice(),
        [(
            "modal".to_string(),
            view_id(),
            "Black Mesa Research".to_string(),
            "profile".to_string()
        )]
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn the_view_is_the_full_profile_with_its_tabs() {
    let (host, _, root) = host("shape", &[]);
    started(&host);
    let tree = render(&host);
    let profile = tree
        .nodes
        .iter()
        .find(|n| n.tag == "mana:profile")
        .expect("a mana:profile");
    assert_eq!(profile.props["name"], "Black Mesa Research");
    assert_eq!(
        profile.props["stats"],
        serde_json::json!([{"text": "1,204 Members", "dot": "offline"}]),
        "no online count without presence.read"
    );
    assert_eq!(
        profile.props["banner"],
        "https://cdn.example/banners/1/banner.png?size=1024"
    );
    assert_eq!(
        profile.props["icon"],
        "https://cdn.example/icons/1/icon.png?size=256"
    );
    assert_eq!(
        profile.props["badges"],
        serde_json::json!(["verified", "partnered"]),
        "both badges, each on its own, here"
    );
    assert_eq!(profile.props["tab"], "roles", "Roles is the first tab");
    let tabs: Vec<String> = tree
        .nodes
        .iter()
        .filter(|n| n.tag == "mana:tab")
        .map(|n| n.props["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(tabs, ["roles", "images", "friends", "blocked"]);

    let on_the_card: Vec<String> = tree
        .children(
            tree.nodes
                .iter()
                .position(|n| n.tag == "mana:profile")
                .unwrap() as u32,
        )
        .filter(|c| tree.node(*c).tag != "mana:tab")
        .flat_map(|c| subtree_texts(&tree, c))
        .collect();
    for expected in ["Science, mostly.", "14 boosts"] {
        assert!(
            on_the_card.iter().any(|t| t == expected),
            "{expected:?} is missing from the card: {on_the_card:?}"
        );
    }

    let in_tab = |mut node: usize| loop {
        match tree.nodes[node].parent {
            Some(parent) if tree.nodes[parent as usize].tag == "mana:tab" => break true,
            Some(parent) => node = parent as usize,
            None => break false,
        }
    };
    let sections: Vec<&str> = tree
        .nodes
        .iter()
        .enumerate()
        .filter(|(i, n)| n.tag == "mana:section" && !in_tab(*i))
        .filter_map(|(_, n)| n.props.get("title").and_then(|v| v.as_str()))
        .collect();
    assert_eq!(
        sections,
        ["Created", "Owner", "Server Boosts", "Channels", "Server ID"]
    );
    assert_eq!(tab_label(&tree, "roles"), "3 Roles");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn picking_a_tab_goes_through_the_plugin() {
    let (host, _, root) = host("tabs", &[]);
    started(&host);
    open(&host);
    let tree = render(&host);
    let profile = tree
        .nodes
        .iter()
        .position(|n| n.tag == "mana:profile")
        .unwrap() as u32;
    let action = tree
        .action(profile, "tab")
        .expect("the profile reports tab picks")
        .to_string();
    let after = host
        .ui_action(
            ID,
            &view_id(),
            &action,
            Some(serde_json::json!("friends")),
            Some("{}"),
        )
        .unwrap()
        .expect("re-rendered");
    assert_eq!(
        after
            .nodes
            .iter()
            .find(|n| n.tag == "mana:profile")
            .unwrap()
            .props["tab"],
        "friends"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn the_roles_tab_lists_every_role_with_its_permissions_compactly() {
    let (host, _, root) = host("roles", &[]);
    started(&host);
    open(&host);
    let tree = render(&host);
    let roles = tab_texts(&tree, "roles");

    let order: Vec<&String> = roles
        .iter()
        .filter(|t| ["Guard", "Scientist", "@everyone"].contains(&t.as_str()))
        .collect();
    assert_eq!(order, ["Guard", "Scientist", "@everyone"], "{roles:?}");

    assert!(
        roles.iter().any(|t| t == "Administrator") && roles.iter().any(|t| t == "All permissions"),
        "{roles:?}"
    );

    for expected in [
        "Kick Members",
        "Manage Messages",
        "Add Reactions",
        "View Channel",
        "+2",
        "6 permissions",
    ] {
        assert!(
            roles.iter().any(|t| t == expected),
            "{expected:?} is missing from {roles:?}"
        );
    }
    assert!(
        !roles.iter().any(|t| t == "Embed Links"),
        "closed rows hide the rest"
    );

    let row = clickable_with_text(&tree, "Scientist");
    let action = tree.action(row, "click").unwrap().to_string();
    host.ui_action(ID, &view_id(), &action, None, None).unwrap();
    let open_tree = render(&host);
    let open_roles = tab_texts(&open_tree, "roles");
    for expected in ["Embed Links", "Send Messages"] {
        assert!(
            open_roles.iter().any(|t| t == expected),
            "{expected:?} is missing from {open_roles:?}"
        );
    }
    assert!(
        !open_roles.iter().any(|t| t == "+2"),
        "nothing is hidden when it's open"
    );
    let action = open_tree
        .action(clickable_with_text(&open_tree, "Scientist"), "click")
        .unwrap()
        .to_string();
    host.ui_action(ID, &view_id(), &action, None, None).unwrap();
    assert!(
        !tab_texts(&render(&host), "roles")
            .iter()
            .any(|t| t == "Embed Links")
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn the_images_tab_lists_every_picture_and_each_opens_bigger() {
    fn images_under(tree: &ViewTree, node: u32, out: &mut Vec<(String, String)>) {
        let n = tree.node(node);
        if n.tag == "mana:image" {
            out.push((
                n.props["src"].as_str().unwrap().to_string(),
                n.props["open"].as_str().unwrap().to_string(),
            ));
        }
        for child in tree.children(node) {
            images_under(tree, child, out);
        }
    }

    let (host, _, root) = host("images", &[]);
    started(&host);
    let tree = render(&host);

    assert_eq!(tab_label(&tree, "images"), "Images", "no count on this tab");
    let texts = tab_texts(&tree, "images");
    assert!(texts.iter().any(|t| t == "Invite Background"), "{texts:?}");
    assert!(
        !texts.iter().any(|t| t == "Server Icon" || t == "Banner"),
        "already on the card: {texts:?}"
    );

    assert!(
        texts.iter().any(|t| t == "Discovery Splash") && texts.iter().any(|t| t == "Not set"),
        "{texts:?}"
    );
    let tab = tree
        .nodes
        .iter()
        .position(|n| {
            n.tag == "mana:tab" && n.props.get("id").and_then(|v| v.as_str()) == Some("images")
        })
        .unwrap() as u32;
    let mut images = Vec::new();
    images_under(&tree, tab, &mut images);
    assert_eq!(images.len(), 3, "the invite background and two emoji");
    assert_eq!(
        images[0],
        (
            "https://cdn.example/splashes/1/splash.png?size=256".to_string(),
            "https://cdn.example/splashes/1/splash.png?size=2048".to_string()
        ),
        "the thumbnail is small, the picture it opens is big"
    );
    assert_eq!(images[1].1, "https://cdn.example/emojis/9.png?size=512");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn required_permissions_alone_give_the_profile_without_the_extras() {
    let (host, client, root) = host("bare", &[]);
    started(&host);
    open(&host);
    let tree = render(&host);
    assert_eq!(
        count(&tree, "mana:avatar"),
        0,
        "the owner and the people need members.read"
    );
    assert!(
        texts(&tree).iter().any(|t| t.starts_with("Not available")),
        "it says why the owner is missing"
    );
    let profile = tree.nodes.iter().find(|n| n.tag == "mana:profile").unwrap();
    assert_eq!(
        profile.props["stats"],
        serde_json::json!([{"text": "1,204 Members", "dot": "offline"}]),
        "online counts need presence.read"
    );
    assert_eq!(tab_label(&tree, "friends"), "No Friends");
    let empty = tree
        .nodes
        .iter()
        .filter(|n| n.tag == "mana:empty-state")
        .filter_map(|n| n.props.get("text").and_then(|v| v.as_str()))
        .collect::<Vec<_>>();
    assert!(
        empty
            .iter()
            .any(|t| t.starts_with("Allow \"See your friends and blocked users\"")),
        "{empty:?}"
    );
    assert!(
        client.profile_lookups.lock().unwrap().is_empty(),
        "no lookups without the permissions"
    );
    assert_eq!(
        slot_items(&host),
        0,
        "the menu-under-the-name entry needs ui.slots"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn with_the_optional_permissions_it_finds_your_friends_and_blocked_users_in_the_server() {
    let (host, client, root) = host(
        "full",
        &[
            "members.read",
            "relationships.read",
            "presence.read",
            "ui.slots",
        ],
    );
    started(&host);
    open(&host);

    wait_until("the lookups to finish", || {
        tab_label(&render(&host), "friends") == "2 Friends"
            && tab_label(&render(&host), "blocked") == "1 Blocked User"
    });

    let tree = render(&host);
    let friends = tab_texts(&tree, "friends");
    assert!(
        friends.iter().any(|t| t == "Alice") && friends.iter().any(|t| t == "Bob"),
        "{friends:?}"
    );
    assert!(
        !friends.iter().any(|t| t == "Carol"),
        "Carol isn't in the server: {friends:?}"
    );
    assert!(
        !friends.iter().any(|t| t.contains("Looking for")),
        "{friends:?}"
    );
    let blocked = tab_texts(&tree, "blocked");
    assert!(
        blocked.iter().any(|t| t == "Dave") && !blocked.iter().any(|t| t == "Erin"),
        "{blocked:?}"
    );
    let mut looked_up = client.profile_lookups.lock().unwrap().clone();
    looked_up.sort();
    assert_eq!(
        looked_up.iter().map(|id| id.get()).collect::<Vec<_>>(),
        vec![BOB, CAROL, ERIN],
        "only people the client hadn't loaded are looked up"
    );

    let alice = tree
        .nodes
        .iter()
        .find(|n| {
            n.tag == "mana:avatar" && n.props.get("name").and_then(|v| v.as_str()) == Some("Alice")
        })
        .unwrap();
    assert_eq!(alice.props["status"], "online");
    let owner = texts(&tree).iter().any(|t| t == "Gordon");
    assert!(owner, "the owner's name, with members.read");
    let stats = &tree
        .nodes
        .iter()
        .find(|n| n.tag == "mana:profile")
        .unwrap()
        .props["stats"];
    assert_eq!(
        *stats,
        serde_json::json!([{"text": "312 Online", "dot": "online"}, {"text": "1,204 Members", "dot": "offline"}])
    );
    assert_eq!(slot_items(&host), 1);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn granting_a_permission_later_restarts_the_plugin_so_it_registers_the_slot() {
    let (host, _, root) = host("grant", &[]);
    started(&host);
    assert_eq!(slot_items(&host), 0);
    host.set_permission(ID, "ui.slots", true).unwrap();
    wait_until("the slot item to appear", || slot_items(&host) == 1);
    host.set_permission(ID, "ui.slots", false).unwrap();
    wait_until("the slot item to go away", || slot_items(&host) == 0);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn copying_the_id_is_a_gesture_and_needs_no_permission() {
    let (host, client, root) = host("copy", &[]);
    started(&host);
    let tree = render(&host);

    let row = clickable_with_text(&tree, &GUILD.to_string());
    let action = tree.action(row, "click").unwrap().to_string();
    host.ui_action(ID, &view_id(), &action, None, None).unwrap();
    assert_eq!(
        client.clipboard.lock().unwrap().as_slice(),
        [GUILD.to_string()]
    );
    assert!(
        client
            .toasts
            .lock()
            .unwrap()
            .iter()
            .any(|(text, kind)| kind == "success" && text == "Server ID copied")
    );
    let _ = fs::remove_dir_all(root);
}
