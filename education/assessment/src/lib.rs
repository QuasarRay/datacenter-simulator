//! Trainer-side evidence validation. A JSON assertion is trusted only when its
//! collector and artifact store are outside the learner's security principal.
use ncp_assessment_kernel::tier_matches;
use ncp_metaverify::{evidence_complete, receipt_current as current_receipt};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Facet {
    pub id: String,
    pub domain: u8,
    pub tier: u8,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Station {
    pub id: u16,
    pub name: String,
    pub facets: Vec<Facet>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub protocol: String,
    pub name: String,
    pub exercise: u16,
    pub attempt: u64,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub facet: String,
    pub tier: u8,
    /// 0: repaired baseline; 1: independent topology/workload holdout.
    pub case: u8,
    pub passed: bool,
    /// SHA-256 of a nonempty raw observation in the trainer artifact store.
    pub artifact: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Intervention {
    /// Exactly one intervention per domain. Domain 0=AIO, 1=AIN, 2=AII.
    pub domain: u8,
    pub failed_when_removed: bool,
    pub recovered_when_restored: bool,
    pub artifact: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub request: Request,
    pub observations: Vec<Observation>,
    pub interventions: Vec<Intervention>,
    pub deepops_revision: String,
    pub deepops_artifact: String,
}
#[derive(Debug, Serialize)]
pub struct Receipt {
    pub protocol: String,
    pub name: String,
    pub attempt: u64,
    pub exercise: u16,
    pub tier: u8,
    pub ops: u8,
    pub net: u8,
    pub infra: u8,
    pub coupled: bool,
    pub provenance: bool,
    pub observed_facets: u64,
    pub feedback: String,
}
pub const DEEPOPS: &str = "dc80499ea34b3f36563ed039421076de15071517";

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub protocol: String,
    pub request: Request,
    pub scope: String,
    pub case: u8,
    pub raw: String,
}
impl Artifact {
    pub fn matches(&self, q: &Request, scope: &str, case: u8) -> bool {
        self.protocol == "ncp-artifact-v1"
            && self.request.protocol == q.protocol
            && self.request.name == q.name
            && self.scope == scope
            && self.case == case
            && !self.raw.trim().is_empty()
            && current_receipt(
                q.attempt,
                self.request.attempt,
                q.exercise,
                self.request.exercise,
            )
    }
}

pub fn accepted(receipt: &Receipt, station: &Station) -> bool {
    if station.facets.is_empty() || station.facets.len() > 63 {
        return false;
    }
    let domains = (receipt.ops == 2) as u8
        | (((receipt.net == 2) as u8) << 1)
        | (((receipt.infra == 2) as u8) << 2);
    // evaluate sets a facet bit only after both baseline and holdout pass.
    // coupled covers all three removal/restoration interventions.
    evidence_complete(
        (1u64 << station.facets.len()) - 1,
        receipt.observed_facets,
        receipt.observed_facets,
        domains,
        if receipt.coupled { 7 } else { 0 },
        receipt.provenance,
    )
}

pub fn validate_bank(bank: &[Station]) -> Result<(), String> {
    if bank.len() != 40 {
        return Err("expected forty stations".into());
    }
    let mut names = BTreeSet::new();
    for (i, s) in bank.iter().enumerate() {
        let expected = format!("e{:02}{}", i / 2 + 1, if i % 2 == 0 { 'a' } else { 'b' });
        if s.id as usize != i
            || s.name != expected
            || !names.insert(&s.name)
            || s.facets.is_empty()
            || s.facets.len() > 63
        {
            return Err("invalid station identity or facet count".into());
        }
        let mut ids = BTreeSet::new();
        let mut domains = [false; 3];
        for f in &s.facets {
            if f.domain > 2 || !tier_matches(f.tier, f.tier) || !ids.insert(&f.id) {
                return Err("invalid facet identity, domain or tier".into());
            }
            domains[f.domain as usize] = true;
        }
        if !domains.into_iter().all(|x| x) {
            return Err("station omits a domain".into());
        }
    }
    Ok(())
}

/// No scoring path accepts learner-provided status files. `artifact_valid` must
/// inspect a trainer-owned content-addressed store, not a submitted filename.
pub fn evaluate(
    s: &Station,
    q: &Request,
    b: Option<&Bundle>,
    artifact_valid: impl Fn(&str, &str, u8) -> bool,
) -> Receipt {
    let mut r = Receipt {
        protocol: "ncp-grade-v1".into(),
        name: q.name.clone(),
        attempt: q.attempt,
        exercise: q.exercise,
        tier: 4,
        ops: 3,
        net: 3,
        infra: 3,
        coupled: false,
        provenance: false,
        observed_facets: 0,
        feedback: "Required live evidence is missing or invalid; mastery remains blocked.".into(),
    };
    let Some(b) = b else {
        return r;
    };
    if q.protocol != "ncp-grade-v1"
        || b.request.protocol != q.protocol
        || q.name != s.name
        || b.request.name != q.name
        || q.exercise != s.id
        || !current_receipt(q.attempt, b.request.attempt, q.exercise, b.request.exercise)
        || s.facets.is_empty()
        || s.facets.len() > 63
        || b.observations.len() != 2 * s.facets.len()
        || b.interventions.len() != 3
    {
        return r;
    }
    r.provenance =
        b.deepops_revision == DEEPOPS && artifact_valid(&b.deepops_artifact, "deepops", 0);
    let mut domains = [true; 3];
    let mut present = [false; 3];
    for (bit, f) in s.facets.iter().enumerate() {
        if f.domain > 2 {
            return r;
        }
        present[f.domain as usize] = true;
        let passed = (0..2).all(|case| {
            let rows: Vec<_> = b
                .observations
                .iter()
                .filter(|o| o.facet == f.id && o.case == case)
                .collect();
            rows.len() == 1
                && rows[0].passed
                && tier_matches(f.tier, rows[0].tier)
                && artifact_valid(&rows[0].artifact, &f.id, case)
        });
        domains[f.domain as usize] &= passed;
        if passed {
            r.observed_facets |= 1u64 << bit;
        }
    }
    r.coupled = (0..3).all(|domain| {
        let rows: Vec<_> = b
            .interventions
            .iter()
            .filter(|i| i.domain == domain)
            .collect();
        rows.len() == 1
            && rows[0].failed_when_removed
            && rows[0].recovered_when_restored
            && artifact_valid(&rows[0].artifact, &format!("intervention/{domain}"), 0)
    });
    let statuses = std::array::from_fn::<_, 3, _>(|i| if present[i] && domains[i] { 2 } else { 1 });
    [r.ops, r.net, r.infra] = statuses;
    if accepted(&r, s) {
        r.feedback = "Integrated station accepted from trainer observations at the required execution tiers.".into();
    } else {
        r.feedback =
            "Integrated station incomplete. Inspect the learner-safe incident report.".into();
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_artifacts_are_bound_to_attempt_scope_and_case() {
        let q = Request {
            protocol: "ncp-grade-v1".into(),
            name: "e01a".into(),
            exercise: 0,
            attempt: 77,
        };
        let mut artifact = Artifact {
            protocol: "ncp-artifact-v1".into(),
            request: q.clone(),
            scope: "AII-1.1/deployment".into(),
            case: 0,
            raw: "device observation".into(),
        };
        assert!(artifact.matches(&q, "AII-1.1/deployment", 0));
        assert!(!artifact.matches(&q, "AII-1.1/validation", 0));
        assert!(!artifact.matches(&q, "AII-1.1/deployment", 1));
        artifact.request.attempt = 76;
        assert!(!artifact.matches(&q, "AII-1.1/deployment", 0));
    }
    fn sample(s: &Station) -> (Request, Bundle) {
        let q = Request {
            protocol: "ncp-grade-v1".into(),
            name: s.name.clone(),
            exercise: s.id,
            attempt: 9,
        };
        let b = Bundle {
            request: q.clone(),
            observations: s
                .facets
                .iter()
                .flat_map(|f| {
                    (0..2).map(move |case| Observation {
                        facet: f.id.clone(),
                        tier: f.tier,
                        case,
                        passed: true,
                        artifact: "test".into(),
                    })
                })
                .collect(),
            interventions: (0..3)
                .map(|domain| Intervention {
                    domain,
                    failed_when_removed: true,
                    recovered_when_restored: true,
                    artifact: "test".into(),
                })
                .collect(),
            deepops_revision: DEEPOPS.into(),
            deepops_artifact: "test".into(),
        };
        (q, b)
    }
    fn accepted(r: Receipt, s: &Station) -> bool {
        super::accepted(&r, s)
    }
    #[test]
    fn entire_bank_rejects_every_missing_facet_domain_case_and_provenance() {
        let bank: Vec<Station> = serde_json::from_str(include_str!("../bank.json")).unwrap();
        validate_bank(&bank).unwrap();
        for s in &bank {
            let (q, b) = sample(s);
            assert!(accepted(evaluate(s, &q, Some(&b), |_, _, _| true), s));
            let mut extra = evaluate(s, &q, Some(&b), |_, _, _| true);
            extra.observed_facets |= 1u64 << 63;
            assert!(!accepted(extra, s));
            assert!(!accepted(evaluate(s, &q, None, |_, _, _| true), s));
            assert!(!accepted(evaluate(s, &q, Some(&b), |_, _, _| false), s));
            for i in 0..b.observations.len() {
                let mut broken = b.clone();
                broken.observations[i].passed = false;
                assert!(!accepted(evaluate(s, &q, Some(&broken), |_, _, _| true), s));
                broken = b.clone();
                broken.observations[i].tier = 1;
                assert!(!accepted(evaluate(s, &q, Some(&broken), |_, _, _| true), s));
                broken = b.clone();
                broken.observations.remove(i);
                assert!(!accepted(evaluate(s, &q, Some(&broken), |_, _, _| true), s));
            }
            for d in 0..3 {
                let mut broken = b.clone();
                broken.interventions[d].failed_when_removed = false;
                assert!(!accepted(evaluate(s, &q, Some(&broken), |_, _, _| true), s));
                broken = b.clone();
                broken.interventions[d].recovered_when_restored = false;
                assert!(!accepted(evaluate(s, &q, Some(&broken), |_, _, _| true), s));
            }
            let mut old = b.clone();
            old.request.attempt = 8;
            assert!(!accepted(evaluate(s, &q, Some(&old), |_, _, _| true), s));
        }
    }
}
