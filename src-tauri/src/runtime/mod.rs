use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, Mutex, MutexGuard};

const SERVICE_ORDER: [ServiceKind; 10] = [
    ServiceKind::Storage,
    ServiceKind::Identity,
    ServiceKind::DeviceRegistry,
    ServiceKind::Discovery,
    ServiceKind::Pairing,
    ServiceKind::Protocol,
    ServiceKind::Transfer,
    ServiceKind::Clipboard,
    ServiceKind::Media,
    ServiceKind::Notifications,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceKind {
    Storage,
    Identity,
    DeviceRegistry,
    Discovery,
    Pairing,
    Protocol,
    Transfer,
    Clipboard,
    Media,
    Notifications,
}

impl ServiceKind {
    pub const fn all() -> &'static [Self] {
        &SERVICE_ORDER
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceStatus {
    NotConfigured,
    Inactive,
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimePhase {
    Created,
    Starting,
    Partial,
    Ready,
    Failed,
    Stopping,
    Stopped,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleOperation {
    Configure,
    Start,
    Shutdown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleStage {
    Start,
    Stop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceFailureCode {
    InitializationFailed,
    DependencyUnavailable,
    ShutdownFailed,
    InternalState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LifecycleError {
    pub service: ServiceKind,
    pub stage: LifecycleStage,
    pub code: ServiceFailureCode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeError {
    InvalidTransition {
        operation: LifecycleOperation,
        phase: RuntimePhase,
    },
    Lifecycle {
        errors: Vec<LifecycleError>,
    },
    StateUnavailable,
}

impl RuntimeError {
    pub fn lifecycle_errors(&self) -> &[LifecycleError] {
        match self {
            Self::Lifecycle { errors } => errors,
            Self::InvalidTransition { .. } | Self::StateUnavailable => &[],
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTransition { operation, phase } => {
                write!(
                    formatter,
                    "invalid runtime {operation:?} from phase {phase:?}"
                )
            }
            Self::Lifecycle { errors } => {
                write!(
                    formatter,
                    "runtime lifecycle failed for {} service(s)",
                    errors.len()
                )
            }
            Self::StateUnavailable => formatter.write_str("runtime state is unavailable"),
        }
    }
}

impl Error for RuntimeError {}

pub trait RuntimeService: Send {
    fn kind(&self) -> ServiceKind;

    fn start(&mut self) -> Result<(), ServiceFailureCode>;

    fn stop(&mut self) -> Result<(), ServiceFailureCode>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceSnapshot {
    pub kind: ServiceKind,
    pub status: ServiceStatus,
    pub failure: Option<ServiceFailureCode>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeSnapshot {
    pub phase: RuntimePhase,
    pub services: Vec<ServiceSnapshot>,
}

struct ServiceSlot {
    kind: ServiceKind,
    status: ServiceStatus,
    failure: Option<ServiceFailureCode>,
    service: Option<Box<dyn RuntimeService>>,
}

impl ServiceSlot {
    fn not_configured(kind: ServiceKind) -> Self {
        Self {
            kind,
            status: ServiceStatus::NotConfigured,
            failure: None,
            service: None,
        }
    }

    fn snapshot(&self) -> ServiceSnapshot {
        ServiceSnapshot {
            kind: self.kind,
            status: self.status,
            failure: self.failure,
        }
    }
}

pub struct RuntimeBuilder {
    services: Vec<ServiceSlot>,
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        Self {
            services: SERVICE_ORDER
                .iter()
                .copied()
                .map(ServiceSlot::not_configured)
                .collect(),
        }
    }

    pub fn register<S>(mut self, service: S) -> Result<Self, RuntimeBuildError>
    where
        S: RuntimeService + 'static,
    {
        let kind = service.kind();
        let slot = self
            .services
            .iter_mut()
            .find(|slot| slot.kind == kind)
            .ok_or(RuntimeBuildError::UnknownService { kind })?;

        if slot.status != ServiceStatus::NotConfigured {
            return Err(RuntimeBuildError::AlreadyConfigured { kind });
        }

        slot.status = ServiceStatus::Stopped;
        slot.service = Some(Box::new(service));
        Ok(self)
    }

    pub fn configure_inactive(mut self, kind: ServiceKind) -> Result<Self, RuntimeBuildError> {
        let slot = self
            .services
            .iter_mut()
            .find(|slot| slot.kind == kind)
            .ok_or(RuntimeBuildError::UnknownService { kind })?;

        if slot.status != ServiceStatus::NotConfigured {
            return Err(RuntimeBuildError::AlreadyConfigured { kind });
        }

        slot.status = ServiceStatus::Inactive;
        Ok(self)
    }

    pub fn build(self) -> Runtime {
        Runtime {
            phase: RuntimePhase::Created,
            services: self.services,
        }
    }
}

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeBuildError {
    UnknownService { kind: ServiceKind },
    AlreadyConfigured { kind: ServiceKind },
}

impl Display for RuntimeBuildError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownService { kind } => write!(formatter, "unknown service {kind:?}"),
            Self::AlreadyConfigured { kind } => {
                write!(formatter, "service {kind:?} is already configured")
            }
        }
    }
}

impl Error for RuntimeBuildError {}

pub struct Runtime {
    phase: RuntimePhase,
    services: Vec<ServiceSlot>,
}

impl Runtime {
    pub fn new() -> Self {
        RuntimeBuilder::new().build()
    }

    pub fn phase(&self) -> RuntimePhase {
        self.phase
    }

    pub fn snapshot(&self) -> RuntimeSnapshot {
        RuntimeSnapshot {
            phase: self.phase,
            services: self.services.iter().map(ServiceSlot::snapshot).collect(),
        }
    }

    pub fn start(&mut self) -> Result<RuntimeSnapshot, RuntimeError> {
        if self.phase != RuntimePhase::Created {
            return Err(RuntimeError::InvalidTransition {
                operation: LifecycleOperation::Start,
                phase: self.phase,
            });
        }

        self.phase = RuntimePhase::Starting;
        let mut cleanup_indices = Vec::new();
        let mut errors = Vec::new();

        for index in 0..self.services.len() {
            if self.services[index].service.is_none() {
                continue;
            }

            self.services[index].status = ServiceStatus::Starting;
            let result = match self.services[index].service.as_mut() {
                Some(service) => service.start(),
                None => Err(ServiceFailureCode::InternalState),
            };

            cleanup_indices.push(index);
            match result {
                Ok(()) => {
                    self.services[index].failure = None;
                    self.services[index].status = ServiceStatus::Running;
                }
                Err(code) => {
                    self.services[index].failure = Some(code);
                    self.services[index].status = ServiceStatus::Failed;
                    errors.push(LifecycleError {
                        service: self.services[index].kind,
                        stage: LifecycleStage::Start,
                        code,
                    });
                    break;
                }
            }
        }

        if errors.is_empty() {
            self.phase = if self.all_services_running() {
                RuntimePhase::Ready
            } else {
                RuntimePhase::Partial
            };
            return Ok(self.snapshot());
        }

        for index in cleanup_indices.into_iter().rev() {
            self.stop_slot(index, &mut errors);
        }
        self.phase = RuntimePhase::Failed;
        Err(RuntimeError::Lifecycle { errors })
    }

    pub fn shutdown(&mut self) -> Result<RuntimeSnapshot, RuntimeError> {
        if self.phase == RuntimePhase::Stopped {
            return Ok(self.snapshot());
        }

        if matches!(self.phase, RuntimePhase::Starting | RuntimePhase::Stopping) {
            return Err(RuntimeError::InvalidTransition {
                operation: LifecycleOperation::Shutdown,
                phase: self.phase,
            });
        }

        self.phase = RuntimePhase::Stopping;
        let mut errors = Vec::new();
        for index in (0..self.services.len()).rev() {
            if matches!(
                self.services[index].status,
                ServiceStatus::Running | ServiceStatus::Failed
            ) {
                self.stop_slot(index, &mut errors);
            }
        }

        if errors.is_empty() {
            self.phase = RuntimePhase::Stopped;
            Ok(self.snapshot())
        } else {
            self.phase = RuntimePhase::Failed;
            Err(RuntimeError::Lifecycle { errors })
        }
    }

    fn all_services_running(&self) -> bool {
        self.services
            .iter()
            .all(|slot| slot.status == ServiceStatus::Running)
    }

    fn stop_slot(&mut self, index: usize, errors: &mut Vec<LifecycleError>) {
        self.services[index].status = ServiceStatus::Stopping;
        let result = match self.services[index].service.as_mut() {
            Some(service) => service.stop(),
            None => Err(ServiceFailureCode::InternalState),
        };

        match result {
            Ok(()) => {
                self.services[index].failure = None;
                self.services[index].status = ServiceStatus::Stopped;
            }
            Err(code) => {
                self.services[index].failure = Some(code);
                self.services[index].status = ServiceStatus::Failed;
                errors.push(LifecycleError {
                    service: self.services[index].kind,
                    stage: LifecycleStage::Stop,
                    code,
                });
            }
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct RuntimeState {
    inner: Arc<Mutex<Runtime>>,
}

impl RuntimeState {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            inner: Arc::new(Mutex::new(runtime)),
        }
    }

    pub fn configure(&self, runtime: Runtime) -> Result<(), RuntimeError> {
        let mut current = self.lock()?;
        if current.phase != RuntimePhase::Created {
            return Err(RuntimeError::InvalidTransition {
                operation: LifecycleOperation::Configure,
                phase: current.phase,
            });
        }
        *current = runtime;
        Ok(())
    }

    pub fn start(&self) -> Result<RuntimeSnapshot, RuntimeError> {
        self.lock()?.start()
    }

    pub fn shutdown(&self) -> Result<RuntimeSnapshot, RuntimeError> {
        self.lock()?.shutdown()
    }

    pub fn snapshot(&self) -> Result<RuntimeSnapshot, RuntimeError> {
        Ok(self.lock()?.snapshot())
    }

    fn lock(&self) -> Result<MutexGuard<'_, Runtime>, RuntimeError> {
        self.inner
            .lock()
            .map_err(|_| RuntimeError::StateUnavailable)
    }
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self::new(Runtime::default())
    }
}
