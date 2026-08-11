use kerf::TOOL_VERSION;
use kerf::candidates::{self, CandidateInput};
use kerf::lts::{self, CompareInput, Limits, LtsInput};
use kerf::report::{ToolError, sha256_hex};
use kerf::transport::{self, TransportInput};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

const RESULT_SCHEMA: &str = "kerf.result.v1";

#[derive(Serialize)]
struct ResultEnvelope<T> {
    schema_version: &'static str,
    tool_version: &'static str,
    operation: String,
    input_sha256: String,
    status: &'static str,
    limits: Option<Limits>,
    result: T,
}

#[derive(Debug)]
struct ExecutionError {
    error: ToolError,
    operation: Option<String>,
    input_sha256: Option<String>,
    limits: Option<Limits>,
}

#[derive(Serialize)]
struct ErrorEnvelope<'a> {
    schema_version: &'static str,
    tool_version: &'static str,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    operation: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_sha256: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limits: Option<Limits>,
    error: &'a ToolError,
}

fn main() {
    if let Err(error) = run() {
        let envelope = ErrorEnvelope {
            schema_version: "kerf.error.v1",
            tool_version: TOOL_VERSION,
            status: if error.error.code == "resource_limit" {
                "unknown"
            } else {
                "error"
            },
            operation: error.operation.as_deref(),
            input_sha256: error.input_sha256.as_deref(),
            limits: error.limits,
            error: &error.error,
        };
        let mut stderr = io::stderr().lock();
        serde_json::to_writer(&mut stderr, &envelope).ok();
        writeln!(&mut stderr).ok();
        std::process::exit(match error.error.code {
            "resource_limit" => 3,
            "internal_invariant" => 4,
            _ => 2,
        });
    }
}

fn run() -> Result<(), ExecutionError> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    match args.as_slice() {
        [flag] if flag == OsStr::new("--version") || flag == OsStr::new("-V") => {
            println!("kerf {TOOL_VERSION}");
            Ok(())
        }
        [flag] if flag == OsStr::new("--help") || flag == OsStr::new("-h") => {
            print_help();
            Ok(())
        }
        [command, path] => {
            let command = command
                .to_str()
                .ok_or_else(|| bare_error(ToolError::invalid("command must be valid UTF-8")))?;
            if !is_command(command) {
                return Err(ExecutionError {
                    error: ToolError::invalid(format!(
                        "unknown command {command:?}; run kerf --help"
                    )),
                    operation: Some(command.to_owned()),
                    input_sha256: None,
                    limits: None,
                });
            }
            dispatch(command, Path::new(path))
        }
        _ => Err(bare_error(ToolError::invalid(
            "usage: kerf <validate-lts|quotient|distinguish|validate-candidates|pareto|score-routes> <input.json>",
        ))),
    }
}

fn is_command(command: &str) -> bool {
    matches!(
        command,
        "validate-lts"
            | "quotient"
            | "distinguish"
            | "validate-candidates"
            | "pareto"
            | "score-routes"
    )
}

fn dispatch(command: &str, path: &Path) -> Result<(), ExecutionError> {
    let bytes = fs::read(path).map_err(|error| ExecutionError {
        error: ToolError::io(format!("could not read {}: {error}", path.display())),
        operation: Some(command.to_owned()),
        input_sha256: None,
        limits: None,
    })?;
    let input_sha256 = sha256_hex(&bytes);
    match command {
        "validate-lts" => {
            let input: LtsInput =
                parse(&bytes).map_err(|error| contextual(error, command, &input_sha256, None))?;
            let limits = Some(input.limits);
            let result = lts::validate(&input)
                .map_err(|error| contextual(error, command, &input_sha256, limits))?;
            emit(command, input_sha256.clone(), limits, result)
                .map_err(|error| contextual(error, command, &input_sha256, limits))
        }
        "quotient" => {
            let input: LtsInput =
                parse(&bytes).map_err(|error| contextual(error, command, &input_sha256, None))?;
            let limits = Some(input.limits);
            let result = lts::quotient(&input)
                .map_err(|error| contextual(error, command, &input_sha256, limits))?;
            emit(command, input_sha256.clone(), limits, result)
                .map_err(|error| contextual(error, command, &input_sha256, limits))
        }
        "distinguish" => {
            let input: CompareInput =
                parse(&bytes).map_err(|error| contextual(error, command, &input_sha256, None))?;
            let limits = Some(input.limits);
            let result = lts::compare(&input)
                .map_err(|error| contextual(error, command, &input_sha256, limits))?;
            emit(command, input_sha256.clone(), limits, result)
                .map_err(|error| contextual(error, command, &input_sha256, limits))
        }
        "validate-candidates" => {
            let input: CandidateInput =
                parse(&bytes).map_err(|error| contextual(error, command, &input_sha256, None))?;
            let result = candidates::validate(&input)
                .map_err(|error| contextual(error, command, &input_sha256, None))?;
            emit(command, input_sha256.clone(), None, result)
                .map_err(|error| contextual(error, command, &input_sha256, None))
        }
        "pareto" => {
            let input: CandidateInput =
                parse(&bytes).map_err(|error| contextual(error, command, &input_sha256, None))?;
            let result = candidates::pareto(&input)
                .map_err(|error| contextual(error, command, &input_sha256, None))?;
            emit(command, input_sha256.clone(), None, result)
                .map_err(|error| contextual(error, command, &input_sha256, None))
        }
        "score-routes" => {
            let input: TransportInput =
                parse(&bytes).map_err(|error| contextual(error, command, &input_sha256, None))?;
            let result = transport::score(&input)
                .map_err(|error| contextual(error, command, &input_sha256, None))?;
            emit(command, input_sha256.clone(), None, result)
                .map_err(|error| contextual(error, command, &input_sha256, None))
        }
        _ => unreachable!("command validated before input is read"),
    }
}

fn bare_error(error: ToolError) -> ExecutionError {
    ExecutionError {
        error,
        operation: None,
        input_sha256: None,
        limits: None,
    }
}

fn contextual(
    error: ToolError,
    operation: &str,
    input_sha256: &str,
    limits: Option<Limits>,
) -> ExecutionError {
    ExecutionError {
        error,
        operation: Some(operation.to_owned()),
        input_sha256: Some(input_sha256.to_owned()),
        limits,
    }
}

fn parse<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, ToolError> {
    serde_json::from_slice(bytes)
        .map_err(|error| ToolError::invalid(format!("invalid JSON input: {error}")))
}

fn emit<T: Serialize>(
    operation: &str,
    input_sha256: String,
    limits: Option<Limits>,
    result: T,
) -> Result<(), ToolError> {
    let envelope = ResultEnvelope {
        schema_version: RESULT_SCHEMA,
        tool_version: TOOL_VERSION,
        operation: operation.to_owned(),
        input_sha256,
        status: "ok",
        limits,
        result,
    };
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, &envelope)
        .map_err(|error| ToolError::internal(format!("could not serialize result: {error}")))?;
    writeln!(&mut stdout)
        .map_err(|error| ToolError::io(format!("could not write result: {error}")))?;
    Ok(())
}

fn print_help() {
    println!(
        "kerf {TOOL_VERSION}\n\
Exact finite behavioral-factorization checks.\n\n\
USAGE:\n  kerf COMMAND INPUT.json\n\n\
COMMANDS:\n\
  validate-lts          Validate a kerf.lts.v1 model\n\
  quotient              Compute its reachable observational quotient\n\
  distinguish           Compare two kerf.compare.v1 machines and emit a shortest witness\n\
  validate-candidates   Check explicit assignment and authority constraints\n\
  pareto                Compute the exact frontier of the supplied candidate set\n\
  score-routes          Account exact load on supplied static routes\n\n\
All analysis results are versioned JSON on stdout. Errors are typed JSON on stderr."
    );
}
