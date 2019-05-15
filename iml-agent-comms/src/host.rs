// Copyright (c) 2019 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::session::SharedSessions;
use iml_wire_types::Fqdn;
use parking_lot::Mutex;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

/// References an active agent on a remote host.
///
/// Contains references to any active sessions on the remote host,
/// and maintains a `VecDeque` of outgoing messages to send to the remote host.
#[derive(Debug)]
pub struct Host {
    pub fqdn: Fqdn,
    pub client_start_time: String,
    pub queue: Arc<Mutex<VecDeque<Vec<u8>>>>,
    pub sessions: SharedSessions,
}

impl Host {
    pub fn new(fqdn: Fqdn, client_start_time: String) -> Self {
        Self {
            fqdn,
            client_start_time,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

pub type Hosts = HashMap<Fqdn, Host>;
pub type SharedHosts = Arc<Mutex<Hosts>>;

pub fn shared_hosts() -> SharedHosts {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Does this host entry have a different `start_time` than the remote host?
pub fn is_stale(hosts: &Hosts, fqdn: &Fqdn, client_start_time: &str) -> bool {
    match hosts.get(fqdn) {
        Some(h) if h.client_start_time != client_start_time => true,
        _ => false,
    }
}

/// Gets or inserts a new host cooresponding to the given fqdn
pub fn get_or_insert(hosts: &mut Hosts, fqdn: Fqdn, client_start_time: String) -> &Host {
    hosts.entry(fqdn.clone()).or_insert_with(|| {
        log::info!("Adding new host {}", fqdn);

        Host::new(fqdn, client_start_time)
    })
}