//! Contract tests for the fail-closed provider-free plugin lifecycle runtime.

use contextlab_mcp::{
    CapabilityAvailability, CapabilityCompatibility, CapabilityDescriptor,
    CapabilityDiagnosticCode, CapabilityKind, LifecycleContract, LifecyclePhase, PluginId,
    PluginManifest, Version,
};
use contextlab_plugin_runtime::{
    LoadPhase, Plugin, PluginBundle, PluginError, PluginFactory, PluginLoadError, PluginRuntime,
};
use std::sync::{Arc, Mutex};

fn manifest(id: &str, capability_id: &str) -> PluginManifest {
    PluginManifest::new(
        1,
        PluginId::new(id).expect("valid plugin id"),
        format!("{id} plugin"),
        Version::new(1, 0, 0),
        vec![
            CapabilityDescriptor::new(capability_id, CapabilityKind::Tool, Version::new(1, 0, 0))
                .expect("valid capability"),
        ],
        LifecycleContract::new(
            Version::new(1, 0, 0),
            [
                LifecyclePhase::Initialize,
                LifecyclePhase::Activate,
                LifecyclePhase::Deactivate,
                LifecyclePhase::Shutdown,
            ],
        )
        .expect("valid lifecycle"),
    )
    .expect("valid manifest")
}

#[derive(Clone)]
struct FixtureFactory {
    events: Arc<Mutex<Vec<String>>>,
    failure: Option<LoadPhase>,
}

struct FixturePlugin {
    id: String,
    events: Arc<Mutex<Vec<String>>>,
    failure: Option<LoadPhase>,
}

impl PluginFactory for FixtureFactory {
    fn load(&self, manifest: &PluginManifest) -> Result<Box<dyn Plugin>, PluginLoadError> {
        self.events
            .lock()
            .expect("events lock")
            .push(format!("{}:load", manifest.id()));
        if self.failure == Some(LoadPhase::Factory) {
            return Err(PluginLoadError::new("factory rejected plugin"));
        }
        Ok(Box::new(FixturePlugin {
            id: manifest.id().to_owned(),
            events: Arc::clone(&self.events),
            failure: self.failure,
        }))
    }
}

impl FixturePlugin {
    fn event(&self, phase: &str) -> Result<(), PluginError> {
        self.events
            .lock()
            .expect("events lock")
            .push(format!("{}:{phase}", self.id));
        let expected = match phase {
            "initialize" => LoadPhase::Initialize,
            "activate" => LoadPhase::Activate,
            "deactivate" => LoadPhase::Deactivate,
            "shutdown" => LoadPhase::Shutdown,
            _ => unreachable!("fixture phase is known"),
        };
        if self.failure == Some(expected) {
            return Err(PluginError::new(format!("{phase} rejected plugin")));
        }
        Ok(())
    }
}

impl Plugin for FixturePlugin {
    fn initialize(&mut self) -> Result<(), PluginError> {
        self.event("initialize")
    }

    fn activate(&mut self) -> Result<(), PluginError> {
        self.event("activate")
    }

    fn deactivate(&mut self) -> Result<(), PluginError> {
        self.event("deactivate")
    }

    fn shutdown(&mut self) -> Result<(), PluginError> {
        self.event("shutdown")
    }
}

fn bundle(
    source: &str,
    manifest: PluginManifest,
    events: &Arc<Mutex<Vec<String>>>,
    failure: Option<LoadPhase>,
) -> PluginBundle {
    PluginBundle::new(
        source,
        manifest,
        Box::new(FixtureFactory {
            events: Arc::clone(events),
            failure,
        }),
    )
}

#[test]
fn load_report_projects_safe_deterministic_capability_availability() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut runtime = PluginRuntime::current();

    let report = runtime.load_all(vec![
        bundle(
            "30-zeta.json",
            manifest("zeta.plugin", "zeta.tool"),
            &events,
            Some(LoadPhase::Activate),
        ),
        bundle(
            "10-alpha.json",
            manifest("alpha.plugin", "alpha.tool"),
            &events,
            None,
        ),
    ]);

    let projection = report.capability_availability();

    assert_eq!(projection.version(), Version::new(1, 0, 0));
    assert_eq!(
        projection
            .entries()
            .iter()
            .map(|entry| (
                entry.plugin_id().as_str(),
                entry.capability_id().as_str(),
                entry.capability_version(),
                entry.availability(),
                entry.compatibility(),
                entry.diagnostic_code(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                "alpha.plugin",
                "alpha.tool",
                Version::new(1, 0, 0),
                CapabilityAvailability::Available,
                CapabilityCompatibility::Compatible,
                None,
            ),
            (
                "zeta.plugin",
                "zeta.tool",
                Version::new(1, 0, 0),
                CapabilityAvailability::Unavailable,
                CapabilityCompatibility::Compatible,
                Some(CapabilityDiagnosticCode::ActivationRejected),
            ),
        ]
    );
    assert!(projection.entries().iter().all(|entry| {
        entry.diagnostic_code().is_none()
            || entry
                .diagnostic_code()
                .unwrap()
                .as_str()
                .contains("rejected")
    }));
}

#[test]
fn runtime_loads_in_deterministic_order_and_exposes_capabilities_after_activation() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut runtime = PluginRuntime::current();

    let report = runtime.load_all(vec![
        bundle(
            "20-beta.json",
            manifest("beta.plugin", "beta.tool"),
            &events,
            None,
        ),
        bundle(
            "10-alpha.json",
            manifest("alpha.plugin", "alpha.tool"),
            &events,
            None,
        ),
    ]);

    assert_eq!(
        report.loaded_plugins(),
        &[
            PluginId::new("alpha.plugin").unwrap(),
            PluginId::new("beta.plugin").unwrap()
        ]
    );
    assert!(report.diagnostics().is_empty());
    assert_eq!(runtime.registry().capabilities().count(), 2);
    assert_eq!(
        *events.lock().expect("events lock"),
        vec![
            "alpha.plugin:load",
            "alpha.plugin:initialize",
            "alpha.plugin:activate",
            "beta.plugin:load",
            "beta.plugin:initialize",
            "beta.plugin:activate",
        ]
    );
}

#[test]
fn one_load_failure_does_not_expose_partial_capabilities_or_block_other_plugins() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut runtime = PluginRuntime::current();

    let report = runtime.load_all(vec![
        bundle(
            "20-good.json",
            manifest("good.plugin", "good.tool"),
            &events,
            None,
        ),
        bundle(
            "10-bad.json",
            manifest("bad.plugin", "bad.tool"),
            &events,
            Some(LoadPhase::Factory),
        ),
    ]);

    assert_eq!(report.loaded_plugins().len(), 1);
    assert_eq!(report.loaded_plugins()[0].as_str(), "good.plugin");
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(report.diagnostics()[0].source(), "10-bad.json");
    assert_eq!(report.diagnostics()[0].phase(), LoadPhase::Factory);
    assert!(runtime.registry().capability("bad.tool").is_none());
    assert!(runtime.registry().capability("good.tool").is_some());
}

#[test]
fn activation_failure_shuts_down_and_remains_fail_closed() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut runtime = PluginRuntime::current();

    let report = runtime.load_all(vec![bundle(
        "10-failing.json",
        manifest("failing.plugin", "failing.tool"),
        &events,
        Some(LoadPhase::Activate),
    )]);

    assert!(report.loaded_plugins().is_empty());
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(report.diagnostics()[0].phase(), LoadPhase::Activate);
    assert!(runtime.registry().capabilities().next().is_none());
    assert_eq!(
        *events.lock().expect("events lock"),
        vec![
            "failing.plugin:load",
            "failing.plugin:initialize",
            "failing.plugin:activate",
            "failing.plugin:shutdown",
        ]
    );
}
