use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::llamacpp::slot::Slot;

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusUpdate {
    pub agent_name: Option<String>,
    pub error: Option<String>,
    pub external_llamacpp_addr: SocketAddr,
    pub idle_slots_count: usize,
    pub is_authorized: Option<bool>,
    pub is_slots_endpoint_enabled: Option<bool>,
    pub processing_slots_count: usize,
    slots: Vec<Slot>,
}

impl StatusUpdate {
    pub fn new(
        agent_name: Option<String>,
        error: Option<String>,
        external_llamacpp_addr: SocketAddr,
        is_authorized: Option<bool>,
        is_slots_endpoint_enabled: Option<bool>,
        slots: Option<Vec<Slot>>,
    ) -> Self {
        let (slots, idle_slots_count) = match slots {
            Some(slots) => (
                slots.clone(),
                slots.iter().filter(|slot| !slot.is_processing).count(),
            ),
            None => (vec![], 0),
        };

        Self {
            agent_name,
            error,
            external_llamacpp_addr,
            idle_slots_count,
            is_authorized,
            is_slots_endpoint_enabled,
            processing_slots_count: slots.len() - idle_slots_count,
            slots,
        }
    }
}

impl actix::Message for StatusUpdate {
    type Result = ();
}
