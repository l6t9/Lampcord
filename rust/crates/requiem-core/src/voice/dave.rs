use std::collections::HashMap;
use std::num::NonZeroU16;

use davey::{DaveSession, MediaType};

use super::opcodes;

#[derive(Debug)]
pub enum DaveAction {
    None,

    KeyPackage(Vec<u8>),

    CommitWelcome(Vec<u8>),

    TransitionReady(u16),

    InvalidCommitWelcome(u16),
}

pub struct Dave {
    protocol_version: u16,
    session: Option<DaveSession>,

    pending: HashMap<u16, u16>,

    downgraded: bool,
}

impl Dave {
    pub fn new() -> Self {
        Self {
            protocol_version: 0,
            session: None,
            pending: HashMap::new(),
            downgraded: false,
        }
    }

    pub fn protocol_version(&self) -> u16 {
        self.protocol_version
    }

    pub fn ready(&self) -> bool {
        self.protocol_version != 0 && self.session.as_ref().is_some_and(DaveSession::is_ready)
    }

    pub fn can_passthrough(&self, user_id: u64) -> bool {
        self.session
            .as_ref()
            .is_some_and(|s| s.is_ready() && s.can_passthrough(user_id))
    }

    pub fn on_session_description(
        &mut self,
        protocol_version: u16,
        user_id: u64,
        channel_id: u64,
    ) -> Option<Vec<u8>> {
        tracing::info!(
            protocol_version,
            user_id,
            channel_id,
            "dave session description"
        );
        self.protocol_version = protocol_version;
        self.pending.clear();
        self.downgraded = false;

        if protocol_version == 0 {
            tracing::info!("dave disabled for this call; media stays transport-encrypted only");
            if let Some(session) = &mut self.session {
                let _ = session.reset();
                session.set_passthrough_mode(true, Some(10));
            }
            return None;
        }

        let version = NonZeroU16::new(protocol_version)?;

        if let Some(session) = &mut self.session {
            if let Err(err) = session.reinit(version, user_id, channel_id, None) {
                tracing::error!(%err, "dave reinit failed");
                return None;
            }
        } else {
            match DaveSession::new(version, user_id, channel_id, None) {
                Ok(session) => self.session = Some(session),
                Err(err) => {
                    tracing::error!(%err, "dave init failed");
                    return None;
                }
            }
        }

        match self
            .session
            .as_mut()
            .and_then(|s| s.create_key_package().ok())
        {
            Some(package) => {
                tracing::debug!(bytes = package.len(), "dave key package created");
                Some(package)
            }
            None => {
                tracing::error!("dave key package creation failed");
                None
            }
        }
    }

    pub fn set_external_sender(&mut self, data: &[u8]) {
        tracing::debug!(bytes = data.len(), "dave external sender");
        let Some(session) = &mut self.session else {
            tracing::warn!("dave external sender arrived before the session existed");
            return;
        };
        if let Err(err) = session.set_external_sender(data) {
            tracing::error!(%err, "dave set_external_sender failed");
        }
    }

    pub fn on_proposals(
        &mut self,
        payload: &[u8],
        recognized_user_ids: &[u64],
    ) -> Option<DaveAction> {
        let op_type = davey_op_type(*payload.first()?)?;
        tracing::debug!(
            ?op_type,
            bytes = payload.len(),
            users = recognized_user_ids.len(),
            "dave proposals"
        );
        let proposals = &payload[1..];
        let commit = match self.session.as_mut()?.process_proposals(
            op_type,
            proposals,
            Some(recognized_user_ids),
        ) {
            Ok(commit) => commit,
            Err(err) => {
                tracing::error!(%err, "dave process_proposals failed");
                return None;
            }
        }?;
        let mut bytes = commit.commit;
        if let Some(welcome) = commit.welcome {
            bytes.extend_from_slice(&welcome);
        }
        tracing::debug!(bytes = bytes.len(), "dave commit welcome built");
        Some(DaveAction::CommitWelcome(bytes))
    }

    pub fn on_commit(
        &mut self,
        commit: &[u8],
        transition_id: u16,
    ) -> Result<Option<DaveAction>, ()> {
        tracing::debug!(transition_id, bytes = commit.len(), "dave commit");
        if let Err(err) = self.session.as_mut().ok_or(())?.process_commit(commit) {
            tracing::error!(%err, transition_id, "dave process_commit failed");
            return Err(());
        }
        self.pending
            .entry(transition_id)
            .or_insert(self.protocol_version);
        Ok((transition_id != 0).then_some(DaveAction::TransitionReady(transition_id)))
    }

    pub fn on_welcome(
        &mut self,
        welcome: &[u8],
        transition_id: u16,
    ) -> Result<Option<DaveAction>, ()> {
        tracing::debug!(transition_id, bytes = welcome.len(), "dave welcome");
        if let Err(err) = self.session.as_mut().ok_or(())?.process_welcome(welcome) {
            tracing::error!(%err, transition_id, "dave process_welcome failed");
            return Err(());
        }
        self.pending
            .entry(transition_id)
            .or_insert(self.protocol_version);
        Ok((transition_id != 0).then_some(DaveAction::TransitionReady(transition_id)))
    }

    pub fn on_prepare_transition(
        &mut self,
        transition_id: u16,
        protocol_version: u16,
    ) -> DaveAction {
        self.pending.insert(transition_id, protocol_version);

        if transition_id == 0 {
            self.execute_transition(transition_id);
            return DaveAction::None;
        }

        if protocol_version == 0 {
            if let Some(session) = &mut self.session {
                session.set_passthrough_mode(true, Some(120));
            }
        }
        DaveAction::TransitionReady(transition_id)
    }

    pub fn on_execute_transition(&mut self, transition_id: u16) {
        self.execute_transition(transition_id);
    }

    pub fn on_prepare_epoch(
        &mut self,
        epoch: u32,
        protocol_version: u16,
        user_id: u64,
        channel_id: u64,
    ) -> Option<Vec<u8>> {
        if epoch != 1 {
            return None;
        }
        self.on_session_description(protocol_version, user_id, channel_id)
    }

    pub fn recover(&mut self, user_id: u64, channel_id: u64) -> DaveAction {
        let version = self.protocol_version;
        self.on_session_description(version, user_id, channel_id);
        DaveAction::None
    }

    fn execute_transition(&mut self, transition_id: u16) {
        let Some(target) = self.pending.remove(&transition_id) else {
            return;
        };
        let old = self.protocol_version;
        self.protocol_version = target;

        if old != target && target == 0 {
            self.downgraded = true;
        } else if transition_id > 0 && self.downgraded {
            self.downgraded = false;
            if let Some(session) = &mut self.session {
                session.set_passthrough_mode(true, Some(10));
            }
        }
    }

    pub fn encrypt_opus(&mut self, frame: &[u8]) -> Option<Vec<u8>> {
        if !self.ready() {
            tracing::trace!(
                bytes = frame.len(),
                "dave encrypt skipped; session not ready"
            );
            return None;
        }
        match self.session.as_mut()?.encrypt_opus(frame) {
            Ok(frame) => Some(frame.into_owned()),
            Err(err) => {
                tracing::warn!(%err, "dave opus encrypt failed; sending transport-only frame");
                None
            }
        }
    }

    pub fn encrypt_video(&mut self, frame: &[u8]) -> Option<Vec<u8>> {
        if !self.ready() {
            return None;
        }
        match self
            .session
            .as_mut()?
            .encrypt(MediaType::VIDEO, davey::Codec::VP8, frame)
        {
            Ok(frame) => Some(frame.into_owned()),
            Err(err) => {
                tracing::warn!(%err, "dave video encrypt failed; sending transport-only frame");
                None
            }
        }
    }

    pub fn decrypt(&mut self, user_id: u64, frame: &[u8]) -> Option<Vec<u8>> {
        let session = self.session.as_mut()?;
        if !session.is_ready() && !session.can_passthrough(user_id) {
            tracing::trace!(
                user_id,
                bytes = frame.len(),
                "dave decrypt skipped; session not ready"
            );
            return None;
        }
        match session.decrypt(user_id, MediaType::AUDIO, frame) {
            Ok(frame) => Some(frame),
            Err(err) => {
                tracing::debug!(%err, user_id, bytes = frame.len(), "dave decrypt failed");
                None
            }
        }
    }
}

impl Default for Dave {
    fn default() -> Self {
        Self::new()
    }
}

fn davey_op_type(byte: u8) -> Option<davey::ProposalsOperationType> {
    match opcodes::ProposalsOperationType::from_byte(byte)? {
        opcodes::ProposalsOperationType::Append => Some(davey::ProposalsOperationType::APPEND),
        opcodes::ProposalsOperationType::Revoke => Some(davey::ProposalsOperationType::REVOKE),
    }
}

pub use davey::DAVE_PROTOCOL_VERSION as MAX_DAVE_PROTOCOL_VERSION;
