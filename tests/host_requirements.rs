#![cfg(feature = "serde")]

use greentic_types::deployment::DeploymentPlan;
use greentic_types::deployment::host::*;
use serde_json::{Value, json};

fn fixture() -> Value {
    json!({
        "schema_version":"1.0.0","os":"linux","architecture":"aarch64",
        "network":"bridge",
        "devices":[{"source":"/dev/serial/by-id/usb-reference","target":"/dev/coordinator","access":"read_write"}],
        "sockets":[],
        "mounts":[{"source":"/var/lib/example","target":"/data","read_only":false,"owner_uid":1000,"owner_gid":1000}],
        "ports":[{"host_port":8080,"workload_port":8080,"protocol":"tcp","bind":"loopback"}],
        "health":{"port":8080,"path":"/health","startup_timeout_seconds":60}
    })
}

#[test]
fn existing_deployment_plans_remain_unchanged_and_opt_into_the_wrapper() {
    let legacy = json!({"pack_id":"example","pack_version":"1.0.0",
        "tenant":"tenant-a","environment":"local","runners":[],
        "channels":[],"secrets":[],"oauth":[],"extra":{"custom":"retained"}});
    let plan: DeploymentPlan = serde_json::from_value(legacy.clone()).expect("old deployment plan");
    assert_eq!(serde_json::to_value(&plan).expect("serialize"), legacy);
    let wrapper = HostDeploymentPlan {
        host: serde_json::from_value(fixture()).expect("host requirements"),
        deployment: plan,
    };
    let serialized = serde_json::to_value(&wrapper).expect("wrapper");
    assert_eq!(serialized["deployment"], legacy);
    assert_eq!(
        serde_json::from_value::<HostDeploymentPlan>(serialized).expect("reload"),
        wrapper
    );
}

#[test]
fn arm64_requirements_round_trip_without_losing_privilege_bounds() {
    let value = fixture();
    let requirements: HostRequirements =
        serde_json::from_value(value.clone()).expect("requirements");
    assert_eq!(requirements.validate(), Ok(()));
    assert_eq!(
        serde_json::to_value(&requirements).expect("serialize"),
        value
    );
    let mut x86 = requirements;
    x86.architecture = HostArchitecture::X86_64;
    assert_eq!(x86.validate(), Ok(()));
}

#[test]
fn unknown_versions_fields_architectures_and_privileges_fail_closed() {
    for (field, value) in [
        ("schema_version", json!("2.0.0")),
        ("architecture", json!("armv7")),
        ("os", json!("windows")),
        ("privileged", json!(true)),
        ("capabilities", json!(["SYS_ADMIN"])),
    ] {
        let mut request = fixture();
        request[field] = value;
        assert!(
            serde_json::from_value::<HostRequirements>(request).is_err(),
            "{field}"
        );
    }
}

#[test]
fn unsafe_paths_and_device_directory_requests_are_denied() {
    for path in [
        "/",
        "relative",
        "/dev",
        "/dev/../mem",
        "/dev/*",
        "/dev//ttyUSB0",
        "/dev/ttyUSB0\n--privileged",
        "/dev/%n",
        "/dev/device:rw",
    ] {
        let mut request = fixture();
        request["devices"][0]["source"] = json!(path);
        let requirements: HostRequirements = serde_json::from_value(request).expect("syntax");
        assert!(requirements.validate().is_err(), "{path}");
    }
    for path in ["/dev", "/dev/serial", "/proc", "/sys", "/etc", "/run"] {
        let mut request = fixture();
        request["mounts"][0]["source"] = json!(path);
        let requirements: HostRequirements = serde_json::from_value(request).expect("syntax");
        assert!(requirements.validate().is_err(), "{path}");
    }
}

#[test]
fn overlapping_targets_and_unbounded_requests_are_denied() {
    let mut request = fixture();
    request["sockets"] = json!([{"source":"/run/example.sock","target":"/data/socket"}]);
    assert_eq!(
        serde_json::from_value::<HostRequirements>(request)
            .expect("syntax")
            .validate(),
        Err(HostRequirementError::DuplicateTarget)
    );
    let mut requirements: HostRequirements = serde_json::from_value(fixture()).expect("syntax");
    requirements.devices = vec![requirements.devices[0].clone(); 33];
    assert_eq!(
        requirements.validate(),
        Err(HostRequirementError::TooManyMappings)
    );
}

#[test]
fn contradictory_ports_and_unbounded_health_checks_are_denied() {
    let base: HostRequirements = serde_json::from_value(fixture()).expect("syntax");
    let mut requirements = base.clone();
    requirements.network = HostNetworkMode::None;
    assert_eq!(
        requirements.validate(),
        Err(HostRequirementError::InvalidPort)
    );
    let mut requirements = base.clone();
    requirements.network = HostNetworkMode::Host;
    requirements.ports[0].host_port = 8081;
    assert_eq!(
        requirements.validate(),
        Err(HostRequirementError::InvalidPort)
    );
    let mut requirements = base.clone();
    requirements.ports.push(requirements.ports[0].clone());
    assert_eq!(
        requirements.validate(),
        Err(HostRequirementError::InvalidPort)
    );
    for timeout in [0, 301] {
        let mut requirements = base.clone();
        requirements.health.startup_timeout_seconds = timeout;
        assert_eq!(
            requirements.validate(),
            Err(HostRequirementError::InvalidHealth)
        );
    }
    for path in [
        "https://example.org/health",
        "//example.org/health",
        "/health?command=run",
    ] {
        let mut requirements = base.clone();
        requirements.health.path = path.into();
        assert_eq!(
            requirements.validate(),
            Err(HostRequirementError::InvalidHealth)
        );
    }
}
