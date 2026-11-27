mod forms;
mod instance;

use crate::features::captcha;
use crate::features::title_bar;
use crate::shell;
use requiem_core::config;
use requiem_core::rest;
use requiem_core::ui;
use requiem_ui::form;
use requiem_ui::paint;
use std::fmt;
use std::future;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime;

use requiem_core::auth::{
    self, ApiError, AuthRequestError, CaptchaChallenge, CaptchaSolution, LoginError, RegisterForm,
};
use requiem_core::instances::{Instance, InstanceStore};
use requiem_core::properties::SuperProperties;
use requiem_core::ui::Fonts;
use requiem_ui::components::div;
use requiem_ui::control::section;
use requiem_ui::form::Form;
use requiem_ui::node::Node;
use requiem_ui::signal::{Effect, Signal, signal};
use requiem_ui::style::cascade;
use requiem_ui::widget::anim::ToggleState;
use requiem_ui::widget::primitives::ImageWidget;
use requiem_ui::widget::{WidgetId, WidgetTree};

use crate::shell::classes::auth_screen as cls;

const BACKGROUND_PX: (u32, u32) = (1920, 1200);
const WORDMARK_PX: (u32, u32) = (248, 48);

const MODAL_CLOSE_GRACE: Duration = Duration::from_millis(350);

const PICKER_DOM_ID: &str = "auth-instance-picker";
const DOB_DOM_ID: &str = "auth-dob";

#[derive(Clone, PartialEq)]
enum Screen {
    Login,
    Register,

    Mfa { ticket: String, instance: Instance },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DobPart {
    Month,
    Day,
    Year,
}

#[derive(Clone, Copy, Default, PartialEq)]
struct Dob {
    month: Option<u32>,
    day: Option<u32>,
    year: Option<u32>,
}

pub enum AuthOutcome {
    LoggedIn { instance: Instance, token: String },
    MfaRequired { instance: Instance, ticket: String },
    Failed(ApiError),
    ResetSent { login: String },
}

#[derive(Clone)]
pub struct AuthContext {
    pub fonts: Fonts,
    pub runtime: runtime::Handle,

    pub deliver: Arc<dyn Fn(AuthOutcome) + Send + Sync>,
}

#[derive(Clone)]
pub struct AuthState {
    instances: Signal<InstanceStore>,
    screen: Signal<Screen>,
    submitting: Signal<bool>,

    form: Form,
    dob: Signal<Dob>,
    dob_open: Signal<Option<DobPart>>,
    consent: Signal<ToggleState>,
    picker_open: Signal<bool>,
    add_open: Signal<ToggleState>,

    add_form: Form,
    advanced_open: Signal<bool>,

    reset_notice: Signal<(ToggleState, String)>,
}

impl AuthState {
    pub fn new(instances: InstanceStore) -> Self {
        Self {
            instances: signal(instances),
            screen: signal(Screen::Login),
            submitting: signal(false),
            form: Form::new(Some("login")),
            dob: signal(Dob::default()),
            dob_open: signal(None),
            consent: signal(ToggleState::new(false)),
            picker_open: signal(false),
            add_open: signal(ToggleState::new(false)),
            add_form: Form::new(None),
            advanced_open: signal(false),
            reset_notice: signal((ToggleState::new(false), String::new())),
        }
    }

    pub fn reset_for_logout(&self) {
        self.screen.set(Screen::Login);
        self.submitting.set(false);
        self.form.reset(Some("login"));
        self.dob.set(Dob::default());
        self.dob_open.set(None);
        self.consent.set(ToggleState::new(false));
        self.picker_open.set(false);
        self.add_open.set(ToggleState::new(false));
        self.add_form.reset(None);
        self.reset_notice
            .set((ToggleState::new(false), String::new()));
    }

    fn switch_screen(&self, screen: Screen) {
        self.form.remove("password");
        self.form.clear_errors();
        self.dob_open.set(None);
        self.form.focus_on(match screen {
            Screen::Login => "login",
            Screen::Register => "email",
            Screen::Mfa { .. } => "code",
        });
        self.screen.set(screen);
    }
}

fn modal_visible(toggle: &ToggleState) -> bool {
    toggle.on || toggle.changed_at.elapsed() < MODAL_CLOSE_GRACE
}

pub fn mount(tree: &mut WidgetTree, state: &AuthState, ctx: &AuthContext) -> Vec<Effect> {
    let mut effects = Vec::new();
    let root = tree.root;
    tree.nodes[root].classes.push(cls::ROOT.to_string());

    let background =
        ui::asset_svg("image/auth_background", BACKGROUND_PX.0, BACKGROUND_PX.1);
    tree.mount(
        root,
        Node::new("div", ImageWidget::new(background).with_cover()).class(cls::BACKGROUND),
    );
    let wordmark = ui::asset_svg("image/auth_wordmark", WORDMARK_PX.0, WORDMARK_PX.1);
    tree.mount(
        root,
        Node::new("div", ImageWidget::new(wordmark)).class(cls::WORDMARK),
    );

    {
        let (state, ctx) = (state.clone(), ctx.clone());
        effects.push(section(tree, root, move || forms::build(&state, &ctx)));
    }

    tree.mount(root, title_bar::bare());

    {
        let state = state.clone();
        effects.push(section(tree, root, move || instance::picker(&state)));
    }
    {
        let state = state.clone();
        effects.push(section(tree, root, move || {
            let open = state.add_open.get();
            if !modal_visible(&open) {
                return div();
            }
            instance::add_modal(&state, open)
        }));
    }
    {
        let state = state.clone();
        effects.push(section(tree, root, move || {
            let (open, login) = state.reset_notice.get();
            if !modal_visible(&open) {
                return div();
            }
            forms::reset_notice(&state, open, login)
        }));
    }

    cascade::apply_stylesheet(tree, &shell::load_theme());
    effects
}

pub fn handle_mouse_down(state: &AuthState, tree: &WidgetTree, handled_id: Option<WidgetId>) {
    let inside = |dom_id: &str| {
        handled_id.is_some_and(|id| paint::is_descendant_or_self(tree, id, dom_id))
    };
    if state.picker_open.peek() && !inside(PICKER_DOM_ID) {
        state.picker_open.set(false);
    }
    if state.dob_open.peek().is_some() && !inside(DOB_DOM_ID) {
        state.dob_open.set(None);
    }
}

pub fn apply_outcome(state: &AuthState, outcome: AuthOutcome) -> Option<(Instance, String)> {
    state.submitting.set(false);
    match outcome {
        AuthOutcome::LoggedIn { instance, token } => Some((instance, token)),
        AuthOutcome::MfaRequired { instance, ticket } => {
            state.switch_screen(Screen::Mfa { ticket, instance });
            None
        }
        AuthOutcome::Failed(error) => {
            state.form.set_errors(error.into());
            None
        }
        AuthOutcome::ResetSent { login } => {
            state
                .reset_notice
                .set((ToggleState::new(false).set(true), login));
            None
        }
    }
}

fn to_api_error(err: impl fmt::Display) -> ApiError {
    ApiError {
        message: Some(err.to_string()),
        ..Default::default()
    }
}

fn from_request_error(err: AuthRequestError) -> ApiError {
    match err {
        AuthRequestError::Api(error) => error,
        other => to_api_error(other),
    }
}

const MAX_CAPTCHAS: usize = 2;

async fn with_captcha<Fut>(
    origin: &str,
    attempt: impl FnMut(Option<CaptchaSolution>) -> Fut,
) -> AuthOutcome
where
    Fut: future::Future<Output = Result<AuthOutcome, ApiError>>,
{
    retry_with_captcha(
        |challenge| async move {
            captcha::solve(challenge, origin.to_string())
                .await
                .map_err(|err| err.message())
        },
        attempt,
    )
    .await
}

async fn retry_with_captcha<SolveFut, Fut>(
    mut solve: impl FnMut(CaptchaChallenge) -> SolveFut,
    mut attempt: impl FnMut(Option<CaptchaSolution>) -> Fut,
) -> AuthOutcome
where
    SolveFut: future::Future<Output = Result<CaptchaSolution, String>>,
    Fut: future::Future<Output = Result<AuthOutcome, ApiError>>,
{
    let mut solution = None;
    let mut shown = 0;
    loop {
        match attempt(solution.take()).await {
            Ok(outcome) => return outcome,
            Err(ApiError {
                captcha: Some(challenge),
                ..
            }) if shown < MAX_CAPTCHAS => {
                shown += 1;
                match solve(challenge).await {
                    Ok(answer) => solution = Some(answer),
                    Err(message) => return AuthOutcome::Failed(to_api_error(message)),
                }
            }
            Err(error) => return AuthOutcome::Failed(error),
        }
    }
}

fn spawn_request<F, Fut>(state: &AuthState, ctx: &AuthContext, instance: Instance, request: F)
where
    F: FnOnce(wreq::Client, config::BackendConfig, Instance) -> Fut + Send + 'static,
    Fut: future::Future<Output = AuthOutcome> + Send + 'static,
{
    if state.submitting.peek() {
        return;
    }
    let Some(backend) = instance.backend() else {
        state.form.set_general_error(requiem_i18n::t!(
            "auth-no-base-url",
            name = instance.name.as_str()
        ));
        return;
    };
    state.submitting.set(true);
    let deliver = ctx.deliver.clone();
    ctx.runtime.spawn(async move {
        let outcome = match rest::build_client() {
            Ok(client) => request(client, backend, instance).await,
            Err(err) => AuthOutcome::Failed(to_api_error(err)),
        };
        deliver(outcome);
    });
}

fn submit_login(state: &AuthState, ctx: &AuthContext) {
    if !state.form.require(&["login", "password"]) {
        return;
    }
    let (login, password) = (state.form.value("login"), state.form.value("password"));
    let instance = state.instances.peek().selected_instance();
    spawn_request(
        state,
        ctx,
        instance,
        move |client, backend, instance| async move {
            with_captcha(&backend.web_base_url(), |captcha| {
                let (client, backend, instance, login, password) =
                    (&client, &backend, &instance, &login, &password);
                async move {
                    match auth::login(
                        client,
                        backend,
                        &SuperProperties::desktop_like(),
                        login,
                        password,
                        captcha.as_ref(),
                    )
                    .await
                    {
                        Ok(token) => Ok(AuthOutcome::LoggedIn {
                            instance: instance.clone(),
                            token,
                        }),
                        Err(LoginError::MfaRequired { ticket }) => Ok(AuthOutcome::MfaRequired {
                            instance: instance.clone(),
                            ticket,
                        }),
                        Err(LoginError::Api { error, .. }) => Err(error),
                        Err(other) => Err(to_api_error(other)),
                    }
                }
            })
            .await
        },
    );
}

fn submit_register(state: &AuthState, ctx: &AuthContext) {
    let dob = state.dob.peek();
    let fields_ok = state.form.require(&["email", "username", "password"]);
    let (Some(month), Some(day), Some(year)) = (dob.month, dob.day, dob.year) else {
        state
            .form
            .set_error("date_of_birth", form::REQUIRED_MESSAGE);
        return;
    };
    if !fields_ok {
        return;
    }
    if !state.consent.peek().on {
        state
            .form
            .set_general_error(requiem_i18n::t!("auth-you-need-to-agree-to-our"));
        return;
    }
    let (email, global_name, username, password) = (
        state.form.value("email"),
        state.form.value("global_name"),
        state.form.value("username"),
        state.form.value("password"),
    );
    let date_of_birth = format!("{year:04}-{month:02}-{day:02}");
    let instance = state.instances.peek().selected_instance();
    spawn_request(
        state,
        ctx,
        instance,
        move |client, backend, instance| async move {
            with_captcha(&backend.web_base_url(), |captcha| {
                let (client, backend, instance) = (&client, &backend, &instance);
                let form = RegisterForm {
                    email: &email,
                    global_name: Some(&global_name),
                    username: &username,
                    password: &password,
                    date_of_birth: &date_of_birth,
                    consent: true,
                };
                async move {
                    match auth::register(
                        client,
                        backend,
                        &SuperProperties::desktop_like(),
                        form,
                        captcha.as_ref(),
                    )
                    .await
                    {
                        Ok(token) => Ok(AuthOutcome::LoggedIn {
                            instance: instance.clone(),
                            token,
                        }),
                        Err(err) => Err(from_request_error(err)),
                    }
                }
            })
            .await
        },
    );
}

fn submit_mfa(state: &AuthState, ctx: &AuthContext, ticket: String, instance: Instance) {
    if !state.form.require(&["code"]) {
        return;
    }
    let code = state.form.value("code");
    spawn_request(
        state,
        ctx,
        instance,
        move |client, backend, instance| async move {
            match auth::mfa_login(
                &client,
                &backend,
                &SuperProperties::desktop_like(),
                &ticket,
                &code,
            )
            .await
            {
                Ok(token) => AuthOutcome::LoggedIn { instance, token },
                Err(err) => AuthOutcome::Failed(from_request_error(err)),
            }
        },
    );
}

fn forgot_password(state: &AuthState, ctx: &AuthContext) {
    if !state.form.require(&["login"]) {
        return;
    }
    let login = state.form.value("login");
    let instance = state.instances.peek().selected_instance();
    spawn_request(
        state,
        ctx,
        instance,
        move |client, backend, _instance| async move {
            with_captcha(&backend.web_base_url(), |captcha| {
                let (client, backend, login) = (&client, &backend, &login);
                async move {
                    match auth::forgot_password(
                        client,
                        backend,
                        &SuperProperties::desktop_like(),
                        login,
                        captcha.as_ref(),
                    )
                    .await
                    {
                        Ok(()) => Ok(AuthOutcome::ResetSent {
                            login: login.clone(),
                        }),
                        Err(err) => Err(from_request_error(err)),
                    }
                }
            })
            .await
        },
    );
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use requiem_core::auth::CaptchaService;

    use super::*;

    fn challenge() -> CaptchaChallenge {
        CaptchaChallenge {
            service: CaptchaService::Hcaptcha,
            sitekey: Some("site".into()),
            rqdata: None,
            rqtoken: None,
            session_id: None,
            invisible: false,
            reasons: vec![],
        }
    }

    fn captcha_error() -> ApiError {
        ApiError {
            captcha: Some(challenge()),
            message: Some("refused".into()),
            ..Default::default()
        }
    }

    fn run<F: future::Future>(future: F) -> F::Output {
        runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(future)
    }

    fn solved(challenge: CaptchaChallenge) -> future::Ready<Result<CaptchaSolution, String>> {
        future::ready(Ok(CaptchaSolution::new(&challenge, "answer".into())))
    }

    #[test]
    fn a_request_without_a_captcha_is_sent_once_and_shows_no_window() {
        let (attempts, windows) = (Cell::new(0), Cell::new(0));
        let outcome = run(retry_with_captcha(
            |c| {
                windows.set(windows.get() + 1);
                solved(c)
            },
            |_| {
                attempts.set(attempts.get() + 1);
                future::ready(Ok(AuthOutcome::ResetSent { login: "a".into() }))
            },
        ));
        assert!(matches!(outcome, AuthOutcome::ResetSent { .. }));
        assert_eq!((attempts.get(), windows.get()), (1, 0));
    }

    #[test]
    fn a_challenge_is_solved_and_the_same_request_is_sent_again_with_the_answer() {
        let sent = Cell::new(0);
        let outcome = run(retry_with_captcha(solved, |answer| {
            sent.set(sent.get() + 1);
            future::ready(match answer {
                None => Err(captcha_error()),
                Some(answer) => {
                    assert_eq!(answer.key, "answer");
                    Ok(AuthOutcome::ResetSent { login: "a".into() })
                }
            })
        }));
        assert!(matches!(outcome, AuthOutcome::ResetSent { .. }));
        assert_eq!(sent.get(), 2);
    }

    #[test]
    fn a_server_that_keeps_refusing_gets_two_windows_and_then_its_own_answer() {
        let (attempts, windows) = (Cell::new(0), Cell::new(0));
        let outcome = run(retry_with_captcha(
            |c| {
                windows.set(windows.get() + 1);
                solved(c)
            },
            |_| {
                attempts.set(attempts.get() + 1);
                future::ready(Err(captcha_error()))
            },
        ));

        assert_eq!(
            (windows.get(), attempts.get()),
            (MAX_CAPTCHAS, MAX_CAPTCHAS + 1)
        );
        assert!(
            matches!(outcome, AuthOutcome::Failed(error) if error.message.as_deref() == Some("refused"))
        );
    }

    #[test]
    fn closing_the_window_ends_the_submit_without_another_request() {
        let attempts = Cell::new(0);
        let outcome = run(retry_with_captcha(
            |_| future::ready(Err("not completed".to_string())),
            |_| {
                attempts.set(attempts.get() + 1);
                future::ready(Err(captcha_error()))
            },
        ));
        assert_eq!(attempts.get(), 1);
        assert!(
            matches!(outcome, AuthOutcome::Failed(error) if error.message.as_deref() == Some("not completed"))
        );
    }

    #[test]
    fn other_errors_pass_straight_through() {
        let error = ApiError {
            status: 400,
            message: Some("Invalid login".into()),
            ..Default::default()
        };
        let outcome = run(retry_with_captcha(solved, |_| {
            future::ready(Err(error.clone()))
        }));
        assert!(matches!(outcome, AuthOutcome::Failed(e) if e.status == 400));
    }
}
