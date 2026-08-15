//! Discovery local de candidatos via mDNS/DNS-SD.
//!
//! Este módulo conhece a implementação de discovery, mas não concede
//! identidade, presença, pairing, trust ou capabilities. O registro de
//! candidatos é mantido separado do runtime e da bridge para que possa ser
//! testado com anúncios falsos sem depender da rede local.

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use mdns_sd::{ResolvedService, ScopedIp, ServiceDaemon, ServiceEvent};
use sha2::{Digest, Sha256};

use crate::domain::{
    CandidateId, CapabilityDirection, CapabilityInfo, CapabilityKey, DevicePlatform,
    DiscoveryCandidate, DiscoveryCandidateState, DiscoveryEndpoint, UtcTimestamp,
};
use crate::runtime::{RuntimeService, ServiceFailureCode, ServiceKind};

pub const SERVICE_TYPE: &str = "_pulse._udp.local.";
pub const PROTOCOL_VERSION: &str = "1";
pub const MODEL_VERSION: &str = "1";
pub const CANDIDATE_TTL: Duration = Duration::from_secs(120);
const DISCOVERY_POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiscoveryError {
    InvalidServiceType,
    InvalidFullname,
    MissingProperty(&'static str),
    InvalidProperty(&'static str),
    UnsupportedProperty(&'static str),
    UnsupportedCapability,
    NoUsableEndpoint,
}

impl Display for DiscoveryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let code = match self {
            Self::InvalidServiceType => "invalid service type",
            Self::InvalidFullname => "invalid service fullname",
            Self::MissingProperty(_) => "required discovery property is missing",
            Self::InvalidProperty(_) => "discovery property is invalid",
            Self::UnsupportedProperty(_) => "discovery property is unsupported",
            Self::UnsupportedCapability => "discovery capability is unsupported",
            Self::NoUsableEndpoint => "discovery announcement has no usable endpoint",
        };
        formatter.write_str(code)
    }
}

impl std::error::Error for DiscoveryError {}

/// Endereço resolvido de um anúncio. O valor IPv6 inclui o escopo da
/// interface quando o resolver o fornece, por exemplo `%enp42s0`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DiscoveryAddress {
    pub value: String,
}

impl DiscoveryAddress {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    fn from_scoped_ip(address: &ScopedIp) -> Option<Self> {
        let ip = address.to_ip_addr();
        if ip.is_loopback() || ip.is_unspecified() || ip.is_multicast() {
            return None;
        }

        let rendered = address.to_string();
        let value = if address.is_ipv6() {
            format!("[{rendered}]")
        } else {
            rendered
        };
        Some(Self { value })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryAnnouncement {
    pub service_type: String,
    pub fullname: String,
    pub port: u16,
    pub addresses: Vec<DiscoveryAddress>,
    pub properties: HashMap<String, String>,
}

impl TryFrom<&ResolvedService> for DiscoveryAnnouncement {
    type Error = DiscoveryError;

    fn try_from(service: &ResolvedService) -> Result<Self, Self::Error> {
        let addresses = service
            .get_addresses()
            .iter()
            .filter_map(DiscoveryAddress::from_scoped_ip)
            .collect();
        let properties = service
            .get_properties()
            .iter()
            .map(|property| {
                (
                    property.key().to_ascii_lowercase(),
                    property.val_str().to_owned(),
                )
            })
            .collect();

        Ok(Self {
            service_type: service.ty_domain.clone(),
            fullname: service.fullname.clone(),
            port: service.port,
            addresses,
            properties,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CandidateChange {
    Added(DiscoveryCandidate),
    Updated(DiscoveryCandidate),
    Expired(DiscoveryCandidate),
}

#[derive(Clone, Debug)]
struct RegisteredCandidate {
    generation: u64,
    deadline: Instant,
    endpoints: Vec<DiscoveryEndpoint>,
    candidate: DiscoveryCandidate,
}

#[derive(Clone, Debug)]
pub struct CandidateRegistry {
    ttl: Duration,
    candidates: HashMap<String, RegisteredCandidate>,
}

impl CandidateRegistry {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            candidates: HashMap::new(),
        }
    }

    pub fn ttl(&self) -> Duration {
        self.ttl
    }

    pub fn len(&self) -> usize {
        self.candidates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    pub fn candidates(&self) -> Vec<DiscoveryCandidate> {
        let mut candidates: Vec<_> = self
            .candidates
            .values()
            .map(|registered| registered.candidate.clone())
            .collect();
        candidates.sort_by(|left, right| left.id.0.cmp(&right.id.0));
        candidates
    }

    pub fn active_candidates(&self) -> Vec<DiscoveryCandidate> {
        self.candidates
            .values()
            .filter(|registered| registered.candidate.state == DiscoveryCandidateState::Discovered)
            .map(|registered| registered.candidate.clone())
            .collect()
    }

    pub fn endpoints_for(&self, fullname: &str) -> Option<Vec<DiscoveryEndpoint>> {
        self.candidates
            .get(&normalize_name(fullname))
            .map(|registered| registered.endpoints.clone())
    }

    pub fn upsert(
        &mut self,
        announcement: DiscoveryAnnouncement,
        observed_at: UtcTimestamp,
        now: Instant,
        expires_at: UtcTimestamp,
    ) -> Result<CandidateChange, DiscoveryError> {
        let parsed = parse_announcement(&announcement, observed_at.clone())?;
        let service_key = normalize_name(&announcement.fullname);
        let existing = self.candidates.remove(&service_key);
        let (generation, candidate_id, discovered_at, change_kind) = match existing {
            Some(previous) if previous.candidate.state == DiscoveryCandidateState::Discovered => (
                previous.generation,
                previous.candidate.id,
                previous.candidate.discovered_at,
                CandidateChangeKind::Updated,
            ),
            Some(previous) => {
                let generation = previous.generation.saturating_add(1);
                (
                    generation,
                    candidate_id(&service_key, generation),
                    parsed.discovered_at.clone(),
                    CandidateChangeKind::Added,
                )
            }
            None => (
                0,
                candidate_id(&service_key, 0),
                parsed.discovered_at.clone(),
                CandidateChangeKind::Added,
            ),
        };

        let candidate = DiscoveryCandidate {
            id: candidate_id,
            presented_name: parsed.presented_name,
            platform: parsed.platform,
            endpoint: parsed.endpoint,
            advertised_capabilities: parsed.advertised_capabilities,
            state: DiscoveryCandidateState::Discovered,
            discovered_at,
            last_seen_at: observed_at.clone(),
            expires_at,
            updated_at: observed_at,
        };
        self.candidates.insert(
            service_key.clone(),
            RegisteredCandidate {
                generation,
                deadline: now + self.ttl,
                endpoints: parsed.endpoints,
                candidate: candidate.clone(),
            },
        );

        Ok(match change_kind {
            CandidateChangeKind::Added => CandidateChange::Added(candidate),
            CandidateChangeKind::Updated => CandidateChange::Updated(candidate),
        })
    }

    pub fn remove(
        &mut self,
        service_type: &str,
        fullname: &str,
        observed_at: UtcTimestamp,
    ) -> Option<CandidateChange> {
        if !is_service_type(service_type) {
            return None;
        }

        let key = normalize_name(fullname);
        let registered = self.candidates.get_mut(&key)?;
        if registered.candidate.state == DiscoveryCandidateState::Expired {
            return None;
        }

        registered.candidate.state = DiscoveryCandidateState::Expired;
        registered.candidate.updated_at = observed_at;
        Some(CandidateChange::Expired(registered.candidate.clone()))
    }

    pub fn expire(&mut self, now: Instant, observed_at: UtcTimestamp) -> Vec<DiscoveryCandidate> {
        let expired_keys: Vec<_> = self
            .candidates
            .iter()
            .filter(|(_, registered)| {
                registered.candidate.state == DiscoveryCandidateState::Discovered
                    && registered.deadline <= now
            })
            .map(|(key, _)| key.clone())
            .collect();

        expired_keys
            .into_iter()
            .filter_map(
                |key| match self.remove(SERVICE_TYPE, &key, observed_at.clone()) {
                    Some(CandidateChange::Expired(candidate)) => Some(candidate),
                    _ => None,
                },
            )
            .collect()
    }
}

#[derive(Clone, Copy)]
enum CandidateChangeKind {
    Added,
    Updated,
}

#[derive(Clone, Debug)]
struct ParsedAnnouncement {
    presented_name: String,
    platform: DevicePlatform,
    endpoint: DiscoveryEndpoint,
    endpoints: Vec<DiscoveryEndpoint>,
    advertised_capabilities: Vec<CapabilityInfo>,
    discovered_at: UtcTimestamp,
}

fn parse_announcement(
    announcement: &DiscoveryAnnouncement,
    observed_at: UtcTimestamp,
) -> Result<ParsedAnnouncement, DiscoveryError> {
    if !is_service_type(&announcement.service_type) {
        return Err(DiscoveryError::InvalidServiceType);
    }
    if announcement.port == 0 {
        return Err(DiscoveryError::InvalidProperty("port"));
    }
    let presented_name = instance_name(&announcement.fullname)?;
    let platform = parse_platform(required_property(announcement, "platform")?)?;
    validate_version(announcement, "proto", PROTOCOL_VERSION)?;
    validate_version(announcement, "model", MODEL_VERSION)?;
    validate_version(announcement, "transport", "quic")?;

    let mut addresses = announcement.addresses.clone();
    addresses.sort_by(|left, right| left.value.cmp(&right.value));
    let endpoints: Vec<_> = addresses
        .iter()
        .map(|address| DiscoveryEndpoint {
            value: format!("udp://{}:{}", address.value, announcement.port),
        })
        .collect();
    let endpoint = endpoints
        .first()
        .cloned()
        .ok_or(DiscoveryError::NoUsableEndpoint)?;
    let advertised_capabilities = parse_capabilities(announcement, observed_at.clone())?;

    Ok(ParsedAnnouncement {
        presented_name,
        platform,
        endpoint,
        endpoints,
        advertised_capabilities,
        discovered_at: observed_at,
    })
}

fn parse_capabilities(
    announcement: &DiscoveryAnnouncement,
    observed_at: UtcTimestamp,
) -> Result<Vec<CapabilityInfo>, DiscoveryError> {
    let Some(value) = announcement.properties.get("caps") else {
        return Ok(Vec::new());
    };

    let mut capabilities = Vec::new();
    for raw_key in value
        .split(',')
        .map(str::trim)
        .filter(|key| !key.is_empty())
    {
        let (key, direction) = capability(raw_key).ok_or(DiscoveryError::UnsupportedCapability)?;
        capabilities.push(CapabilityInfo {
            key,
            available: true,
            direction: Some(direction),
            observed_at: observed_at.clone(),
        });
    }
    capabilities.sort_by_key(|capability| capability_name(capability.key));
    capabilities.dedup_by_key(|capability| capability_name(capability.key));
    Ok(capabilities)
}

fn capability(value: &str) -> Option<(CapabilityKey, CapabilityDirection)> {
    Some(match value {
        "files.send" => (CapabilityKey::FilesSend, CapabilityDirection::Send),
        "files.receive" => (CapabilityKey::FilesReceive, CapabilityDirection::Receive),
        "clipboard.read" => (CapabilityKey::ClipboardRead, CapabilityDirection::Read),
        "clipboard.write" => (CapabilityKey::ClipboardWrite, CapabilityDirection::Write),
        "text.send" => (CapabilityKey::TextSend, CapabilityDirection::Send),
        "links.send" => (CapabilityKey::LinksSend, CapabilityDirection::Send),
        "media.read" => (CapabilityKey::MediaRead, CapabilityDirection::Read),
        "media.control" => (CapabilityKey::MediaControl, CapabilityDirection::Control),
        "notifications.receive" => (
            CapabilityKey::NotificationsReceive,
            CapabilityDirection::Receive,
        ),
        "commands.execute" => (CapabilityKey::CommandsExecute, CapabilityDirection::Execute),
        _ => return None,
    })
}

fn capability_name(key: CapabilityKey) -> &'static str {
    match key {
        CapabilityKey::FilesSend => "files.send",
        CapabilityKey::FilesReceive => "files.receive",
        CapabilityKey::ClipboardRead => "clipboard.read",
        CapabilityKey::ClipboardWrite => "clipboard.write",
        CapabilityKey::TextSend => "text.send",
        CapabilityKey::LinksSend => "links.send",
        CapabilityKey::MediaRead => "media.read",
        CapabilityKey::MediaControl => "media.control",
        CapabilityKey::NotificationsReceive => "notifications.receive",
        CapabilityKey::CommandsExecute => "commands.execute",
    }
}

fn parse_platform(value: &str) -> Result<DevicePlatform, DiscoveryError> {
    match value {
        "linux" => Ok(DevicePlatform::Linux),
        "android" => Ok(DevicePlatform::Android),
        "ios" => Ok(DevicePlatform::Ios),
        "windows" => Ok(DevicePlatform::Windows),
        _ => Err(DiscoveryError::UnsupportedProperty("platform")),
    }
}

fn validate_version(
    announcement: &DiscoveryAnnouncement,
    property: &'static str,
    expected: &str,
) -> Result<(), DiscoveryError> {
    let value = required_property(announcement, property)?;
    if value == expected {
        Ok(())
    } else {
        Err(DiscoveryError::UnsupportedProperty(property))
    }
}

fn required_property<'a>(
    announcement: &'a DiscoveryAnnouncement,
    key: &'static str,
) -> Result<&'a str, DiscoveryError> {
    announcement
        .properties
        .get(key)
        .map(String::as_str)
        .ok_or(DiscoveryError::MissingProperty(key))
}

fn is_service_type(value: &str) -> bool {
    value.eq_ignore_ascii_case(SERVICE_TYPE)
}

fn instance_name(fullname: &str) -> Result<String, DiscoveryError> {
    let suffix_start = fullname
        .len()
        .checked_sub(SERVICE_TYPE.len())
        .ok_or(DiscoveryError::InvalidFullname)?;
    if !fullname[suffix_start..].eq_ignore_ascii_case(SERVICE_TYPE) {
        return Err(DiscoveryError::InvalidFullname);
    }
    let instance = &fullname[..suffix_start];
    if instance.is_empty() || instance.chars().any(char::is_control) {
        return Err(DiscoveryError::InvalidFullname);
    }
    Ok(instance.to_owned())
}

fn normalize_name(value: &str) -> String {
    value.to_ascii_lowercase()
}

fn candidate_id(service_key: &str, generation: u64) -> CandidateId {
    let mut digest = Sha256::new();
    digest.update(service_key.as_bytes());
    digest.update(generation.to_le_bytes());
    let digest = digest.finalize();
    let value = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    CandidateId(format!("candidate-{value}"))
}

#[derive(Clone)]
pub struct CandidateStore {
    inner: Arc<Mutex<CandidateRegistry>>,
}

impl CandidateStore {
    pub fn new(ttl: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(CandidateRegistry::new(ttl))),
        }
    }

    pub fn candidates(&self) -> Vec<DiscoveryCandidate> {
        self.inner
            .lock()
            .map(|registry| registry.candidates())
            .unwrap_or_default()
    }

    pub fn ttl(&self) -> Duration {
        self.inner
            .lock()
            .map(|registry| registry.ttl())
            .unwrap_or(CANDIDATE_TTL)
    }

    fn upsert(
        &self,
        announcement: DiscoveryAnnouncement,
        observed_at: UtcTimestamp,
        now: Instant,
        expires_at: UtcTimestamp,
    ) {
        if let Ok(mut registry) = self.inner.lock() {
            let _ = registry.upsert(announcement, observed_at, now, expires_at);
        }
    }

    fn remove(&self, service_type: &str, fullname: &str, observed_at: UtcTimestamp) {
        if let Ok(mut registry) = self.inner.lock() {
            let _ = registry.remove(service_type, fullname, observed_at);
        }
    }

    fn expire(&self, now: Instant, observed_at: UtcTimestamp) {
        if let Ok(mut registry) = self.inner.lock() {
            let _ = registry.expire(now, observed_at);
        }
    }
}

pub struct DiscoveryService {
    store: CandidateStore,
    daemon: Option<ServiceDaemon>,
    worker: Option<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
}

impl DiscoveryService {
    pub fn new() -> Self {
        Self::with_ttl(CANDIDATE_TTL)
    }

    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            store: CandidateStore::new(ttl),
            daemon: None,
            worker: None,
            stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn store(&self) -> CandidateStore {
        self.store.clone()
    }
}

impl Default for DiscoveryService {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeService for DiscoveryService {
    fn kind(&self) -> ServiceKind {
        ServiceKind::Discovery
    }

    fn start(&mut self) -> Result<(), ServiceFailureCode> {
        if self.worker.is_some() {
            return Ok(());
        }

        self.stop.store(false, Ordering::Release);
        let daemon = ServiceDaemon::new().map_err(|_| ServiceFailureCode::InitializationFailed)?;
        let receiver = daemon
            .browse(SERVICE_TYPE)
            .map_err(|_| ServiceFailureCode::InitializationFailed)?;
        let stop = Arc::clone(&self.stop);
        let store = self.store.clone();
        let worker = thread::Builder::new()
            .name("pulse-discovery".to_owned())
            .spawn(move || {
                while !stop.load(Ordering::Acquire) {
                    if let Ok(event) = receiver.try_recv() {
                        handle_service_event(&store, event);
                    } else {
                        store.expire(Instant::now(), UtcTimestamp(utc_timestamp()));
                        thread::sleep(DISCOVERY_POLL_INTERVAL);
                    }
                }
            })
            .map_err(|_| ServiceFailureCode::InitializationFailed)?;

        self.daemon = Some(daemon);
        self.worker = Some(worker);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ServiceFailureCode> {
        self.stop.store(true, Ordering::Release);
        if let Some(daemon) = self.daemon.take() {
            let _ = daemon.stop_browse(SERVICE_TYPE);
            let _ = daemon.shutdown();
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        Ok(())
    }
}

fn handle_service_event(store: &CandidateStore, event: ServiceEvent) {
    match event {
        ServiceEvent::ServiceResolved(service) => {
            let Ok(announcement) = DiscoveryAnnouncement::try_from(service.as_ref()) else {
                return;
            };
            let now = SystemTime::now();
            let expires_at = now.checked_add(store.ttl()).unwrap_or(now);
            store.upsert(
                announcement,
                UtcTimestamp(utc_timestamp_at(now)),
                Instant::now(),
                UtcTimestamp(utc_timestamp_at(expires_at)),
            );
        }
        ServiceEvent::ServiceRemoved(service_type, fullname) => {
            store.remove(&service_type, &fullname, UtcTimestamp(utc_timestamp()));
        }
        _ => {}
    }
}

fn utc_timestamp() -> String {
    utc_timestamp_at(SystemTime::now())
}

fn utc_timestamp_at(time: SystemTime) -> String {
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let seconds = duration.as_secs() as i64;
    let days = seconds / 86_400;
    let seconds_in_day = seconds % 86_400;
    let (year, month, day) = civil_date_from_days(days);
    let hour = seconds_in_day / 3_600;
    let minute = (seconds_in_day % 3_600) / 60;
    let second = seconds_in_day % 60;

    format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{:03}Z",
        duration.subsec_millis()
    )
}

fn civil_date_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let shifted_days = days_since_epoch + 719_468;
    let era = if shifted_days >= 0 {
        shifted_days
    } else {
        shifted_days - 146_096
    } / 146_097;
    let day_of_era = shifted_days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}
