use pulse_lib::domain::{
    CapabilityGrantState, DiscoveryCandidateState, NotificationState, PairingState, PresenceState,
    RemoteCommandState, TransferState, TrustState,
};

#[test]
fn presence_transitions_keep_offline_distinct_from_trust() {
    assert!(PresenceState::Online.can_transition_to(PresenceState::Stale));
    assert!(PresenceState::Stale.can_transition_to(PresenceState::Offline));
    assert!(!PresenceState::Offline.can_transition_to(PresenceState::Stale));
    assert!(TrustState::Trusted.can_transition_to(TrustState::Revoked));
}

#[test]
fn terminal_pairing_and_transfer_states_do_not_reopen() {
    assert!(PairingState::AwaitingConfirmation.can_transition_to(PairingState::Expired));
    assert!(!PairingState::Confirmed.can_transition_to(PairingState::AwaitingConfirmation));
    assert!(TransferState::Failed.can_transition_to(TransferState::Queued));
    assert!(!TransferState::Completed.can_transition_to(TransferState::Active));
}

#[test]
fn capability_and_notification_cycles_are_explicit() {
    assert!(CapabilityGrantState::Denied.can_transition_to(CapabilityGrantState::Requested));
    assert!(CapabilityGrantState::Granted.can_transition_to(CapabilityGrantState::Revoked));
    assert!(!CapabilityGrantState::Granted.can_transition_to(CapabilityGrantState::Requested));
    assert!(NotificationState::Delivered.can_transition_to(NotificationState::Dismissed));
    assert!(!NotificationState::Dismissed.can_transition_to(NotificationState::Queued));
}

#[test]
fn discovery_and_remote_command_terminals_are_closed() {
    assert!(DiscoveryCandidateState::Discovered.can_transition_to(DiscoveryCandidateState::Expired));
    assert!(
        !DiscoveryCandidateState::Expired.can_transition_to(DiscoveryCandidateState::Discovered)
    );
    assert!(RemoteCommandState::Running.can_transition_to(RemoteCommandState::Succeeded));
    assert!(!RemoteCommandState::Succeeded.can_transition_to(RemoteCommandState::Running));
}
