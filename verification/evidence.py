"""Exact named checklists shared by qualification and verification adapters.

A checklist cannot establish that a caller actually performed a check. Its job
is to reject incomplete, repeated, unknown, false or stale declared outcomes.
"""
from verification.dsl import require
from verification.materials import unique
from verification.specification import report_complete


class Checklist:
    def __init__(self, required, identity):
        names = list(required)
        unique(names, nonempty=True)
        require(len(names) <= 64 and type(identity) is str and bool(identity), "bounded checklist and identity required")
        self.required = tuple(sorted(names))
        self.identity = identity
        self._records = {}

    def record(self, name, passed=True):
        require(name in self.required and name not in self._records, "unknown or duplicate check: " + str(name))
        require(type(passed) is bool, "check outcome must be a boolean")
        self._records[name] = passed

    @property
    def complete(self):
        passed = sum(1 << i for i, name in enumerate(self.required) if self._records.get(name) is True)
        failed = sum(1 << i for i, name in enumerate(self.required) if self._records.get(name) is False)
        return report_complete.call((1 << len(self.required)) - 1, passed, failed)

    def report(self):
        return {"schema": "ncp-checklist-v1", "identity": self.identity, "required": list(self.required),
                "records": [{"name": name, "passed": self._records[name]} for name in self.required if name in self._records],
                "complete": self.complete}

    def finish(self):
        require(self.complete, "checklist incomplete or failed: " + ", ".join(name for name in self.required if self._records.get(name) is not True))
        return self.report()

    @classmethod
    def validate(cls, report, required, identity):
        require(type(report) is dict and set(report) == {"schema", "identity", "required", "records", "complete"}, "checklist report schema")
        ledger = cls(required, identity)
        require(report["schema"] == "ncp-checklist-v1" and report["identity"] == identity
                and report["required"] == list(ledger.required), "stale identity or altered checklist")
        require(type(report["records"]) is list and len(report["records"]) <= len(ledger.required), "checklist record bound")
        for row in report["records"]:
            require(type(row) is dict and set(row) == {"name", "passed"}, "checklist row schema")
            ledger.record(row["name"], row["passed"])
        require(type(report["complete"]) is bool and report["complete"] == ledger.complete, "claimed completion contradicts records")
        ledger.finish()
        return ledger
