use std::{collections::HashMap, sync::Arc};

use pallet_cf_vaults::CeremonyId;
use tokio::sync::mpsc;

use crate::{
    logging::CEREMONY_ID_KEY,
    p2p::AccountId,
    signing::{KeygenInfo, KeygenOutcome},
};

use super::{
    client_inner::KeyGenMessageWrapped,
    keygen_state::KeygenState,
    utils::{get_index_mapping, project_signers},
    InnerEvent, KeygenResultInfo,
};

#[derive(Clone)]
pub struct KeygenManager {
    /// States for each ceremony_id
    keygen_states: HashMap<CeremonyId, KeygenState>,
    /// Used to propagate events upstream
    event_sender: mpsc::UnboundedSender<InnerEvent>,
    /// Validator id of our node
    id: AccountId,
    logger: slog::Logger,
}

impl KeygenManager {
    pub fn new(
        id: AccountId,
        event_sender: mpsc::UnboundedSender<InnerEvent>,
        logger: &slog::Logger,
    ) -> Self {
        KeygenManager {
            keygen_states: Default::default(),
            event_sender,
            id,
            logger: logger.clone(),
        }
    }

    pub fn cleanup(&mut self) {
        let mut events_to_send = vec![];

        // Have to clone so it can be used inside the closure
        let logger = &self.logger;
        self.keygen_states.retain(|ceremony_id, state| {
            if let Some(bad_nodes) = state.try_expiring() {
                slog::warn!(logger, "Keygen state expired and will be abandoned");
                let outcome = KeygenOutcome::timeout(*ceremony_id, bad_nodes);

                events_to_send.push(InnerEvent::KeygenResult(outcome));

                false
            } else {
                true
            }
        });

        for event in events_to_send {
            if let Err(err) = self.event_sender.send(event) {
                slog::error!(self.logger, "Unable to send event, error: {}", err);
            }
        }
    }

    pub fn on_keygen_request(&mut self, keygen_info: KeygenInfo) {
        let KeygenInfo {
            ceremony_id,
            signers,
        } = keygen_info;

        let logger = self.logger.new(slog::o!(CEREMONY_ID_KEY => ceremony_id));

        // TODO: check the number of participants?

        if !signers.contains(&self.id) {
            // TODO: alert
            slog::warn!(
                logger,
                "Keygen request ignored: we are not among participants",
            );

            return;
        }

        let validator_map = Arc::new(get_index_mapping(&signers));

        let our_idx = match validator_map.get_idx(&self.id) {
            Some(idx) => idx,
            None => {
                // This should be impossible because of the check above,
                // but I don't like unwrapping (would be better if we
                // could combine this with the check above)
                slog::warn!(logger, "Request to sign ignored: could not derive our idx");
                return;
            }
        };

        // Check that signer ids are known for this key
        let signer_idxs = match project_signers(&signers, &validator_map) {
            Ok(signer_idxs) => signer_idxs,
            Err(_) => {
                // TODO: alert
                slog::warn!(logger, "Request to sign ignored: invalid signers");
                return;
            }
        };

        let logger = self.logger.clone();

        let entry = self
            .keygen_states
            .entry(ceremony_id)
            .or_insert_with(|| KeygenState::new_unauthorised(logger));

        entry.on_keygen_request(
            ceremony_id,
            self.event_sender.clone(),
            validator_map,
            our_idx,
            signer_idxs,
        );
    }

    pub fn process_keygen_data(
        &mut self,
        sender_id: AccountId,
        msg: KeyGenMessageWrapped,
    ) -> Option<KeygenResultInfo> {
        let KeyGenMessageWrapped { ceremony_id, data } = msg;

        // TODO: how can I avoid cloning the logger?
        let logger = self.logger.clone();

        let state = self
            .keygen_states
            .entry(ceremony_id)
            .or_insert_with(|| KeygenState::new_unauthorised(logger));

        let res = state.process_message(sender_id, data);

        // TODO: this is not a complete solution, we need to clean up the state
        // when it is failed too
        if res.is_some() {
            debug_assert!(self.keygen_states.remove(&ceremony_id).is_some());
            slog::debug!(self.logger, "Removed a successfully finished keygen ceremony"; "ceremony_id" => ceremony_id);
        }

        res
    }
}

#[cfg(test)]
impl KeygenManager {
    pub fn expire_all(&mut self) {
        for (_, state) in &mut self.keygen_states {
            state.set_expiry_time(std::time::Instant::now());
        }
    }

    pub fn get_stage_for(&self, ceremony_id: CeremonyId) -> Option<String> {
        self.keygen_states
            .get(&ceremony_id)
            .and_then(|s| s.get_stage())
    }
}
