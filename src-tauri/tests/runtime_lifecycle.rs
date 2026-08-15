use std::sync::{Arc, Mutex};

use pulse_lib::runtime::{
    LifecycleOperation, LifecycleStage, Runtime, RuntimeBuilder, RuntimeError, RuntimePhase,
    RuntimeService, RuntimeState, ServiceFailureCode, ServiceKind, ServiceStatus,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Call {
    Start(ServiceKind),
    Stop(ServiceKind),
}

struct FakeService {
    kind: ServiceKind,
    calls: Arc<Mutex<Vec<Call>>>,
    start_failure: Option<ServiceFailureCode>,
    stop_failure: Option<ServiceFailureCode>,
}

impl FakeService {
    fn new(kind: ServiceKind, calls: Arc<Mutex<Vec<Call>>>) -> Self {
        Self {
            kind,
            calls,
            start_failure: None,
            stop_failure: None,
        }
    }

    fn failing_start(mut self, code: ServiceFailureCode) -> Self {
        self.start_failure = Some(code);
        self
    }

    fn failing_stop(mut self, code: ServiceFailureCode) -> Self {
        self.stop_failure = Some(code);
        self
    }
}

impl RuntimeService for FakeService {
    fn kind(&self) -> ServiceKind {
        self.kind
    }

    fn start(&mut self) -> Result<(), ServiceFailureCode> {
        self.calls.lock().unwrap().push(Call::Start(self.kind));
        self.start_failure.take().map_or(Ok(()), Err)
    }

    fn stop(&mut self) -> Result<(), ServiceFailureCode> {
        self.calls.lock().unwrap().push(Call::Stop(self.kind));
        self.stop_failure.take().map_or(Ok(()), Err)
    }
}

fn status_of(snapshot: &pulse_lib::runtime::RuntimeSnapshot, kind: ServiceKind) -> ServiceStatus {
    snapshot
        .services
        .iter()
        .find(|service| service.kind == kind)
        .map(|service| service.status)
        .expect("service must exist in runtime snapshot")
}

#[test]
fn default_runtime_is_partial_without_claiming_services_are_available() {
    let mut runtime = Runtime::default();

    let snapshot = runtime
        .start()
        .expect("default runtime should start partially");

    assert_eq!(snapshot.phase, RuntimePhase::Partial);
    assert!(snapshot
        .services
        .iter()
        .all(|service| service.status == ServiceStatus::NotConfigured));
    assert_eq!(runtime.shutdown().unwrap().phase, RuntimePhase::Stopped);
}

#[test]
fn start_and_shutdown_follow_the_declared_service_order() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let runtime = RuntimeBuilder::new()
        .register(FakeService::new(ServiceKind::Protocol, calls.clone()))
        .unwrap()
        .register(FakeService::new(ServiceKind::Storage, calls.clone()))
        .unwrap()
        .register(FakeService::new(ServiceKind::Discovery, calls.clone()))
        .unwrap()
        .configure_inactive(ServiceKind::Transfer)
        .unwrap()
        .build();
    let mut runtime = runtime;

    let snapshot = runtime.start().unwrap();

    assert_eq!(snapshot.phase, RuntimePhase::Partial);
    assert_eq!(
        status_of(&snapshot, ServiceKind::Transfer),
        ServiceStatus::Inactive
    );
    assert_eq!(
        *calls.lock().unwrap(),
        vec![
            Call::Start(ServiceKind::Storage),
            Call::Start(ServiceKind::Discovery),
            Call::Start(ServiceKind::Protocol),
        ]
    );

    runtime.shutdown().unwrap();

    assert_eq!(
        *calls.lock().unwrap(),
        vec![
            Call::Start(ServiceKind::Storage),
            Call::Start(ServiceKind::Discovery),
            Call::Start(ServiceKind::Protocol),
            Call::Stop(ServiceKind::Protocol),
            Call::Stop(ServiceKind::Discovery),
            Call::Stop(ServiceKind::Storage),
        ]
    );
}

#[test]
fn failed_start_stops_the_failed_service_and_previous_services() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let failing_service = FakeService::new(ServiceKind::Discovery, calls.clone())
        .failing_start(ServiceFailureCode::DependencyUnavailable);
    let mut runtime = RuntimeBuilder::new()
        .register(FakeService::new(ServiceKind::Storage, calls.clone()))
        .unwrap()
        .register(failing_service)
        .unwrap()
        .register(FakeService::new(ServiceKind::Protocol, calls.clone()))
        .unwrap()
        .build();

    let error = runtime.start().expect_err("discovery should fail to start");

    assert_eq!(
        error.lifecycle_errors(),
        &[pulse_lib::runtime::LifecycleError {
            service: ServiceKind::Discovery,
            stage: LifecycleStage::Start,
            code: ServiceFailureCode::DependencyUnavailable,
        },]
    );
    assert_eq!(runtime.phase(), RuntimePhase::Failed);
    let snapshot = runtime.snapshot();
    assert_eq!(
        status_of(&snapshot, ServiceKind::Storage),
        ServiceStatus::Stopped
    );
    assert_eq!(
        status_of(&snapshot, ServiceKind::Discovery),
        ServiceStatus::Stopped
    );
    assert_eq!(
        status_of(&snapshot, ServiceKind::Protocol),
        ServiceStatus::Stopped
    );
    assert_eq!(
        *calls.lock().unwrap(),
        vec![
            Call::Start(ServiceKind::Storage),
            Call::Start(ServiceKind::Discovery),
            Call::Stop(ServiceKind::Discovery),
            Call::Stop(ServiceKind::Storage),
        ]
    );
}

#[test]
fn cleanup_failure_is_returned_without_skipping_other_cleanup() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut runtime = RuntimeBuilder::new()
        .register(
            FakeService::new(ServiceKind::Storage, calls.clone())
                .failing_stop(ServiceFailureCode::ShutdownFailed),
        )
        .unwrap()
        .register(
            FakeService::new(ServiceKind::Discovery, calls.clone())
                .failing_start(ServiceFailureCode::InitializationFailed),
        )
        .unwrap()
        .build();

    let error = runtime.start().expect_err("discovery should fail to start");

    assert_eq!(runtime.phase(), RuntimePhase::Failed);
    assert_eq!(error.lifecycle_errors().len(), 2);
    assert_eq!(error.lifecycle_errors()[1].service, ServiceKind::Storage);
    assert_eq!(error.lifecycle_errors()[1].stage, LifecycleStage::Stop);
    assert_eq!(
        status_of(&runtime.snapshot(), ServiceKind::Storage),
        ServiceStatus::Failed
    );
    assert_eq!(
        *calls.lock().unwrap(),
        vec![
            Call::Start(ServiceKind::Storage),
            Call::Start(ServiceKind::Discovery),
            Call::Stop(ServiceKind::Discovery),
            Call::Stop(ServiceKind::Storage),
        ]
    );
}

#[test]
fn shutdown_failure_is_propagated_and_other_services_continue() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut runtime = RuntimeBuilder::new()
        .register(
            FakeService::new(ServiceKind::Storage, calls.clone())
                .failing_stop(ServiceFailureCode::ShutdownFailed),
        )
        .unwrap()
        .register(FakeService::new(ServiceKind::Discovery, calls.clone()))
        .unwrap()
        .build();

    runtime.start().unwrap();
    let error = runtime
        .shutdown()
        .expect_err("storage shutdown should fail");

    assert_eq!(runtime.phase(), RuntimePhase::Failed);
    assert!(error
        .lifecycle_errors()
        .iter()
        .any(|error| error.service == ServiceKind::Storage));
    assert_eq!(
        status_of(&runtime.snapshot(), ServiceKind::Discovery),
        ServiceStatus::Stopped
    );
    assert_eq!(
        status_of(&runtime.snapshot(), ServiceKind::Storage),
        ServiceStatus::Failed
    );
    assert_eq!(
        *calls.lock().unwrap(),
        vec![
            Call::Start(ServiceKind::Storage),
            Call::Start(ServiceKind::Discovery),
            Call::Stop(ServiceKind::Discovery),
            Call::Stop(ServiceKind::Storage),
        ]
    );
}

#[test]
fn runtime_state_is_shareable_and_shutdown_is_idempotent_after_success() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let runtime = RuntimeBuilder::new()
        .register(FakeService::new(ServiceKind::Storage, calls.clone()))
        .unwrap()
        .build();
    let state = RuntimeState::new(runtime);
    let other_state = state.clone();

    state.start().unwrap();
    assert_eq!(other_state.snapshot().unwrap().phase, RuntimePhase::Partial);
    other_state.shutdown().unwrap();
    state.shutdown().unwrap();

    assert_eq!(
        *calls.lock().unwrap(),
        vec![
            Call::Start(ServiceKind::Storage),
            Call::Stop(ServiceKind::Storage)
        ]
    );
}

#[test]
fn invalid_start_transition_is_reported() {
    let state = RuntimeState::default();
    state.start().unwrap();

    let error = state.start().expect_err("second start must be rejected");

    assert_eq!(
        error,
        RuntimeError::InvalidTransition {
            operation: LifecycleOperation::Start,
            phase: RuntimePhase::Partial,
        }
    );
}
