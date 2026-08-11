use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

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

fn run_value(command: &str, input: &Value) -> Output {
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let path =
        std::env::temp_dir().join(format!("kerf-cli-{}-{sequence}.json", std::process::id()));
    std::fs::write(
        &path,
        serde_json::to_vec(input).expect("input must serialize"),
    )
    .expect("temporary input must write");
    let output = Command::new(env!("CARGO_BIN_EXE_kerf"))
        .arg(command)
        .arg(&path)
        .output()
        .expect("kerf must run");
    std::fs::remove_file(path).expect("temporary input must be removable");
    output
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
    assert_eq!(
        output["result"]["representation"]["schema_version"],
        "kerf.factor-representation.v2"
    );
    assert_eq!(
        output["result"]["representation"]["certified_context"]["initial"],
        "branch"
    );
}

#[test]
fn one_factor_recomposition_is_exact_under_its_certified_context() {
    let output = run(
        "recompose",
        "examples/context-side-information.recompose.json",
    );
    assert_eq!(output["operation"], "recompose");
    assert_eq!(output["result"]["verdict"], "equivalent");
    assert_eq!(output["result"]["explored_composed_states"], 3);
    assert_eq!(output["limits"]["max_pairs"], 16);
}

#[test]
fn widened_recomposition_is_successfully_unproved() {
    let mut input: Value = serde_json::from_slice(
        &std::fs::read(repo_path(
            "examples/context-side-information.recompose.json",
        ))
        .expect("example must read"),
    )
    .expect("example must be JSON");
    input["context"] = Value::Null;
    let path = std::env::temp_dir().join(format!(
        "kerf-recompose-unproved-{}.json",
        std::process::id()
    ));
    std::fs::write(
        &path,
        serde_json::to_vec(&input).expect("input must serialize"),
    )
    .expect("temporary input must write");
    let output = Command::new(env!("CARGO_BIN_EXE_kerf"))
        .arg("recompose")
        .arg(&path)
        .output()
        .expect("kerf must run");
    std::fs::remove_file(&path).expect("temporary input must be removable");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let result: Value = serde_json::from_slice(&output.stdout).expect("stdout must be JSON");
    assert_eq!(result["result"]["verdict"], "unproved");
    assert_eq!(
        result["result"]["unproved"]["word"],
        serde_json::json!(["c"])
    );
}

#[test]
fn recomposition_product_limit_is_unknown() {
    let mut input: Value = serde_json::from_slice(
        &std::fs::read(repo_path("examples/two-factor.recompose.json")).expect("example must read"),
    )
    .expect("example must be JSON");
    input["limits"]["max_product_states"] = serde_json::json!(1);
    let path =
        std::env::temp_dir().join(format!("kerf-recompose-limit-{}.json", std::process::id()));
    std::fs::write(
        &path,
        serde_json::to_vec(&input).expect("input must serialize"),
    )
    .expect("temporary input must write");
    let output = Command::new(env!("CARGO_BIN_EXE_kerf"))
        .arg("recompose")
        .arg(&path)
        .output()
        .expect("kerf must run");
    std::fs::remove_file(&path).expect("temporary input must be removable");
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr).expect("stderr must be JSON");
    assert_eq!(error["status"], "unknown");
    assert_eq!(error["operation"], "recompose");
    assert_eq!(error["limits"]["max_product_states"], 1);
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
fn certificate_examples_cover_relation_witness_and_trace() {
    let cases = [
        (
            "examples/simple-equivalence.cert.json",
            "compare_equivalence_relation",
        ),
        (
            "examples/rng-order.witness.cert.json",
            "compare_inequivalence_witness",
        ),
        ("examples/toggle.trace.cert.json", "lts_trace"),
    ];
    for (path, kind) in cases {
        let output = run("verify-certificate", path);
        assert_eq!(output["operation"], "verify-certificate");
        assert_eq!(output["result"]["kind"], kind);
        assert_eq!(output["result"]["accepted"], true);
        assert!(output["limits"].is_null());
    }
}

#[test]
fn false_certificate_is_a_completed_analysis_but_invalid_subject_is_an_error() {
    let mut input: Value = serde_json::from_slice(
        &std::fs::read(repo_path("examples/simple-equivalence.cert.json"))
            .expect("example must read"),
    )
    .expect("example must be JSON");
    input["claim"]["relation"] = serde_json::json!([]);
    let rejected = run_value("verify-certificate", &input);
    assert!(rejected.status.success());
    assert!(rejected.stderr.is_empty());
    let result: Value = serde_json::from_slice(&rejected.stdout).expect("stdout must be JSON");
    assert_eq!(result["result"]["accepted"], false);
    assert_eq!(
        result["result"]["rejection"]["code"],
        "certificate_malformed"
    );

    input["claim"]["subject"]["left"]["actions"] = serde_json::json!([]);
    let invalid = run_value("verify-certificate", &input);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());
    let error: Value = serde_json::from_slice(&invalid.stderr).expect("stderr must be JSON");
    assert_eq!(error["operation"], "verify-certificate");
    assert_eq!(error["error"]["code"], "invalid_input");
}

#[cfg(target_os = "linux")]
#[test]
fn unwritable_stdout_is_a_typed_io_error() {
    let output = Command::new("sh")
        .args([
            "-c",
            "exec 1>/dev/full; exec \"$1\" verify-certificate \"$2\"",
            "sh",
        ])
        .arg(env!("CARGO_BIN_EXE_kerf"))
        .arg(repo_path("examples/simple-equivalence.cert.json"))
        .output()
        .expect("shell must run kerf with stdout closed");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr).expect("stderr must be JSON");
    assert_eq!(error["error"]["code"], "io_error");
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
