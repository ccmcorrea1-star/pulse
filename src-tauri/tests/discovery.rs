use std::collections::HashMap;
use std::time::{Duration, Instant};

use pulse_lib::discovery::{
    CandidateChange, CandidateRegistry, DiscoveryAddress, DiscoveryAnnouncement, DiscoveryError,
    SERVICE_TYPE,
};
use pulse_lib::domain::{DiscoveryCandidateState, UtcTimestamp};

fn announcement() -> DiscoveryAnnouncement {
    let mut properties = HashMap::new();
    properties.insert("proto".to_owned(), "1".to_owned());
    properties.insert("model".to_owned(), "1".to_owned());
    properties.insert("transport".to_owned(), "quic".to_owned());
    properties.insert("platform".to_owned(), "linux".to_owned());
    properties.insert(
        "caps".to_owned(),
        "files.send,text.send,files.send".to_owned(),
    );
    DiscoveryAnnouncement {
        service_type: SERVICE_TYPE.to_owned(),
        fullname: "devbox._pulse._udp.local.".to_owned(),
        port: 42000,
        addresses: vec![DiscoveryAddress::new("192.168.2.20")],
        properties,
    }
}

fn timestamp(value: &str) -> UtcTimestamp {
    UtcTimestamp(value.to_owned())
}

#[test]
fn valid_peer_is_added_and_duplicate_updates_the_same_candidate() {
    let mut registry = CandidateRegistry::new(Duration::from_secs(120));
    let now = Instant::now();
    let added = registry
        .upsert(
            announcement(),
            timestamp("2026-08-15T12:00:00.000Z"),
            now,
            timestamp("2026-08-15T12:02:00.000Z"),
        )
        .unwrap();
    let first_id = match added {
        CandidateChange::Added(candidate) => candidate.id,
        other => panic!("expected added candidate, got {other:?}"),
    };

    let mut updated_announcement = announcement();
    updated_announcement.port = 43000;
    let updated = registry
        .upsert(
            updated_announcement,
            timestamp("2026-08-15T12:00:30.000Z"),
            now + Duration::from_secs(30),
            timestamp("2026-08-15T12:02:30.000Z"),
        )
        .unwrap();

    match updated {
        CandidateChange::Updated(candidate) => {
            assert_eq!(candidate.id, first_id);
            assert_eq!(candidate.endpoint.value, "udp://192.168.2.20:43000");
        }
        other => panic!("expected updated candidate, got {other:?}"),
    }
    assert_eq!(registry.len(), 1);
    assert_eq!(registry.active_candidates().len(), 1);
}

#[test]
fn all_resolved_endpoints_are_kept_while_domain_exposes_a_primary_one() {
    let mut peer = announcement();
    peer.addresses = vec![
        DiscoveryAddress::new("192.168.2.20"),
        DiscoveryAddress::new("[fe80::12%enp42s0]"),
    ];
    let mut registry = CandidateRegistry::new(Duration::from_secs(120));
    registry
        .upsert(
            peer,
            timestamp("2026-08-15T12:00:00.000Z"),
            Instant::now(),
            timestamp("2026-08-15T12:02:00.000Z"),
        )
        .unwrap();

    assert_eq!(
        registry
            .endpoints_for("devbox._pulse._udp.local.")
            .unwrap()
            .iter()
            .map(|endpoint| endpoint.value.as_str())
            .collect::<Vec<_>>(),
        vec!["udp://192.168.2.20:42000", "udp://[fe80::12%enp42s0]:42000",]
    );
}

#[test]
fn invalid_and_out_of_scope_announcements_are_rejected() {
    let mut registry = CandidateRegistry::new(Duration::from_secs(120));
    let now = Instant::now();

    let mut invalid_type = announcement();
    invalid_type.service_type = "_other._udp.local.".to_owned();
    assert_eq!(
        registry
            .upsert(
                invalid_type,
                timestamp("2026-08-15T12:00:00.000Z"),
                now,
                timestamp("2026-08-15T12:02:00.000Z"),
            )
            .unwrap_err(),
        DiscoveryError::InvalidServiceType
    );

    let mut invalid_capability = announcement();
    invalid_capability
        .properties
        .insert("caps".to_owned(), "camera.read".to_owned());
    assert_eq!(
        registry
            .upsert(
                invalid_capability,
                timestamp("2026-08-15T12:00:00.000Z"),
                now,
                timestamp("2026-08-15T12:02:00.000Z"),
            )
            .unwrap_err(),
        DiscoveryError::UnsupportedCapability
    );

    let mut no_endpoint = announcement();
    no_endpoint.addresses.clear();
    assert_eq!(
        registry
            .upsert(
                no_endpoint,
                timestamp("2026-08-15T12:00:00.000Z"),
                now,
                timestamp("2026-08-15T12:02:00.000Z"),
            )
            .unwrap_err(),
        DiscoveryError::NoUsableEndpoint
    );
    assert!(registry.is_empty());
}

#[test]
fn removal_and_ttl_expire_candidates_without_creating_trust() {
    let mut registry = CandidateRegistry::new(Duration::from_secs(10));
    let now = Instant::now();
    registry
        .upsert(
            announcement(),
            timestamp("2026-08-15T12:00:00.000Z"),
            now,
            timestamp("2026-08-15T12:00:10.000Z"),
        )
        .unwrap();

    let expired = registry.expire(
        now + Duration::from_secs(10),
        timestamp("2026-08-15T12:00:10.000Z"),
    );
    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0].state, DiscoveryCandidateState::Expired);
    assert!(registry.active_candidates().is_empty());

    let reappeared = registry
        .upsert(
            announcement(),
            timestamp("2026-08-15T12:00:20.000Z"),
            now + Duration::from_secs(20),
            timestamp("2026-08-15T12:00:30.000Z"),
        )
        .unwrap();
    match reappeared {
        CandidateChange::Added(candidate) => {
            assert_eq!(candidate.state, DiscoveryCandidateState::Discovered);
            assert_ne!(candidate.id, expired[0].id);
        }
        other => panic!("expected a new candidate generation, got {other:?}"),
    }

    let removed = registry.remove(
        SERVICE_TYPE,
        "devbox._pulse._udp.local.",
        timestamp("2026-08-15T12:00:21.000Z"),
    );
    assert!(matches!(removed, Some(CandidateChange::Expired(_))));
}

#[test]
fn ipv6_endpoint_keeps_interface_scope() {
    let mut peer = announcement();
    peer.addresses = vec![DiscoveryAddress::new("[fe80::12%enp42s0]")];
    let mut registry = CandidateRegistry::new(Duration::from_secs(120));
    let change = registry
        .upsert(
            peer,
            timestamp("2026-08-15T12:00:00.000Z"),
            Instant::now(),
            timestamp("2026-08-15T12:02:00.000Z"),
        )
        .unwrap();

    let candidate = match change {
        CandidateChange::Added(candidate) => candidate,
        other => panic!("expected added candidate, got {other:?}"),
    };
    assert_eq!(candidate.endpoint.value, "udp://[fe80::12%enp42s0]:42000");
}
