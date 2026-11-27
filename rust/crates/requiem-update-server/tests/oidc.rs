use base64::Engine as _;
use std::sync::Arc;

use async_trait::async_trait;
use base64::engine::general_purpose;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header};
use requiem_update_server::auth::{Authenticator, GithubOidc, KeyProvider, OidcPolicy};
use requiem_update_server::error::ApiError;
use serde_json::{Value, json};
use std::time;
use std::time::SystemTime;

const PRIVATE: &str = include_str!("data/test-only-rsa-private.pem");
const PUBLIC: &str = include_str!("data/test-only-rsa-public.pem");
const AUDIENCE: &str = "https://updates.example.test/api";

struct Keys;

#[async_trait]
impl KeyProvider for Keys {
    async fn key(&self, kid: &str) -> Option<DecodingKey> {
        (kid == "test-kid").then(|| DecodingKey::from_rsa_pem(PUBLIC.as_bytes()).unwrap())
    }
}

fn policy() -> OidcPolicy {
    OidcPolicy {
        issuer: "https://token.actions.githubusercontent.com".into(),
        audience: AUDIENCE.into(),
        repository: "owner/repo".into(),
        environment: Some("update-signing".into()),
        workflows: vec![
            "owner/repo/.github/workflows/release.yml@".into(),
            "owner/repo/.github/workflows/desktop-identity.yml@".into(),
        ],
    }
}

fn auth(policy: OidcPolicy) -> GithubOidc {
    GithubOidc {
        policy,
        keys: Arc::new(Keys),
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn good_claims() -> Value {
    json!({
        "iss": "https://token.actions.githubusercontent.com",
        "aud": AUDIENCE,
        "exp": now() + 300,
        "sub": "repo:owner/repo:environment:update-signing",
        "repository": "owner/repo",
        "environment": "update-signing",
        "job_workflow_ref": "owner/repo/.github/workflows/release.yml@refs/tags/v1.0.0",
    })
}

fn token(claims: &Value, kid: &str) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.into());
    jsonwebtoken::encode(
        &header,
        claims,
        &EncodingKey::from_rsa_pem(PRIVATE.as_bytes()).unwrap(),
    )
    .unwrap()
}

fn with(mut claims: Value, key: &str, value: Value) -> Value {
    claims[key] = value;
    claims
}

fn without(mut claims: Value, key: &str) -> Value {
    claims.as_object_mut().unwrap().remove(key);
    claims
}

fn status(result: Result<String, ApiError>) -> u16 {
    match result {
        Ok(_) => 200,
        Err(ApiError::Unauthorized(_)) => 401,
        Err(ApiError::Forbidden(_)) => 403,
        Err(other) => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn a_job_of_the_right_repository_environment_and_workflow_is_accepted() {
    let who = auth(policy())
        .authenticate(&token(&good_claims(), "test-kid"))
        .await
        .unwrap();
    assert_eq!(who, "repo:owner/repo:environment:update-signing");

    let identity = with(
        good_claims(),
        "job_workflow_ref",
        json!("owner/repo/.github/workflows/desktop-identity.yml@refs/heads/main"),
    );
    assert!(
        auth(policy())
            .authenticate(&token(&identity, "test-kid"))
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn tokens_that_are_not_for_this_server_or_not_from_the_right_job_are_refused() {
    let a = auth(policy());
    let cases: Vec<(&str, String, u16)> = vec![
        (
            "another audience",
            token(
                &with(good_claims(), "aud", json!("https://evil.example")),
                "test-kid",
            ),
            401,
        ),
        (
            "another issuer",
            token(
                &with(good_claims(), "iss", json!("https://evil.example")),
                "test-kid",
            ),
            401,
        ),
        (
            "expired",
            token(&with(good_claims(), "exp", json!(now() - 3600)), "test-kid"),
            401,
        ),
        (
            "no expiry",
            token(&without(good_claims(), "exp"), "test-kid"),
            401,
        ),
        (
            "an unknown signing key",
            token(&good_claims(), "other-kid"),
            401,
        ),
        (
            "another repository",
            token(
                &with(good_claims(), "repository", json!("someone/else")),
                "test-kid",
            ),
            403,
        ),
        (
            "no repository",
            token(&without(good_claims(), "repository"), "test-kid"),
            403,
        ),
        (
            "another environment",
            token(
                &with(good_claims(), "environment", json!("production")),
                "test-kid",
            ),
            403,
        ),
        (
            "no environment",
            token(&without(good_claims(), "environment"), "test-kid"),
            403,
        ),
        (
            "another workflow",
            token(
                &with(
                    good_claims(),
                    "job_workflow_ref",
                    json!("owner/repo/.github/workflows/other.yml@refs/heads/main"),
                ),
                "test-kid",
            ),
            403,
        ),
        (
            "a workflow of another repository with a similar name",
            token(
                &with(
                    good_claims(),
                    "job_workflow_ref",
                    json!("evil/repo/.github/workflows/release.yml@refs/heads/main"),
                ),
                "test-kid",
            ),
            403,
        ),
        ("not a token", "garbage".into(), 401),
        ("empty", String::new(), 401),
    ];
    for (name, token, expected) in cases {
        assert_eq!(status(a.authenticate(&token).await), expected, "{name}");
    }
}

#[tokio::test]
async fn an_unsigned_or_differently_signed_token_is_refused() {
    let a = auth(policy());

    let enc = |v: &Value| general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(v).unwrap());
    let none = format!(
        "{}.{}.",
        enc(&json!({"alg": "none", "kid": "test-kid"})),
        enc(&good_claims())
    );
    assert_eq!(status(a.authenticate(&none).await), 401);

    let genuine = token(&good_claims(), "test-kid");
    let (head, _) = genuine.rsplit_once('.').unwrap();
    assert_eq!(status(a.authenticate(&format!("{head}.AAAA")).await), 401);

    let mut header = Header::new(Algorithm::HS256);
    header.kid = Some("test-kid".into());
    let confused = jsonwebtoken::encode(
        &header,
        &good_claims(),
        &EncodingKey::from_secret(PUBLIC.as_bytes()),
    )
    .unwrap();
    assert_eq!(status(a.authenticate(&confused).await), 401);
}

#[tokio::test]
async fn without_an_environment_or_workflow_restriction_only_the_repository_counts() {
    let loose = OidcPolicy {
        environment: None,
        workflows: vec![],
        ..policy()
    };
    let a = auth(loose);
    assert!(
        a.authenticate(&token(
            &without(without(good_claims(), "environment"), "job_workflow_ref"),
            "test-kid"
        ))
        .await
        .is_ok()
    );
    assert_eq!(
        status(
            a.authenticate(&token(
                &with(good_claims(), "repository", json!("x/y")),
                "test-kid"
            ))
            .await
        ),
        403
    );
}
