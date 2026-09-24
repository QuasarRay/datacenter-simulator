"""Reviewed laws. Builders expand once into execution and both proof backends.

Witnesses rule out vacuity on concrete examples; all-input proofs establish the
postconditions. Neither mechanism proves that these laws fully describe teaching.
"""
from verification.dsl import Registry

registry = Registry()
predicate = registry.predicate


@predicate(parameters={"required": "u64", "available": "u64"},
           witnesses=[((0, 0), True), ((5, 7), True), ((5, 3), False)],
           scope="Every prerequisite bit is available; an empty chunk is allowed.")
def subset(required, available, result):
    return (required & available).eq(required), [result.eq(~(required & available).ne(required))]


@predicate(parameters={"required": "u64", "observed": "u64"},
           witnesses=[((5, 7), True), ((5, 3), False), ((0, 0), False)],
           scope="Nonempty source-facet coverage; absent facets cannot be accepted.")
def coverage(required, observed, result):
    return required.ne(0) & (required & observed).eq(required), [
        (~result).eq(required.eq(0) | (required & observed).ne(required))]


@predicate(parameters={"required": "u64", "taught": "u64", "practiced": "u64", "exercise": "bool"},
           witnesses=[((3, 7, 3, True), True), ((3, 7, 1, True), False), ((3, 7, 0, False), True)],
           scope="Projects require teaching; exercises additionally require prior project practice, per 64-bit chunk.")
def learning_ready(required, taught, practiced, exercise, result):
    return (required & taught).eq(required) & ((~exercise) | (required & practiced).eq(required)), [
        (~result).eq((required & taught).ne(required) | (exercise & (required & practiced).ne(required)))]


@predicate(parameters={"parent": "u16", "child": "u16", "total": "u16"},
           witnesses=[((0, 1, 2), True), ((1, 0, 2), False), ((1, 1, 2), False), ((0, 2, 2), False)],
           scope="Every prerequisite precedes its consumer in the finite learning graph.")
def ordered_edge(parent, child, total, result):
    return parent.lt(child) & child.lt(total), [result.eq(~(child.le(parent) | total.le(child)))]


@predicate(parameters={"current": "u16", "requested": "u16", "total": "u16"},
           witnesses=[((0, 0, 40), True), ((0, 1, 40), False), ((40, 40, 40), False)],
           scope="Only the current nonterminal sequential stage is eligible.")
def next_stage(current, requested, total, result):
    return current.eq(requested) & current.lt(total), [
        (~result).eq(current.ne(requested) | total.le(current))]


@predicate(parameters={"required": "u64", "baseline": "u64", "holdout": "u64", "domains": "u8", "recovery": "u8", "provenance": "bool"},
           witnesses=[((7, 7, 7, 7, 7, True), True), ((7, 7, 3, 7, 7, True), False),
                      ((7, 7, 7, 3, 7, True), False), ((7, 7, 7, 7, 7, False), False)],
           scope="Exact baseline/holdout facet sets, all three causal domains, recovery and provenance are indivisible.")
def evidence_complete(required, baseline, holdout, domains, recovery, provenance, result):
    body = required.ne(0) & baseline.eq(required) & holdout.eq(required) & domains.eq(7) & recovery.eq(7) & provenance
    invalid = required.eq(0) | baseline.ne(required) | holdout.ne(required) | domains.ne(7) | recovery.ne(7) | (~provenance)
    return body, [(~result).eq(invalid)]


@predicate(parameters={"required": "u64", "passed": "u64", "failed": "u64"},
           witnesses=[((7, 7, 0), True), ((7, 3, 0), False), ((7, 7, 1), False), ((0, 0, 0), False)],
           scope="A proof/report succeeds only with the exact nonempty checklist and no failures.")
def report_complete(required, passed, failed, result):
    return required.ne(0) & passed.eq(required) & failed.eq(0), [
        (~result).eq(required.eq(0) | passed.ne(required) | failed.ne(0))]


@predicate(parameters={"attempt": "u64", "receipt": "u64", "exercise": "u16", "observed": "u16"},
           witnesses=[((1, 1, 0, 0), True), ((0, 0, 0, 0), False), ((2, 1, 0, 0), False)],
           scope="An observation belongs to exactly one nonzero attempt and exercise.")
def receipt_current(attempt, receipt, exercise, observed, result):
    return attempt.ne(0) & attempt.eq(receipt) & exercise.eq(observed), [
        (~result).eq(attempt.eq(0) | attempt.ne(receipt) | exercise.ne(observed))]
