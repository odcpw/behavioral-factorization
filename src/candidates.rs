use crate::report::ToolError;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const CANDIDATE_SCHEMA: &str = "kerf.candidates.v1";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateInput {
    pub schema_version: String,
    pub elements: Vec<String>,
    pub modules: Vec<String>,
    #[serde(default)]
    pub constraints: Constraints,
    pub metric_definitions: Vec<MetricDefinition>,
    pub candidates: Vec<Candidate>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Constraints {
    pub co_locate: Vec<[String; 2]>,
    pub separate: Vec<[String; 2]>,
    pub pinned: BTreeMap<String, String>,
    pub authority_groups: Vec<AuthorityGroup>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityGroup {
    pub id: String,
    pub members: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricDefinition {
    pub id: String,
    pub unit: String,
    pub direction: Direction,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Minimize,
    Maximize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub id: String,
    pub assignment: BTreeMap<String, String>,
    #[serde(default)]
    pub authority: BTreeMap<String, String>,
    pub metrics: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct CandidateValidationResult {
    pub algorithm: &'static str,
    pub candidates: Vec<CandidateStatus>,
    pub feasible_count: usize,
    pub nonclaims: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct ParetoResult {
    pub algorithm: &'static str,
    pub supplied_set_enumeration: SuppliedSetCertificate,
    pub frontier: Vec<String>,
    pub dominated: Vec<DominatedCandidate>,
    pub infeasible: Vec<String>,
    pub metric_definitions: Vec<MetricDefinitionOutput>,
    pub nonclaims: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct SuppliedSetCertificate {
    pub supplied_candidate_count: usize,
    pub feasible_candidate_count: usize,
    pub statement: &'static str,
}

#[derive(Debug, Serialize)]
pub struct MetricDefinitionOutput {
    pub id: String,
    pub unit: String,
    pub direction: Direction,
}

#[derive(Debug, Serialize)]
pub struct CandidateStatus {
    pub id: String,
    pub feasible: bool,
    pub violations: Vec<ConstraintViolation>,
}

#[derive(Debug, Serialize)]
pub struct ConstraintViolation {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DominatedCandidate {
    pub id: String,
    pub dominated_by: Vec<String>,
}

struct Validated<'a> {
    metric_definitions: Vec<(&'a MetricDefinition, Direction)>,
    candidates: Vec<ValidatedCandidate<'a>>,
}

struct ValidatedCandidate<'a> {
    candidate: &'a Candidate,
    metrics: Vec<u128>,
    violations: Vec<ConstraintViolation>,
}

/// Validate the candidate schema and its executable hard constraints.
///
/// # Errors
///
/// Returns [`ToolError`] when IDs, references, assignments, authority selections,
/// metrics, or schema versions are invalid.
pub fn validate(input: &CandidateInput) -> Result<CandidateValidationResult, ToolError> {
    let validated = validate_all(input)?;
    let feasible_count = validated
        .candidates
        .iter()
        .filter(|candidate| candidate.violations.is_empty())
        .count();
    Ok(CandidateValidationResult {
        algorithm: "explicit_constraint_validation_v1",
        candidates: validated
            .candidates
            .into_iter()
            .map(|candidate| CandidateStatus {
                id: candidate.candidate.id.clone(),
                feasible: candidate.violations.is_empty(),
                violations: candidate.violations,
            })
            .collect(),
        feasible_count,
        nonclaims: candidate_nonclaims(),
    })
}

/// Compute the exact Pareto frontier of the feasible supplied candidate set.
///
/// # Errors
///
/// Returns [`ToolError`] when the candidate model is invalid.
pub fn pareto(input: &CandidateInput) -> Result<ParetoResult, ToolError> {
    let validated = validate_all(input)?;
    let feasible: Vec<&ValidatedCandidate<'_>> = validated
        .candidates
        .iter()
        .filter(|candidate| candidate.violations.is_empty())
        .collect();
    let mut frontier = Vec::new();
    let mut dominated = Vec::new();
    for candidate in &feasible {
        let mut dominators: Vec<String> = feasible
            .iter()
            .filter(|other| {
                other.candidate.id != candidate.candidate.id
                    && dominates(other, candidate, &validated.metric_definitions)
            })
            .map(|other| other.candidate.id.clone())
            .collect();
        dominators.sort();
        if dominators.is_empty() {
            frontier.push(candidate.candidate.id.clone());
        } else {
            dominated.push(DominatedCandidate {
                id: candidate.candidate.id.clone(),
                dominated_by: dominators,
            });
        }
    }
    frontier.sort();
    dominated.sort_by(|left, right| left.id.cmp(&right.id));
    let mut infeasible: Vec<String> = validated
        .candidates
        .iter()
        .filter(|candidate| !candidate.violations.is_empty())
        .map(|candidate| candidate.candidate.id.clone())
        .collect();
    infeasible.sort();
    Ok(ParetoResult {
        algorithm: "exact_product_order_v1",
        supplied_set_enumeration: SuppliedSetCertificate {
            supplied_candidate_count: validated.candidates.len(),
            feasible_candidate_count: feasible.len(),
            statement: "All supplied candidates were validated and all represented-feasible candidates were pairwise compared; no candidate-space exhaustiveness is claimed.",
        },
        frontier,
        dominated,
        infeasible,
        metric_definitions: validated
            .metric_definitions
            .iter()
            .map(|(definition, direction)| MetricDefinitionOutput {
                id: definition.id.clone(),
                unit: definition.unit.clone(),
                direction: *direction,
            })
            .collect(),
        nonclaims: candidate_nonclaims(),
    })
}

#[allow(clippy::too_many_lines)]
fn validate_all(input: &CandidateInput) -> Result<Validated<'_>, ToolError> {
    if input.schema_version != CANDIDATE_SCHEMA {
        return Err(ToolError::invalid(format!(
            "unsupported schema_version {:?}; expected {:?}",
            input.schema_version, CANDIDATE_SCHEMA
        )));
    }
    let elements = unique_ids("elements", &input.elements)?;
    let modules = unique_ids("modules", &input.modules)?;
    if input.candidates.is_empty() {
        return Err(ToolError::invalid("candidates must not be empty"));
    }
    let mut candidate_ids = BTreeSet::new();
    for candidate in &input.candidates {
        require_id("candidate", &candidate.id)?;
        if !candidate_ids.insert(candidate.id.as_str()) {
            return Err(ToolError::invalid(format!(
                "duplicate candidate ID {:?}",
                candidate.id
            )));
        }
    }

    validate_constraints(&input.constraints, &elements, &modules)?;
    let metric_definitions = validate_metric_definitions(&input.metric_definitions)?;
    let expected_metrics: BTreeSet<&str> = input
        .metric_definitions
        .iter()
        .map(|definition| definition.id.as_str())
        .collect();
    let authority_groups: BTreeMap<&str, &AuthorityGroup> = input
        .constraints
        .authority_groups
        .iter()
        .map(|group| (group.id.as_str(), group))
        .collect();
    let expected_authority: BTreeSet<&str> = authority_groups.keys().copied().collect();

    let mut candidates = Vec::with_capacity(input.candidates.len());
    for candidate in &input.candidates {
        let assignment_keys: BTreeSet<&str> =
            candidate.assignment.keys().map(String::as_str).collect();
        if assignment_keys != elements {
            return Err(ToolError::invalid(format!(
                "candidate {:?} assignment keys must equal the declared element set",
                candidate.id
            )));
        }
        for (element, module) in &candidate.assignment {
            if !modules.contains(module.as_str()) {
                return Err(ToolError::invalid(format!(
                    "candidate {:?} assigns element {:?} to unknown module {:?}",
                    candidate.id, element, module
                )));
            }
        }
        let authority_keys: BTreeSet<&str> =
            candidate.authority.keys().map(String::as_str).collect();
        if authority_keys != expected_authority {
            return Err(ToolError::invalid(format!(
                "candidate {:?} authority keys must equal the declared authority-group set",
                candidate.id
            )));
        }
        let metric_keys: BTreeSet<&str> = candidate.metrics.keys().map(String::as_str).collect();
        if metric_keys != expected_metrics {
            return Err(ToolError::invalid(format!(
                "candidate {:?} metric keys must equal the declared metric-definition set",
                candidate.id
            )));
        }
        let metrics = input
            .metric_definitions
            .iter()
            .map(|definition| parse_u128(&candidate.metrics[&definition.id], &definition.id))
            .collect::<Result<Vec<_>, _>>()?;

        let mut violations = Vec::new();
        for [left, right] in &input.constraints.co_locate {
            if candidate.assignment[left] != candidate.assignment[right] {
                violations.push(ConstraintViolation {
                    code: "co_location",
                    message: format!("elements {left:?} and {right:?} must be co-located"),
                });
            }
        }
        for [left, right] in &input.constraints.separate {
            if candidate.assignment[left] == candidate.assignment[right] {
                violations.push(ConstraintViolation {
                    code: "separation",
                    message: format!("elements {left:?} and {right:?} must be separated"),
                });
            }
        }
        for (element, module) in &input.constraints.pinned {
            if candidate.assignment[element] != *module {
                violations.push(ConstraintViolation {
                    code: "pin",
                    message: format!("element {element:?} must be assigned to module {module:?}"),
                });
            }
        }
        for (group_id, representative) in &candidate.authority {
            let group = authority_groups[group_id.as_str()];
            if !group.members.contains(representative) {
                violations.push(ConstraintViolation {
                    code: "authority",
                    message: format!(
                        "authority representative {representative:?} is not a member of group {group_id:?}"
                    ),
                });
            }
        }
        candidates.push(ValidatedCandidate {
            candidate,
            metrics,
            violations,
        });
    }

    Ok(Validated {
        metric_definitions,
        candidates,
    })
}

fn validate_constraints(
    constraints: &Constraints,
    elements: &BTreeSet<&str>,
    modules: &BTreeSet<&str>,
) -> Result<(), ToolError> {
    for (kind, pairs) in [
        ("co_locate", &constraints.co_locate),
        ("separate", &constraints.separate),
    ] {
        for pair in pairs {
            for element in pair {
                if !elements.contains(element.as_str()) {
                    return Err(ToolError::invalid(format!(
                        "{kind} references unknown element {element:?}"
                    )));
                }
            }
            if pair[0] == pair[1] {
                return Err(ToolError::invalid(format!(
                    "{kind} pair must contain two distinct elements"
                )));
            }
        }
    }
    for (element, module) in &constraints.pinned {
        if !elements.contains(element.as_str()) || !modules.contains(module.as_str()) {
            return Err(ToolError::invalid(format!(
                "pin {element:?} -> {module:?} references an unknown element or module"
            )));
        }
    }
    let mut group_ids = BTreeSet::new();
    for group in &constraints.authority_groups {
        require_id("authority group", &group.id)?;
        if !group_ids.insert(group.id.as_str()) {
            return Err(ToolError::invalid(format!(
                "duplicate authority-group ID {:?}",
                group.id
            )));
        }
        if group.members.is_empty() {
            return Err(ToolError::invalid(format!(
                "authority group {:?} must have at least one member",
                group.id
            )));
        }
        let mut members = BTreeSet::new();
        for member in &group.members {
            if !elements.contains(member.as_str()) {
                return Err(ToolError::invalid(format!(
                    "authority group {:?} references unknown element {:?}",
                    group.id, member
                )));
            }
            if !members.insert(member.as_str()) {
                return Err(ToolError::invalid(format!(
                    "authority group {:?} contains duplicate member {:?}",
                    group.id, member
                )));
            }
        }
    }
    Ok(())
}

fn validate_metric_definitions(
    definitions: &[MetricDefinition],
) -> Result<Vec<(&MetricDefinition, Direction)>, ToolError> {
    if definitions.is_empty() {
        return Err(ToolError::invalid("metric_definitions must not be empty"));
    }
    let mut ids = BTreeSet::new();
    let mut result = Vec::with_capacity(definitions.len());
    for definition in definitions {
        require_id("metric", &definition.id)?;
        if definition.unit.is_empty() {
            return Err(ToolError::invalid(format!(
                "metric {:?} must declare a non-empty unit",
                definition.id
            )));
        }
        if !ids.insert(definition.id.as_str()) {
            return Err(ToolError::invalid(format!(
                "duplicate metric ID {:?}",
                definition.id
            )));
        }
        result.push((definition, definition.direction));
    }
    Ok(result)
}

fn unique_ids<'a>(kind: &str, values: &'a [String]) -> Result<BTreeSet<&'a str>, ToolError> {
    if values.is_empty() {
        return Err(ToolError::invalid(format!("{kind} must not be empty")));
    }
    let mut result = BTreeSet::new();
    for value in values {
        require_id(kind, value)?;
        if !result.insert(value.as_str()) {
            return Err(ToolError::invalid(format!("duplicate {kind} ID {value:?}")));
        }
    }
    Ok(result)
}

fn require_id(kind: &str, value: &str) -> Result<(), ToolError> {
    if value.is_empty() {
        Err(ToolError::invalid(format!("{kind} IDs must not be empty")))
    } else {
        Ok(())
    }
}

fn parse_u128(value: &str, metric: &str) -> Result<u128, ToolError> {
    value.parse::<u128>().map_err(|_| {
        ToolError::invalid(format!(
            "metric {metric:?} value {value:?} must be an unsigned base-10 integer string"
        ))
    })
}

fn dominates(
    left: &ValidatedCandidate<'_>,
    right: &ValidatedCandidate<'_>,
    definitions: &[(&MetricDefinition, Direction)],
) -> bool {
    let mut strictly_better = false;
    for (index, (_, direction)) in definitions.iter().enumerate() {
        let ordering = left.metrics[index].cmp(&right.metrics[index]);
        let no_worse = match direction {
            Direction::Minimize => !ordering.is_gt(),
            Direction::Maximize => !ordering.is_lt(),
        };
        if !no_worse {
            return false;
        }
        if !ordering.is_eq() {
            strictly_better = true;
        }
    }
    strictly_better
}

fn candidate_nonclaims() -> Vec<&'static str> {
    vec![
        "Feasibility covers only the executable constraints represented in this input.",
        "Authority selections and metric values are supplied assertions; the tool validates their shape and references, not their truth in an implementation.",
        "The tool does not accept a user-declared semantic-equivalence boolean as proof.",
        "The frontier is exact only within the supplied candidate set and metric definitions.",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> CandidateInput {
        CandidateInput {
            schema_version: CANDIDATE_SCHEMA.to_owned(),
            elements: vec!["world".to_owned(), "view".to_owned()],
            modules: vec!["core".to_owned(), "ui".to_owned()],
            constraints: Constraints {
                pinned: BTreeMap::from([("world".to_owned(), "core".to_owned())]),
                authority_groups: vec![AuthorityGroup {
                    id: "world_state".to_owned(),
                    members: vec!["world".to_owned()],
                }],
                ..Constraints::default()
            },
            metric_definitions: vec![
                MetricDefinition {
                    id: "boundary_bits".to_owned(),
                    unit: "bits_per_step".to_owned(),
                    direction: Direction::Minimize,
                },
                MetricDefinition {
                    id: "headroom".to_owned(),
                    unit: "bytes_per_second".to_owned(),
                    direction: Direction::Maximize,
                },
            ],
            candidates: vec![
                Candidate {
                    id: "tight".to_owned(),
                    assignment: BTreeMap::from([
                        ("world".to_owned(), "core".to_owned()),
                        ("view".to_owned(), "ui".to_owned()),
                    ]),
                    authority: BTreeMap::from([("world_state".to_owned(), "world".to_owned())]),
                    metrics: BTreeMap::from([
                        ("boundary_bits".to_owned(), "10".to_owned()),
                        ("headroom".to_owned(), "100".to_owned()),
                    ]),
                },
                Candidate {
                    id: "loose".to_owned(),
                    assignment: BTreeMap::from([
                        ("world".to_owned(), "core".to_owned()),
                        ("view".to_owned(), "ui".to_owned()),
                    ]),
                    authority: BTreeMap::from([("world_state".to_owned(), "world".to_owned())]),
                    metrics: BTreeMap::from([
                        ("boundary_bits".to_owned(), "20".to_owned()),
                        ("headroom".to_owned(), "50".to_owned()),
                    ]),
                },
            ],
        }
    }

    #[test]
    fn pareto_reports_exact_dominator() {
        let result = pareto(&input()).expect("valid candidates");
        assert_eq!(result.frontier, vec!["tight"]);
        assert_eq!(result.dominated[0].id, "loose");
        assert_eq!(result.dominated[0].dominated_by, vec!["tight"]);
    }

    #[test]
    fn pin_violation_is_infeasible_not_a_metric_penalty() {
        let mut input = input();
        input.candidates[0]
            .assignment
            .insert("world".to_owned(), "ui".to_owned());
        let result = validate(&input).expect("valid schema");
        assert!(!result.candidates[0].feasible);
        assert_eq!(result.candidates[0].violations[0].code, "pin");
    }

    #[test]
    fn missing_metric_is_invalid_input() {
        let mut input = input();
        input.candidates[0].metrics.remove("headroom");
        let error = validate(&input).expect_err("missing metric must not become zero");
        assert_eq!(error.code, "invalid_input");
    }

    #[test]
    fn equal_vectors_both_remain_nondominated() {
        let mut input = input();
        input.candidates[1].metrics = input.candidates[0].metrics.clone();
        let result = pareto(&input).expect("valid candidates");
        assert_eq!(result.frontier, vec!["loose", "tight"]);
        assert!(result.dominated.is_empty());
    }
}
