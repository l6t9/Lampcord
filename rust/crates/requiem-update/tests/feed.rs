use requiem_update::*;
use semver::Version;

fn key(n: u8) -> SigningKey {
    sign_key_from_seed(&[n; 32])
}

fn keyset(keys: &[(&str, &SigningKey)]) -> KeySet {
    KeySet::from_entries(
        keys.iter()
            .map(|(id, k)| KeyEntry::new(id, &k.verifying_key())),
    )
    .unwrap()
}

fn manifest() -> UpdateManifest {
    UpdateManifest {
        schema: 1,
        channel: "stable".into(),
        serial: 10,
        version: "1.2.0".into(),
        released_at: 1_000,
        expires_at: 2_000,
        revoked: vec![],
        notes_url: None,
        sources: vec![],
        artifacts: vec![Artifact {
            target: "windows-x64".into(),
            sha256: sha256_hex(b"zip bytes"),
            size: 9,
            urls: vec!["https://example.test/a.zip".into()],
        }],
    }
}

fn policy<'a>(current: &'a Version) -> Policy<'a> {
    Policy {
        current,
        channel: "stable",
        target: "windows-x64",
        now: 1_500,
        last_serial: 0,
    }
}

fn signed(m: &UpdateManifest, k: &SigningKey) -> Signed {
    Signed::sign(KIND_UPDATE, m, "k1", k).unwrap()
}

#[test]
fn a_correctly_signed_manifest_opens_and_offers_the_update() {
    let k = key(1);
    let keys = keyset(&[("k1", &k)]);
    let opened: UpdateManifest = signed(&manifest(), &k).open(&keys, KIND_UPDATE).unwrap();
    let current = Version::new(1, 1, 0);
    match opened.check(&policy(&current)).unwrap() {
        Decision::Update { version, artifact } => {
            assert_eq!(version, Version::new(1, 2, 0));
            assert_eq!(artifact.target, "windows-x64");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn envelopes_survive_a_trip_through_json() {
    let k = key(1);
    let keys = keyset(&[("k1", &k)]);
    let text = signed(&manifest(), &k).to_json();
    assert!(
        Signed::parse(&text)
            .unwrap()
            .open::<UpdateManifest>(&keys, KIND_UPDATE)
            .is_ok()
    );
}

#[test]
fn a_tampered_payload_is_rejected() {
    let k = key(1);
    let keys = keyset(&[("k1", &k)]);
    let mut env = signed(&manifest(), &k);
    env.payload = env.payload.replace("1.2.0", "9.9.9");
    assert_eq!(env.verify(&keys, KIND_UPDATE), Err(Error::BadSignature));
}

#[test]
fn a_signature_from_another_key_is_rejected() {
    let (real, other) = (key(1), key(2));
    let keys = keyset(&[("k1", &real)]);
    assert_eq!(
        signed(&manifest(), &other).verify(&keys, KIND_UPDATE),
        Err(Error::BadSignature)
    );
}

#[test]
fn a_key_that_is_not_in_the_set_is_rejected() {
    let k = key(1);
    let keys = keyset(&[("other", &k)]);
    assert_eq!(
        signed(&manifest(), &k).verify(&keys, KIND_UPDATE),
        Err(Error::UnknownKey("k1".into()))
    );
}

#[test]
fn an_empty_key_set_trusts_nothing() {
    assert_eq!(
        signed(&manifest(), &key(1)).verify(&KeySet::default(), KIND_UPDATE),
        Err(Error::UnknownKey("k1".into()))
    );
}

#[test]
fn the_official_keys_are_a_primary_and_a_spare_and_reject_other_signers() {
    let official = official_keys();
    assert_eq!(
        official.ids().collect::<Vec<_>>(),
        ["primary-2026", "spare-2026"]
    );

    let forged = Signed::sign(KIND_UPDATE, &manifest(), "primary-2026", &key(1)).unwrap();
    assert_eq!(
        forged.verify(&official, KIND_UPDATE),
        Err(Error::BadSignature)
    );
}

#[test]
fn a_document_cannot_pass_as_another_kind() {
    let k = key(1);
    let keys = keyset(&[("k1", &k)]);
    let env = signed(&manifest(), &k);
    assert!(matches!(
        env.verify(&keys, KIND_IDENTITY),
        Err(Error::WrongKind { .. })
    ));

    let mut relabelled = env;
    relabelled.kind = KIND_IDENTITY.into();
    assert_eq!(
        relabelled.verify(&keys, KIND_IDENTITY),
        Err(Error::BadSignature)
    );
}

#[test]
fn rollback_and_equal_versions_are_up_to_date_not_updates() {
    let m = manifest();
    for current in [Version::new(1, 2, 0), Version::new(2, 0, 0)] {
        assert_eq!(m.check(&policy(&current)).unwrap(), Decision::UpToDate);
    }
}

#[test]
fn prereleases_order_below_their_release() {
    let m = manifest();
    let current = Version::parse("1.2.0-alpha.1").unwrap();
    assert!(matches!(
        m.check(&policy(&current)).unwrap(),
        Decision::Update { .. }
    ));
}

#[test]
fn an_expired_feed_is_rejected() {
    let current = Version::new(1, 0, 0);
    let mut p = policy(&current);
    p.now = 2_000;
    assert_eq!(manifest().check(&p).unwrap_err(), Error::Expired(2_000));
}

#[test]
fn a_replayed_older_feed_is_rejected_but_the_same_serial_is_fine() {
    let current = Version::new(1, 0, 0);
    let mut p = policy(&current);
    p.last_serial = 11;
    assert_eq!(
        manifest().check(&p).unwrap_err(),
        Error::OlderThanSeen {
            found: 10,
            seen: 11
        }
    );
    p.last_serial = 10;
    assert!(manifest().check(&p).is_ok());
}

#[test]
fn another_channel_is_rejected() {
    let current = Version::new(1, 0, 0);
    let mut p = policy(&current);
    p.channel = "beta";
    assert!(matches!(
        manifest().check(&p),
        Err(Error::WrongChannel { .. })
    ));
}

#[test]
fn a_revoked_version_is_never_offered() {
    let current = Version::new(1, 0, 0);
    let mut m = manifest();
    m.revoked = vec!["1.2.0".into()];
    assert_eq!(
        m.check(&policy(&current)).unwrap_err(),
        Error::Revoked("1.2.0".into())
    );
}

#[test]
fn a_missing_target_and_bad_artifacts_are_rejected() {
    let current = Version::new(1, 0, 0);
    let mut p = policy(&current);
    p.target = "linux-x64";
    assert_eq!(
        manifest().check(&p).unwrap_err(),
        Error::NoArtifact("linux-x64".into())
    );

    for mutate in [
        |a: &mut Artifact| a.sha256 = "ABC".into(),
        |a: &mut Artifact| a.urls.clear(),
        |a: &mut Artifact| a.urls = vec!["file:///etc/passwd".into()],
        |a: &mut Artifact| a.size = 0,
    ] {
        let mut m = manifest();
        mutate(&mut m.artifacts[0]);
        assert!(matches!(
            m.check(&policy(&current)),
            Err(Error::BadArtifact(_))
        ));
    }
}

#[test]
fn an_unknown_schema_is_rejected() {
    let current = Version::new(1, 0, 0);
    let mut m = manifest();
    m.schema = 2;
    assert_eq!(
        m.check(&policy(&current)).unwrap_err(),
        Error::UnsupportedSchema(2)
    );
}

#[test]
fn downloads_are_checked_against_size_and_hash() {
    let artifact = manifest().artifacts.remove(0);

    let mut ok = ArtifactCheck::new(&artifact);
    ok.update(b"zip ").unwrap();
    ok.update(b"bytes").unwrap();
    assert_eq!(ok.seen(), 9);
    assert!(ok.finish().is_ok());

    let mut wrong = ArtifactCheck::new(&artifact);
    wrong.update(b"zip byteX").unwrap();
    assert_eq!(wrong.finish(), Err(Error::HashMismatch));

    let mut short = ArtifactCheck::new(&artifact);
    short.update(b"zip").unwrap();
    assert_eq!(
        short.finish(),
        Err(Error::WrongSize {
            expected: 9,
            found: 3
        })
    );

    let mut long = ArtifactCheck::new(&artifact);
    assert_eq!(
        long.update(b"zip bytes and then some"),
        Err(Error::TooLarge { expected: 9 })
    );
}

#[test]
fn a_rotation_signed_by_a_trusted_key_adds_the_new_key_and_retires_the_old() {
    let (old, new) = (key(1), key(2));
    let mut keys = keyset(&[("k1", &old)]);
    let statement = Signed::sign(
        KIND_ROTATION,
        &RotationStatement {
            new_key: KeyEntry::new("k2", &new.verifying_key()),
            retire: vec!["k1".into()],
        },
        "k1",
        &old,
    )
    .unwrap();
    keys.apply_rotation(&statement).unwrap();
    assert_eq!(keys.ids().collect::<Vec<_>>(), ["k2"]);

    assert!(
        signed(&manifest(), &old)
            .verify(&keys, KIND_UPDATE)
            .is_err()
    );
    assert!(
        Signed::sign(KIND_UPDATE, &manifest(), "k2", &new)
            .unwrap()
            .verify(&keys, KIND_UPDATE)
            .is_ok()
    );
}

#[test]
fn a_rotation_from_an_untrusted_key_changes_nothing() {
    let (trusted, attacker) = (key(1), key(9));
    let mut keys = keyset(&[("k1", &trusted)]);
    let statement = Signed::sign(
        KIND_ROTATION,
        &RotationStatement {
            new_key: KeyEntry::new("evil", &attacker.verifying_key()),
            retire: vec!["k1".into()],
        },
        "k1",
        &attacker,
    )
    .unwrap();
    assert_eq!(keys.apply_rotation(&statement), Err(Error::BadSignature));
    assert_eq!(keys.ids().collect::<Vec<_>>(), ["k1"]);
}

#[test]
fn fingerprints_are_stable_and_distinguish_keys() {
    let a = fingerprint(&key(1).verifying_key());
    assert_eq!(a, fingerprint(&key(1).verifying_key()));
    assert_ne!(a, fingerprint(&key(2).verifying_key()));
    assert_eq!(a.split(' ').count(), 16);
}

#[test]
fn a_malformed_key_entry_is_rejected() {
    assert!(
        KeySet::from_entries([KeyEntry {
            id: "x".into(),
            public: "not base64!".into()
        }])
        .is_err()
    );
    assert!(
        KeySet::from_entries([KeyEntry {
            id: "x".into(),
            public: "AAAA".into()
        }])
        .is_err()
    );
}
