use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn run(command: &str, input: &str) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_kerf"))
        .arg(command)
        .arg(repo_path(input))
        .output()
        .expect("kerf must run");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("stdout must be JSON")
}

#[test]
fn quotient_example_is_input_bound_json() {
    let output = run("quotient", "examples/toggle.lts.json");
    assert_eq!(output["schema_version"], "kerf.result.v1");
    assert_eq!(output["operation"], "quotient");
    assert_eq!(output["result"]["equivalence_class_count"], 2);
    assert_eq!(
        output["result"]["unreachable_machine_states"],
        serde_json::json!(["unreachable-copy"])
    );
    assert_eq!(output["input_sha256"].as_str().map(str::len), Some(64));
}

#[test]
fn distinguish_example_returns_shortest_word() {
    let output = run("distinguish", "examples/rng-order.compare.json");
    assert_eq!(output["result"]["equivalent"], false);
    assert_eq!(
        output["result"]["witness"]["word"],
        serde_json::json!(["cast", "cast"])
    );
}

#[test]
fn factor_example_retains_context_side_information() {
    let output = run("factor", "examples/context-side-information.factor.json");
    assert_eq!(output["operation"], "factor");
    assert_eq!(output["limits"]["max_product_states"], 16);
    assert_eq!(output["result"]["lawful"], true);
    assert_eq!(output["result"]["context_retained"], true);
    assert_eq!(output["result"]["boundary_state_count"], 1);
    assert_eq!(
        output["result"]["unreachable_machine_states"],
        serde_json::json!(["unused"])
    );
    assert_eq!(
        output["result"]["representation"]["states"][0]["transitions"]["c"]["status"],
        "unconstrained"
    );
}

#[test]
fn unlawful_factor_is_a_successful_analysis_with_a_witness() {
    let output = run("factor", "examples/observation-conflict.factor.json");
    assert_eq!(output["status"], "ok");
    assert_eq!(output["result"]["lawful"], false);
    assert!(output["result"]["representation"].is_null());
    assert_eq!(
        output["result"]["witness"]["law"],
        "observation_not_constant"
    );
    assert_eq!(
        output["result"]["witness"]["left"]["access_word"],
        serde_json::json!(["a"])
    );
    assert_eq!(
        output["result"]["witness"]["right"]["access_word"],
        serde_json::json!(["b"])
    );
}

#[test]
fn infeasible_candidate_cannot_buy_frontier_membership() {
    let output = run("pareto", "examples/architecture.candidates.json");
    assert_eq!(
        output["result"]["frontier"],
        serde_json::json!(["borrowed-projection"])
    );
    assert_eq!(
        output["result"]["infeasible"],
        serde_json::json!(["illegal-co-location"])
    );
}

#[test]
fn route_example_reports_exact_weighted_hop_traffic() {
    let output = run("score-routes", "examples/memory.transport.json");
    assert_eq!(
        output["result"]["weighted_hop_traffic"]["value"],
        serde_json::json!({"num": "128000", "den": "1"})
    );
    assert!(output["limits"].is_null());
}

#[test]
fn usage_error_is_typed_json_on_stderr() {
    let output = Command::new(env!("CARGO_BIN_EXE_kerf"))
        .output()
        .expect("kerf must run");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr).expect("stderr must be JSON");
    assert_eq!(error["schema_version"], "kerf.error.v1");
    assert_eq!(error["error"]["code"], "invalid_input");
}

#[test]
fn unknown_command_is_rejected_before_input_io() {
    let output = Command::new(env!("CARGO_BIN_EXE_kerf"))
        .args(["frobnicate", "/definitely/missing/input.json"])
        .output()
        .expect("kerf must run");
    assert_eq!(output.status.code(), Some(2));
    let error: Value = serde_json::from_slice(&output.stderr).expect("stderr must be JSON");
    assert_eq!(error["error"]["code"], "invalid_input");
    assert!(
        error["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("unknown command"))
    );
}

#[cfg(unix)]
#[test]
fn non_utf8_command_is_typed_json_not_a_panic() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let output = Command::new(env!("CARGO_BIN_EXE_kerf"))
        .arg(OsString::from_vec(vec![0xff]))
        .arg(repo_path("examples/toggle.lts.json"))
        .output()
        .expect("kerf must run");
    assert_eq!(output.status.code(), Some(2));
    let error: Value = serde_json::from_slice(&output.stderr).expect("stderr must be JSON");
    assert_eq!(error["error"]["message"], "command must be valid UTF-8");
}

#[test]
fn resource_limit_is_unknown_and_retains_input_context() {
    let mut input: Value = serde_json::from_slice(
        &std::fs::read(repo_path("examples/rng-order.compare.json")).expect("example must read"),
    )
    .expect("example must be JSON");
    input["limits"] = serde_json::json!({
        "max_product_states": 100,
        "max_pairs": 100,
        "max_witness_length": 1
    });
    let path = std::env::temp_dir().join(format!("kerf-limit-{}.json", std::process::id()));
    std::fs::write(
        &path,
        serde_json::to_vec(&input).expect("input must serialize"),
    )
    .expect("temporary input must write");
    let output = Command::new(env!("CARGO_BIN_EXE_kerf"))
        .arg("distinguish")
        .arg(&path)
        .output()
        .expect("kerf must run");
    std::fs::remove_file(&path).expect("temporary input must be removable");
    assert_eq!(output.status.code(), Some(3));
    let error: Value = serde_json::from_slice(&output.stderr).expect("stderr must be JSON");
    assert_eq!(error["status"], "unknown");
    assert_eq!(error["operation"], "distinguish");
    assert_eq!(error["input_sha256"].as_str().map(str::len), Some(64));
    assert_eq!(error["limits"]["max_witness_length"], 1);
}

#[test]
fn factor_product_limit_is_unknown_with_factor_limits() {
    let mut input: Value = serde_json::from_slice(
        &std::fs::read(repo_path("examples/context-side-information.factor.json"))
            .expect("example must read"),
    )
    .expect("example must be JSON");
    input["limits"]["max_product_states"] = serde_json::json!(2);
    let path = std::env::temp_dir().join(format!("kerf-factor-limit-{}.json", std::process::id()));
    std::fs::write(
        &path,
        serde_json::to_vec(&input).expect("input must serialize"),
    )
    .expect("temporary input must write");
    let output = Command::new(env!("CARGO_BIN_EXE_kerf"))
        .arg("factor")
        .arg(&path)
        .output()
        .expect("kerf must run");
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr).expect("stderr must be JSON");
    assert_eq!(error["status"], "unknown");
    assert_eq!(error["operation"], "factor");
    assert_eq!(
        error["limits"],
        serde_json::json!({"max_product_states": 2})
    );

    input["factor_map"][0]["machine_state"] = serde_json::json!("ghost");
    std::fs::write(
        &path,
        serde_json::to_vec(&input).expect("input must serialize"),
    )
    .expect("temporary input must write");
    let invalid = Command::new(env!("CARGO_BIN_EXE_kerf"))
        .arg("factor")
        .arg(&path)
        .output()
        .expect("kerf must run");
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());
    let error: Value = serde_json::from_slice(&invalid.stderr).expect("stderr must be JSON");
    assert_eq!(error["status"], "error");
    assert_eq!(error["error"]["code"], "invalid_input");
    assert!(
        error["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("unknown machine state"))
    );

    input["factor_map"][0]["machine_state"] = serde_json::json!("live");
    input["factor_map"]
        .as_array_mut()
        .expect("factor_map must be an array")
        .remove(0);
    std::fs::write(
        &path,
        serde_json::to_vec(&input).expect("input must serialize"),
    )
    .expect("temporary input must write");
    let missing_initial = Command::new(env!("CARGO_BIN_EXE_kerf"))
        .arg("factor")
        .arg(&path)
        .output()
        .expect("kerf must run");
    std::fs::remove_file(&path).expect("temporary input must be removable");
    assert_eq!(missing_initial.status.code(), Some(2));
    assert!(missing_initial.stdout.is_empty());
    let error: Value =
        serde_json::from_slice(&missing_initial.stderr).expect("stderr must be JSON");
    assert_eq!(error["status"], "error");
    assert_eq!(error["error"]["code"], "invalid_input");
    assert!(
        error["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("missing reachable initial source"))
    );
}
