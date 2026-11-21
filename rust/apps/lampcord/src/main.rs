mod backend;
mod ui;

use std::{collections::HashSet, env, ops::Range, path::PathBuf, time::Duration};

use backend::{BackendCommand, BackendHandle};
use gpui::{
    App, Bounds, ClipboardItem, Context, EntityInputHandler, FocusHandle, Focusable, IntoElement,
    KeyDownEvent, ParentElement, Pixels, Point, Render, SharedString, Styled, UTF16Selection,
    Window, WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use gpui_platform::application;
use ui::login::{LoginField, LoginStep, LoginUiState, handle_login_key, render_login};
use ui::theme::{self, M3Theme};
use unicode_segmentation::UnicodeSegmentation;

fn token_store_path() -> PathBuf {
    if let Ok(dir) = env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(dir).join("lampcord").join("accounts.json");
    }
    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("lampcord")
            .join("accounts.json");
    }
    PathBuf::from("lampcord-accounts.json")
}

fn initial_token() -> Option<String> {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--token" {
            if let Some(token) = args.next() {
                if !token.trim().is_empty() {
                    return Some(token);
                }
            }
        } else if let Some(value) = arg.strip_prefix("--token=") {
            if !value.trim().is_empty() {
                return Some(value.to_owned());
            }
        }
    }
    for key in ["LAMPCORD_TOKEN", "DISCORD_TOKEN"] {
        if let Ok(token) = env::var(key) {
            if !token.trim().is_empty() {
                return Some(token);
            }
        }
    }
    None
}

#[derive(Debug, Clone)]
pub(crate) struct ComposerReplyTarget {
    pub message_id: u64,
    pub channel_id: u64,
    pub guild_id: Option<u64>,
    pub author: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ComposerEditTarget {
    pub message_id: u64,
    pub channel_id: u64,
    pub original_content: String,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingAttachment {
    pub path: PathBuf,
    pub filename: String,
    pub size: u64,
    pub content_type: Option<String>,
}

pub(crate) struct Root {
    pub(crate) backend: BackendHandle,
    pub(crate) token: Option<String>,
    pub(crate) requested_channel: Option<u64>,
    pub(crate) composer: String,
    pub(crate) composer_selected_range: Range<usize>,
    pub(crate) composer_selection_reversed: bool,
    pub(crate) composer_marked_range: Option<Range<usize>>,
    pub(crate) composer_last_bounds: Option<Bounds<Pixels>>,
    pub(crate) composer_is_selecting: bool,
    pub(crate) composer_reply: Option<ComposerReplyTarget>,
    pub(crate) composer_edit: Option<ComposerEditTarget>,
    pub(crate) pending_attachments: Vec<PendingAttachment>,
    pub(crate) focus_handle: FocusHandle,
    pub(crate) login: LoginUiState,
    pub(crate) theme: M3Theme,
    pub(crate) show_login: bool,
    pub(crate) login_explicitly_opened: bool,
    pub(crate) show_devices_panel: bool,
    pub(crate) show_member_panel: bool,
    pub(crate) show_debug_stats: bool,
    pub(crate) device_challenge_input: String,
    pub(crate) page: ui::shell::AppPage,
    pub(crate) settings_page: ui::shell::SettingsPage,
    pub(crate) expanded_guild_folders: HashSet<i64>,
    pub(crate) collapsed_categories: HashSet<u64>,
    pub(crate) message_list_state: gpui::ListState,
    pub(crate) message_list_channel_id: Option<u64>,
    pub(crate) message_list_row_count: usize,
    pub(crate) message_list_first_id: Option<u64>,
    pub(crate) message_list_last_id: Option<u64>,
    pub(crate) message_list_last_read: Option<u64>,
}

impl Focusable for Root {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Root {
    fn composer_len_utf16(text: &str) -> usize {
        text.encode_utf16().count()
    }

    fn composer_byte_offset_from_utf16(text: &str, utf16_offset: usize) -> usize {
        if utf16_offset == 0 {
            return 0;
        }
        let mut utf16_count = 0usize;
        for (byte_idx, ch) in text.char_indices() {
            let ch_utf16_len = ch.len_utf16();
            if utf16_count + ch_utf16_len > utf16_offset {
                return byte_idx;
            }
            utf16_count += ch_utf16_len;
            if utf16_count == utf16_offset {
                return byte_idx + ch.len_utf8();
            }
        }
        text.len()
    }

    fn composer_utf16_offset_from_byte(text: &str, byte_offset: usize) -> usize {
        let byte_offset = byte_offset.min(text.len());
        text[..byte_offset].encode_utf16().count()
    }

    fn composer_range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        Self::composer_utf16_offset_from_byte(&self.composer, range.start)
            ..Self::composer_utf16_offset_from_byte(&self.composer, range.end)
    }

    fn composer_range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        let start = Self::composer_byte_offset_from_utf16(&self.composer, range_utf16.start);
        let end = Self::composer_byte_offset_from_utf16(&self.composer, range_utf16.end);
        start.min(end)..start.max(end)
    }

    fn set_composer_text(&mut self, text: String) {
        self.composer = text;
        let len = self.composer.len();
        self.composer_selected_range = len..len;
        self.composer_selection_reversed = false;
        self.composer_marked_range = None;
    }

    fn clear_composer_draft(&mut self) {
        self.composer.clear();
        self.composer_selected_range = 0..0;
        self.composer_selection_reversed = false;
        self.composer_marked_range = None;
        self.composer_reply = None;
        self.composer_edit = None;
        self.pending_attachments.clear();
    }

    fn begin_reply(&mut self, target: ComposerReplyTarget) {
        self.composer_reply = Some(target);
        self.composer_edit = None;
    }

    fn begin_edit(&mut self, target: ComposerEditTarget) {
        self.composer_edit = Some(ComposerEditTarget {
            message_id: target.message_id,
            channel_id: target.channel_id,
            original_content: target.original_content.clone(),
        });
        self.composer_reply = None;
        self.set_composer_text(target.original_content);
    }

    fn cancel_reply_or_edit(&mut self) {
        if self.composer_edit.is_some() {
            self.composer_reply = None;
            self.composer_edit = None;
            self.set_composer_text(String::new());
        } else {
            self.composer_reply = None;
        }
    }

    fn composer_cursor_offset(&self) -> usize {
        if self.composer_selection_reversed {
            self.composer_selected_range.start
        } else {
            self.composer_selected_range.end
        }
    }

    fn composer_move_to(&mut self, offset: usize) {
        let offset = offset.min(self.composer.len());
        self.composer_selected_range = offset..offset;
        self.composer_selection_reversed = false;
    }

    fn composer_select_to(&mut self, offset: usize) {
        let offset = offset.min(self.composer.len());
        if self.composer_selection_reversed {
            self.composer_selected_range.start = offset;
        } else {
            self.composer_selected_range.end = offset;
        }
        if self.composer_selected_range.end < self.composer_selected_range.start {
            self.composer_selection_reversed = !self.composer_selection_reversed;
            self.composer_selected_range =
                self.composer_selected_range.end..self.composer_selected_range.start;
        }
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        if offset == 0 {
            return 0;
        }
        self.composer[..offset]
            .grapheme_indices(true)
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        if offset >= self.composer.len() {
            return self.composer.len();
        }
        self.composer[offset..]
            .grapheme_indices(true)
            .nth(1)
            .map(|(i, _)| offset + i)
            .unwrap_or(self.composer.len())
    }

    fn offset_from_point(&self, point: Point<Pixels>, window: &Window) -> usize {
        let Some(bounds) = self.composer_last_bounds else {
            return self.composer.len();
        };
        let font_size = px(14.);
        let style = window.text_style();
        let run = style.clone().to_run(self.composer.len());
        let line = window.text_system().shape_line(
            SharedString::from(self.composer.clone()),
            font_size,
            &[run],
            None,
        );
        let local_x = point.x - bounds.left();
        line.closest_index_for_x(local_x.max(px(0.)))
    }
}

impl Root {
    fn maybe_auto_connect(&mut self) {
        if let Some(token) = self.token.clone() {
            let snapshot = self.backend.snapshot();
            if snapshot.status == "Idle" || snapshot.status == "Logged out" {
                self.backend.send(BackendCommand::Connect { token });
                self.token = None;
            }
        }
    }

    fn is_chat_ready(&self, status: &str) -> bool {
        matches!(status, "Ready" | "Resumed") || status.starts_with("Connected")
    }

    fn select_guild(&mut self, guild_id: u64, cx: &mut Context<Self>) {
        self.backend.send(BackendCommand::SelectGuild {
            guild_id: Some(guild_id),
        });
        self.requested_channel = None;
        self.clear_composer_draft();
        cx.notify();
    }

    fn select_channel(&mut self, channel_id: u64, cx: &mut Context<Self>) {
        self.backend.send(BackendCommand::SelectChannel {
            channel_id: Some(channel_id),
        });
        self.backend.send(BackendCommand::FetchMessages {
            channel_id,
            before: None,
        });
        self.requested_channel = Some(channel_id);
        self.clear_composer_draft();
        cx.notify();
    }

    fn submit_token_login(&mut self, cx: &mut Context<Self>) {
        let token = self.login.token.trim().to_owned();
        if token.is_empty() {
            self.login.local_error = Some("Please enter a token.".to_owned());
            cx.notify();
            return;
        }
        self.login.local_error = None;
        self.backend.send(BackendCommand::Connect { token });
        self.login.token.clear();
        cx.notify();
    }

    fn submit_credentials_login(&mut self, cx: &mut Context<Self>) {
        let email = self.login.email.trim().to_owned();
        let password = self.login.password.clone();
        if email.is_empty() || password.is_empty() {
            self.login.local_error = Some("Please enter your email/phone and password.".to_owned());
            cx.notify();
            return;
        }
        self.login.local_error = None;
        self.backend.send(BackendCommand::LoginPassword {
            login: email,
            password,
        });
        cx.notify();
    }

    fn submit_mfa_login(&mut self, cx: &mut Context<Self>) {
        let code: String = self
            .login
            .mfa_code
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '-')
            .collect();
        if code.is_empty() {
            self.login.local_error = Some("Please enter your authentication code.".to_owned());
            cx.notify();
            return;
        }
        self.login.local_error = None;
        self.backend.send(BackendCommand::SubmitMfa { code });
        cx.notify();
    }

    fn send_composer(&mut self, cx: &mut Context<Self>) {
        let content = self.composer.trim().to_owned();
        let snapshot = self.backend.snapshot();
        if !self.is_chat_ready(&snapshot.status) {
            return;
        }
        let Some(channel_id) = snapshot.selected_channel_id else {
            return;
        };

        if let Some(edit) = self.composer_edit.clone() {
            if content.is_empty() || edit.channel_id != channel_id {
                return;
            }
            self.backend.send(BackendCommand::EditMessage {
                channel_id,
                message_id: edit.message_id,
                content,
            });
            self.clear_composer_draft();
            cx.notify();
            return;
        }

        if content.is_empty() && self.pending_attachments.is_empty() {
            return;
        }
        if content.is_empty() {
            // Attachments-only send not wired yet; keep draft for now.
            return;
        }

        let reply_to_message_id = self
            .composer_reply
            .as_ref()
            .filter(|r| r.channel_id == channel_id)
            .map(|r| r.message_id);
        let reply_to_guild_id = self
            .composer_reply
            .as_ref()
            .filter(|r| r.channel_id == channel_id)
            .and_then(|r| r.guild_id);

        self.backend.send(BackendCommand::SendMessage {
            channel_id,
            content,
            reply_to_message_id,
            reply_to_guild_id,
        });
        self.clear_composer_draft();
        cx.notify();
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        if key == "f3" {
            self.show_debug_stats = !self.show_debug_stats;
            ui::debug_stats::set_overlay(self.show_debug_stats);
            if self.show_debug_stats {
                eprintln!("[debug-stats] overlay enabled");
            }
            cx.notify();
            return;
        }

        let snapshot = self.backend.snapshot();
        let chat_ready = self.is_chat_ready(&snapshot.status);

        if self.show_login || !chat_ready {
            if handle_login_key(self, event, cx) {
                return;
            }
            return;
        }

        let key = event.keystroke.key.as_str();
        if self.show_devices_panel {
            let snapshot = self.backend.snapshot();
            let has_challenge = snapshot.device_sessions.challenge.is_some();
            if key == "escape" {
                if has_challenge {
                    self.backend
                        .send(BackendCommand::CancelDeviceSessionChallenge);
                    self.device_challenge_input.clear();
                } else {
                    self.show_devices_panel = false;
                }
                cx.notify();
                return;
            }
            if has_challenge {
                match key {
                    "enter" => {
                        let input = self.device_challenge_input.trim().to_owned();
                        if !input.is_empty() {
                            match snapshot.device_sessions.challenge {
                                Some(backend::DeviceSessionChallenge::Password { .. }) => {
                                    self.backend.send(
                                        BackendCommand::SubmitDeviceSessionPassword {
                                            password: input,
                                        },
                                    );
                                }
                                Some(backend::DeviceSessionChallenge::Totp { .. })
                                | Some(backend::DeviceSessionChallenge::BackupCode { .. }) => {
                                    self.backend.send(BackendCommand::SubmitDeviceSessionCode {
                                        code: input,
                                    });
                                }
                                None => {}
                            }
                            self.device_challenge_input.clear();
                            cx.notify();
                        }
                        return;
                    }
                    "backspace" => {
                        self.device_challenge_input.pop();
                        cx.notify();
                        return;
                    }
                    "v" if event.keystroke.modifiers.control
                        || event.keystroke.modifiers.platform =>
                    {
                        if let Some(item) = cx.read_from_clipboard() {
                            if let Some(text) = item.text() {
                                let text = text.replace('\n', "").replace('\r', "");
                                self.device_challenge_input.push_str(&text);
                                cx.notify();
                            }
                        }
                        return;
                    }
                    _ => {}
                }
                if let Some(text) = event.keystroke.key_char.as_ref() {
                    if !event.keystroke.modifiers.control && !event.keystroke.modifiers.platform {
                        self.device_challenge_input.push_str(text);
                        cx.notify();
                        return;
                    }
                } else if key == "space" {
                    if !event.keystroke.modifiers.control && !event.keystroke.modifiers.platform {
                        self.device_challenge_input.push(' ');
                        cx.notify();
                        return;
                    }
                }
                return;
            }
            return;
        }

        if self.page == ui::shell::AppPage::Settings {
            let snapshot = self.backend.snapshot();
            let has_challenge = snapshot.device_sessions.challenge.is_some()
                && self.settings_page == ui::shell::SettingsPage::Devices;
            if key == "escape" {
                if has_challenge {
                    self.backend
                        .send(BackendCommand::CancelDeviceSessionChallenge);
                    self.device_challenge_input.clear();
                } else {
                    self.page = ui::shell::AppPage::Main;
                }
                cx.notify();
                return;
            }
            if has_challenge {
                match key {
                    "enter" => {
                        let input = self.device_challenge_input.trim().to_owned();
                        if !input.is_empty() {
                            match snapshot.device_sessions.challenge {
                                Some(backend::DeviceSessionChallenge::Password { .. }) => {
                                    self.backend.send(
                                        BackendCommand::SubmitDeviceSessionPassword {
                                            password: input,
                                        },
                                    );
                                }
                                Some(backend::DeviceSessionChallenge::Totp { .. })
                                | Some(backend::DeviceSessionChallenge::BackupCode { .. }) => {
                                    self.backend.send(BackendCommand::SubmitDeviceSessionCode {
                                        code: input,
                                    });
                                }
                                None => {}
                            }
                            self.device_challenge_input.clear();
                            cx.notify();
                        }
                        return;
                    }
                    "backspace" => {
                        self.device_challenge_input.pop();
                        cx.notify();
                        return;
                    }
                    "v" if event.keystroke.modifiers.control
                        || event.keystroke.modifiers.platform =>
                    {
                        if let Some(item) = cx.read_from_clipboard() {
                            if let Some(text) = item.text() {
                                let text = text.replace('\n', "").replace('\r', "");
                                self.device_challenge_input.push_str(&text);
                                cx.notify();
                            }
                        }
                        return;
                    }
                    _ => {}
                }
                if let Some(text) = event.keystroke.key_char.as_ref() {
                    if !event.keystroke.modifiers.control && !event.keystroke.modifiers.platform {
                        self.device_challenge_input.push_str(text);
                        cx.notify();
                        return;
                    }
                } else if key == "space" {
                    if !event.keystroke.modifiers.control && !event.keystroke.modifiers.platform {
                        self.device_challenge_input.push(' ');
                        cx.notify();
                        return;
                    }
                }
                return;
            }
            return;
        }

        match key {
            "enter" => {
                if !event.keystroke.modifiers.shift {
                    self.send_composer(cx);
                } else {
                    self.composer_replace_text_in_range(None, "\n");
                    cx.notify();
                }
            }
            "escape" => {
                if self.composer_reply.is_some() || self.composer_edit.is_some() {
                    self.cancel_reply_or_edit();
                } else {
                    self.set_composer_text(String::new());
                }
                cx.notify();
            }
            "backspace" => {
                self.composer_backspace();
                cx.notify();
            }
            "delete" => {
                self.composer_delete();
                cx.notify();
            }
            "left" => {
                if event.keystroke.modifiers.shift {
                    self.composer_select_left();
                } else {
                    self.composer_move_left();
                }
                cx.notify();
            }
            "right" => {
                if event.keystroke.modifiers.shift {
                    self.composer_select_right();
                } else {
                    self.composer_move_right();
                }
                cx.notify();
            }
            "up" => {
                self.composer_move_to(0);
                cx.notify();
            }
            "down" => {
                self.composer_move_to(self.composer.len());
                cx.notify();
            }
            "home" => {
                self.composer_move_to(0);
                cx.notify();
            }
            "end" => {
                self.composer_move_to(self.composer.len());
                cx.notify();
            }
            "a" if event.keystroke.modifiers.control || event.keystroke.modifiers.platform => {
                self.composer_selected_range = 0..self.composer.len();
                self.composer_selection_reversed = false;
                cx.notify();
            }
            "v" if event.keystroke.modifiers.control || event.keystroke.modifiers.platform => {
                if let Some(item) = cx.read_from_clipboard() {
                    if let Some(text) = item.text() {
                        self.composer_replace_text_in_range(None, &text);
                        cx.notify();
                    }
                }
            }
            "c" if event.keystroke.modifiers.control || event.keystroke.modifiers.platform => {
                if !self.composer_selected_range.is_empty() {
                    let text = self.composer[self.composer_selected_range.clone()].to_owned();
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                }
            }
            "x" if event.keystroke.modifiers.control || event.keystroke.modifiers.platform => {
                if !self.composer_selected_range.is_empty() {
                    let text = self.composer[self.composer_selected_range.clone()].to_owned();
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                    self.composer_replace_text_in_range(None, "");
                    cx.notify();
                }
            }
            _ => {
                if let Some(text) = event.keystroke.key_char.as_ref() {
                    if !event.keystroke.modifiers.control && !event.keystroke.modifiers.platform {
                        self.composer_replace_text_in_range(None, text);
                        cx.notify();
                    }
                } else if key == "space" {
                    if !event.keystroke.modifiers.control && !event.keystroke.modifiers.platform {
                        self.composer_replace_text_in_range(None, " ");
                        cx.notify();
                    }
                }
            }
        }
    }

    pub(crate) fn composer_replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.composer_range_from_utf16(range_utf16))
            .or(self.composer_marked_range.clone())
            .unwrap_or(self.composer_selected_range.clone());

        let range_start = range.start.min(self.composer.len());
        let range_end = range.end.min(self.composer.len()).max(range_start);
        self.composer
            .replace_range(range_start..range_end, new_text);
        let new_cursor = range_start + new_text.len();
        self.composer_selected_range = new_cursor..new_cursor;
        self.composer_selection_reversed = false;
        self.composer_marked_range = None;
    }

    pub(crate) fn composer_replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.composer_range_from_utf16(range_utf16))
            .or(self.composer_marked_range.clone())
            .unwrap_or(self.composer_selected_range.clone());

        let range_start = range.start.min(self.composer.len());
        let range_end = range.end.min(self.composer.len()).max(range_start);
        self.composer
            .replace_range(range_start..range_end, new_text);

        if !new_text.is_empty() {
            self.composer_marked_range = Some(range_start..range_start + new_text.len());
        } else {
            self.composer_marked_range = None;
        }

        if let Some(sel_utf16) = new_selected_range_utf16 {
            let rel = self.composer_range_from_utf16(&sel_utf16);
            let start = (range_start + rel.start).min(self.composer.len());
            let end = (range_start + rel.end).min(self.composer.len());
            self.composer_selected_range = start.min(end)..start.max(end);
            self.composer_selection_reversed = sel_utf16.start > sel_utf16.end;
        } else {
            let cursor = range_start + new_text.len();
            self.composer_selected_range = cursor..cursor;
            self.composer_selection_reversed = false;
        }
    }

    fn composer_unmark_text(&mut self) {
        self.composer_marked_range = None;
    }

    fn composer_backspace(&mut self) {
        if self.composer_selected_range.is_empty() {
            let cursor = self.composer_cursor_offset();
            let prev = self.previous_boundary(cursor);
            if prev < cursor {
                self.composer_selected_range = prev..cursor;
                self.composer_replace_text_in_range(None, "");
            }
        } else {
            self.composer_replace_text_in_range(None, "");
        }
    }

    fn composer_delete(&mut self) {
        if self.composer_selected_range.is_empty() {
            let cursor = self.composer_cursor_offset();
            let next = self.next_boundary(cursor);
            if next > cursor {
                self.composer_selected_range = cursor..next;
                self.composer_replace_text_in_range(None, "");
            }
        } else {
            self.composer_replace_text_in_range(None, "");
        }
    }

    fn composer_move_left(&mut self) {
        if self.composer_selected_range.is_empty() {
            let cursor = self.previous_boundary(self.composer_cursor_offset());
            self.composer_move_to(cursor);
        } else {
            self.composer_move_to(self.composer_selected_range.start);
        }
    }

    fn composer_move_right(&mut self) {
        if self.composer_selected_range.is_empty() {
            let cursor = self.next_boundary(self.composer_cursor_offset());
            self.composer_move_to(cursor);
        } else {
            self.composer_move_to(self.composer_selected_range.end);
        }
    }

    fn composer_select_left(&mut self) {
        let cursor = self.previous_boundary(self.composer_cursor_offset());
        self.composer_select_to(cursor);
    }

    fn composer_select_right(&mut self) {
        let cursor = self.next_boundary(self.composer_cursor_offset());
        self.composer_select_to(cursor);
    }

    pub(crate) fn composer_bounds_for_range_utf16(
        &self,
        range_utf16: Range<usize>,
        window: &Window,
    ) -> Option<Bounds<Pixels>> {
        let bounds = self.composer_last_bounds?;
        let range = self.composer_range_from_utf16(&range_utf16);
        let font_size = px(14.);
        let style = window.text_style();
        let run = style.clone().to_run(self.composer.len());
        let line = window.text_system().shape_line(
            SharedString::from(self.composer.clone()),
            font_size,
            &[run],
            None,
        );
        let start_x = line.x_for_index(range.start.min(self.composer.len()));
        let end_x = line.x_for_index(range.end.min(self.composer.len()));
        let line_height = window.line_height();
        Some(Bounds::from_corners(
            Point::new(bounds.left() + start_x, bounds.top()),
            Point::new(bounds.left() + end_x, bounds.top() + line_height),
        ))
    }
}

impl EntityInputHandler for Root {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.composer_range_from_utf16(&range_utf16);
        if range.start > self.composer.len() || range.end > self.composer.len() {
            return None;
        }
        adjusted_range.replace(self.composer_range_to_utf16(&range));
        Some(self.composer[range].to_owned())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.composer_range_to_utf16(&self.composer_selected_range),
            reversed: self.composer_selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.composer_marked_range
            .as_ref()
            .map(|range| self.composer_range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.composer_unmark_text();
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.composer_replace_text_in_range(range_utf16, new_text);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.composer_replace_and_mark_text_in_range(
            range_utf16,
            new_text,
            new_selected_range_utf16,
        );
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _element_bounds: Bounds<Pixels>,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        self.composer_bounds_for_range_utf16(range_utf16, window)
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let byte = self.offset_from_point(point, window);
        Some(Self::composer_utf16_offset_from_byte(&self.composer, byte))
    }

    fn accepts_text_input(&self, _window: &mut Window, _cx: &mut Context<Self>) -> bool {
        // Chat composer is the primary text input in the main view.
        // Login/device/settings challenge inputs are handled via key events in this app today.
        self.page == ui::shell::AppPage::Main && !self.show_login && !self.show_devices_panel
    }
}

impl Render for Root {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _ = self.backend.drain_events();
        self.maybe_auto_connect();
        let snapshot = self.backend.snapshot();

        let chat_ready = self.is_chat_ready(&snapshot.status);

        if !chat_ready {
            if snapshot.login_flow.stage == "mfa" {
                if self.login.step != LoginStep::Mfa {
                    self.login.step = LoginStep::Mfa;
                    self.login.focus = Some(LoginField::MfaCode);
                    self.login.mfa_code.clear();
                    self.login.local_error = None;
                }
            } else if snapshot.login_flow.stage == "busy" {
            } else if snapshot.login_flow.error.is_some() {
                self.login.local_error = snapshot.login_flow.error.clone();
                if self.login.step == LoginStep::Mfa {
                    self.login.step = LoginStep::Credentials;
                    self.login.focus = Some(LoginField::Email);
                }
            } else if snapshot.login_flow.stage.is_empty()
                && snapshot.login_flow.ticket.is_none()
                && self.login.step == LoginStep::Mfa
            {
                self.login.step = LoginStep::Credentials;
                self.login.focus = Some(LoginField::Email);
                self.login.mfa_code.clear();
            }

            if matches!(self.login.step, LoginStep::Forgot | LoginStep::ForgotSent) {
                match snapshot.forgot_password.stage.as_str() {
                    "sent" | "phone_required" => {
                        self.login.step = LoginStep::ForgotSent;
                    }
                    "failed" => {
                        self.login.step = LoginStep::Forgot;
                    }
                    "busy" | "" => {}
                    _ => {}
                }
            }
        }

        if chat_ready {
            self.login.password.clear();
            self.login.mfa_code.clear();
            if !self.login_explicitly_opened {
                self.show_login = false;
            }
        } else if snapshot.status == "Idle"
            || snapshot.status == "Logged out"
            || snapshot.status.contains("Authentication failed")
            || snapshot.status == "Error"
        {
            self.show_login = true;
            self.login_explicitly_opened = false;
        }

        if self.show_login || !chat_ready {
            let theme = self.theme;
            div()
                .id("app-root-login")
                .track_focus(&self.focus_handle(cx))
                .on_key_down(cx.listener(Self::on_key_down))
                .size_full()
                .child(render_login(self, cx, &theme, &snapshot))
                .into_any_element()
        } else {
            if self.show_debug_stats {
                ui::debug_stats::maybe_log_periodic();
            }
            self.render_main(window, cx, &snapshot).into_any_element()
        }
    }
}

fn main() {
    let backend = backend::start(token_store_path());
    let token = initial_token().or_else(|| {
        backend
            .snapshot()
            .saved_accounts
            .first()
            .map(|a| a.token.clone())
    });

    application().run(move |cx: &mut App| {
        if let Ok(http_client) = ui::http_client::ReqwestHttpClient::new() {
            cx.set_http_client(http_client);
        }
        let bounds = Bounds::centered(None, size(px(1200.), px(760.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| {
                    let focus_handle = cx.focus_handle();
                    cx.spawn(async move |this, cx| {
                        loop {
                            cx.background_executor()
                                .timer(Duration::from_millis(250))
                                .await;
                            let Some(entity) = this.upgrade() else {
                                break;
                            };
                            cx.update_entity(&entity, |this: &mut Root, cx| {
                                if this.backend.drain_events() {
                                    cx.notify();
                                }
                            });
                        }
                    })
                    .detach();

                    Root {
                        backend,
                        token,
                        requested_channel: None,
                        composer: String::new(),
                        composer_selected_range: 0..0,
                        composer_selection_reversed: false,
                        composer_marked_range: None,
                        composer_last_bounds: None,
                        composer_is_selecting: false,
                        composer_reply: None,
                        composer_edit: None,
                        pending_attachments: Vec::new(),
                        focus_handle,
                        login: LoginUiState {
                            step: LoginStep::Credentials,
                            focus: Some(LoginField::Email),
                            ..LoginUiState::default()
                        },
                        theme: theme::load_shell_synced_theme(),
                        show_login: true,
                        login_explicitly_opened: false,
                        show_devices_panel: false,
                        show_member_panel: true,
                        show_debug_stats: false,
                        device_challenge_input: String::new(),
                        page: ui::shell::AppPage::Main,
                        settings_page: ui::shell::SettingsPage::Account,
                        expanded_guild_folders: HashSet::new(),
                        collapsed_categories: HashSet::new(),
                        message_list_state: gpui::ListState::new(
                            0,
                            gpui::ListAlignment::Top,
                            gpui::px(400.),
                        ),
                        message_list_channel_id: None,
                        message_list_row_count: 0,
                        message_list_first_id: None,
                        message_list_last_id: None,
                        message_list_last_read: None,
                    }
                });
                window.focus(&view.focus_handle(cx), cx);
                view
            },
        )
        .expect("failed to open Lampcord window");
        cx.activate(true);
    });
}
