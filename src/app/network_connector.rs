// src/app/network_connector.rs
//! Handles establishing the WebSocket connection.

use std::sync::{Arc, Mutex};
use crate::network::NetworkManager;
use crate::log;

pub fn connect(network_manager_arc: &Arc<Mutex<NetworkManager>>) {
    log("App::Network: connect() called.");
    match network_manager_arc.lock() {
        Ok(mut nm) => nm.connect(),
        Err(e) => log(&format!("App::Network: Failed to lock NetworkManager for connect: {:?}", e)),
    }
} 