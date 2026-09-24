"""Finite curriculum refinement over reusable, formally checked predicates.

The graph checks declared prerequisites and skills, not the truth of prose or
physical observations. Each numeric obligation is also emitted for Rust execution.
"""
from dataclasses import dataclass
import json
import re
from verification.dsl import require
from verification.specification import registry


def record(value, keys, optional=()):
    require(type(value) is dict, "expected an object")
    require(set(keys) <= set(value) <= set(keys) | set(optional), "missing or unexpected object fields")
    return value


def sequence(value, limit=2048):
    require(type(value) is list and len(value) <= limit, "expected a bounded list")
    return value


def text(value):
    require(type(value) is str and 0 < len(value) <= 200_000 and "\x00" not in value, "expected nonempty text")
    return value


def unique(value, *, nonempty=False):
    sequence(value)
    require(all(type(v) is str and v for v in value), "expected string identities")
    require(len(value) == len(set(value)) and (bool(value) or not nonempty), "empty or duplicate identity")
    return set(value)


@dataclass(frozen=True)
class Fact:
    name: str
    contract: str
    arguments: tuple


class Obligations:
    def __init__(self): self.facts = []

    def check(self, name, contract, *arguments):
        require(re.fullmatch(r"[A-Za-z0-9_./:-]+", name) is not None, "invalid obligation name")
        predicate = registry.contracts[contract]
        require(predicate.call(*arguments), f"{name}: {contract} rejected {arguments}")
        self.facts.append(Fact(name, contract, arguments))

    def sets(self, name, contract, required, *available, flags=()):
        universe = sorted(set(required).union(*map(set, available)))
        require(len(universe) <= 4096, "skill/facet universe exceeds declared bound")
        for chunk in range(max(1, (len(universe) + 63) // 64)):
            indexes = {item: i for i, item in enumerate(universe[64 * chunk:64 * (chunk + 1)])}
            def mask(items): return sum(1 << indexes[item] for item in set(items) if item in indexes)
            self.check(f"{name}/{chunk}", contract, mask(required), *(mask(a) for a in available), *flags)

    def rust(self):
        require(len({f.name for f in self.facts}) == len(self.facts), "duplicate obligation name")
        lines = ["// Generated finite obligations; no observation or learning claim is implied.",
                 "fn main() {", "    let checks = ["]
        for f in self.facts:
            c = registry.contracts[f.contract]
            args = ", ".join(str(v).lower() if t == "bool" else f"{v}{t}" for (_, t), v in zip(c.parameters, f.arguments))
            lines.append(f"        ({json.dumps(f.name)}, ncp_metaverify::{f.contract}({args})),")
        lines += ["    ];", "    let mut failed = 0usize;", "    for (name, passed) in checks {",
                  '        if !passed { eprintln!("{name}"); failed += 1; }', "    }",
                  '    println!("{{\\\"obligations\\\":{},\\\"failed\\\":{}}}", checks.len(), failed);',
                  "    if failed != 0 { std::process::exit(1); }", "}"]
        return "\n".join(lines) + "\n"


@dataclass(frozen=True)
class LearningItem:
    name: str
    kind: str
    level: int
    prerequisites: tuple = ()
    teaches: tuple = ()
    uses: tuple = ()
    practices: tuple = ()


def learning_graph(items, obligations):
    """Reusable topologically ordered DAG, with ancestor-only skill availability."""
    require(0 < len(items) <= 65535, "learning graph bound")
    index = {item.name: i for i, item in enumerate(items)}
    require(len(index) == len(items), "duplicate learning identity")
    ancestors, providers, levels = {}, {}, {}
    for i, item in enumerate(items):
        require(item.kind in {"foundation", "course", "project", "exercise"}, "unknown learning kind")
        require(type(item.level) is int and item.level == levels.get(item.kind, 0) + 1,
                f"{item.name}: difficulty must increase without gaps in its own sequence")
        levels[item.kind] = item.level
        parents = unique(list(item.prerequisites))
        ancestors[item.name] = set()
        for parent in sorted(parents):
            require(parent in index, f"{item.name}: unknown prerequisite {parent}")
            obligations.check(f"edge/{item.name}/{parent}", "ordered_edge", index[parent], i, len(items))
            ancestors[item.name].add(parent)
            ancestors[item.name].update(ancestors[parent])
        teaches, uses, practices = (unique(list(v)) for v in (item.teaches, item.uses, item.practices))
        require(not teaches or item.kind in {"foundation", "course"}, f"{item.name}: teach new skills in a course")
        require(not practices or item.kind == "project", "practice declarations belong to projects")
        require(practices <= uses, "projects must use every skill they claim to practice")
        for skill in teaches:
            require(skill not in providers, f"duplicate teaching identity: {skill}")
            providers[skill] = item.name
        available = {s for prior in items[:i] if prior.name in ancestors[item.name] for s in prior.teaches}
        practiced = {s for prior in items[:i] if prior.name in ancestors[item.name] for s in prior.practices}
        obligations.sets(f"skills/{item.name}", "learning_ready", uses, available, practiced,
                         flags=(item.kind == "exercise",))


def validate_blueprint(bp, lock):
    record(bp, ["schema", "name", "edition", "independent", "sources", "objectives", "units"])
    require(type(bp["schema"]) is int and bp["schema"] == 1 and bp["independent"] is True, "blueprint edition/schema")
    record(lock, ["schema", "review_basis", "objectives"])
    require(type(lock["schema"]) is int and lock["schema"] == 1, "source-lock schema")
    objects = sequence(bp["objectives"])
    locked = sequence(lock["objectives"])
    wanted = {o["id"]: o for o in locked}
    require(len(wanted) == len(locked) == 113, "source review must preserve all 113 objective identities")
    require(sum(len(o["facets"]) for o in locked) >= 203, "source review cannot drop original facets")
    require(len(objects) == len(wanted) and {o["id"] for o in objects} == set(wanted), "source objective omissions or duplicates")
    record(bp["sources"], ["AIO-W", "AIO-G", "AIN", "AII"])
    for source, definition in bp["sources"].items():
        record(definition, ["url", "date", "weights", "numbering"], ["guide"])
        weights = sequence(definition["weights"], 16)
        require(all(type(w) is int and 0 < w <= 100 for w in weights) and sum(weights) == 100, f"{source}: weights")
    by_id = {}
    for o in objects:
        record(o, ["id", "exam", "source", "locator", "label", "facets", "unit", "assessed_by", "taught_by", "combined_by"])
        identity = {key: o[key] for key in ["id", "exam", "source", "locator", "label", "facets"]}
        require(identity == wanted[o["id"]], f"{o['id']}: source obligations changed; review the source lock explicitly")
        require(o["source"] in bp["sources"] and o["exam"] == o["source"].split("-")[0], "source-domain mismatch")
        unique(o["facets"], nonempty=True)
        require(all("/" not in f for f in o["facets"]), "facet identity delimiter collision")
        by_id[o["id"]] = o
    units = sequence(bp["units"], 20)
    require(len(units) == 20, "expected 20 learning units")
    for n, u in enumerate(units, 1):
        record(u, ["id", "title", "level", "objectives", "course", "project", "exercises", "prerequisites", "status"])
        require(type(u["level"]) is int and u["level"] == n and u["id"] == f"U{n:02}", "unit order/identity")
        require(u["course"] == f"C{n:02}" and u["project"] == f"P{n:02}"
                and u["exercises"] == [f"E{n:02}a", f"E{n:02}b"], "learning identity mismatch")
        require(u["prerequisites"] == ([] if n == 1 else [f"U{n-1:02}"]), "unit prerequisites")
        text(u["title"])
        ids = unique(u["objectives"], nonempty=True)
        require(ids <= set(by_id), "unknown objective reference")
        require({by_id[oid]["exam"] for oid in ids} == {"AIO", "AIN", "AII"}, f"{u['id']}: missing domain")
        facets = [oid + "/" + f for oid in u["objectives"] for f in by_id[oid]["facets"]]
        require(0 < len(facets) <= 63, "station facet mask must be nonempty and fit without truncation")
    for o in objects:
        require(re.fullmatch(r"U(?:0[1-9]|1[0-9]|20)", o["unit"]) is not None, "unknown home unit")
        u = units[int(o["unit"][1:]) - 1]
        require(o["id"] in u["objectives"] and o["taught_by"] == u["course"]
                and o["combined_by"] == u["project"] and o["assessed_by"] == u["exercises"], "broken source learning chain")
    return by_id


def validate_authoring(authored, bp):
    require(set(authored) == set(range(1, 21)), "authored unit inventory")
    for n, d in authored.items():
        record(d, ["skills", "semantics", "steps", "project", "incidents", "symbols", "code"])
        unique(d["skills"], nonempty=True)
        require(0 < len(d["skills"]) <= 64, f"C{n:02}: skill count")
        require(len(sequence(d["incidents"])) == 2 and len(sequence(list(d["symbols"]))) == 3, "incident/symbol arity")
        for field in ["semantics", "project", "code"]: text(d[field])
        for field in ["steps", "incidents", "symbols"]:
            require(bool(d[field]), f"{field}: empty")
            for value in d[field]: text(value)
        import ast
        ast.parse(d["code"], filename=f"C{n:02}.py")


def validate_materials(bp, curriculum, bank, rendered, obligations):
    objects = {o["id"]: o for o in bp["objectives"]}
    require(len(sequence(curriculum)) == 20 and len(sequence(bank)) == 40, "course/station inventory")
    items = [LearningItem("C00", "foundation", 1)]
    courses, projects, exercises = [], [], []
    taught = set()
    for n, (u, entry) in enumerate(zip(bp["units"], curriculum), 1):
        record(entry, ["unit", "course", "project", "exercises", "delivery"])
        require(entry["unit"] == u["id"], "curriculum unit mismatch")
        c, p = entry["course"], entry["project"]
        record(c, ["id", "level", "skills", "uses", "prerequisites", "skill_names", "document", "example", "facets"])
        record(p, ["id", "level", "skills", "prerequisites", "document", "facets"])
        require(c["id"] == u["course"] and p["id"] == u["project"] and c["level"] == p["level"] == n, "course/project identity")
        require(c["skills"] == [f"C{n:02}.S{i:02}" for i in range(1, len(c["skills"]) + 1)]
                and set(c["skill_names"]) == unique(c["skills"], nonempty=True), "teaching identity/name mismatch")
        require(set(c["uses"]) == taught, "course must declare its prior taught skill dependencies")
        taught.update(c["skills"])
        require(c["prerequisites"] == ["C00" if n == 1 else f"C{n-1:02}"], "course prerequisite mismatch")
        expected_projects = [f"C{i:02}" for i in range(1, 21)] + ([] if n == 1 else [f"P{n-1:02}"])
        require(p["prerequisites"] == expected_projects, "project must assume all courses and prior project")
        courses.append(LearningItem(c["id"], "course", c["level"], tuple(c["prerequisites"]), tuple(c["skills"]), tuple(c["uses"])))
        projects.append(LearningItem(p["id"], "project", p["level"], tuple(p["prerequisites"]), uses=tuple(p["skills"]), practices=tuple(p["skills"])))
        expected = {oid + "/" + f for oid in u["objectives"] for f in objects[oid]["facets"]}
        require(len(sequence(entry["exercises"])) == 2, "two incident variants required")
        for stage, expected_doc in [(c, f"courses/{c['id']}.md"), (p, f"projects/{p['id']}.md")]:
            require(stage["document"] == expected_doc and expected_doc in rendered, "missing/unsafe learning document")
            require(unique(stage["facets"], nonempty=True) == expected, f"{stage['id']}: facet mismatch")
            require("```mermaid" in rendered[expected_doc], "missing transition diagram")
            obligations.sets(f"facets/{stage['id']}", "coverage", expected, stage["facets"])
        require(c["example"] == f"examples/{c['id']}.py" and c["example"] in rendered, "missing course code")
        require(all(f"`{f}`" in rendered[c["document"]] for f in expected), "course omits a facet row")
        for j, e in enumerate(entry["exercises"]):
            record(e, ["id", "level", "skills", "prerequisites", "prompt", "facets"])
            require(e["id"] == u["exercises"][j] and e["level"] == 2 * n - 1 + j
                    and e["prerequisites"] == [p["id"]], "exercise identity/order/prerequisite")
            require(e["prompt"] == f"exercises/{e['id']}.md" and e["prompt"] in rendered, "missing exercise prompt")
            require(unique(e["facets"], nonempty=True) == expected, "exercise facet mismatch")
            exercises.append(LearningItem(e["id"], "exercise", e["level"], tuple(e["prerequisites"]), uses=tuple(e["skills"])))
            obligations.sets(f"facets/{e['id']}", "coverage", expected, e["facets"])
            s = bank[2 * (n - 1) + j]
            record(s, ["id", "name", "facets"])
            require(type(s["id"]) is int and s["id"] == 2 * (n - 1) + j and s["name"] == e["id"].lower(), "station identity mismatch")
            rows = sequence(s["facets"], 63)
            require({f["id"] for f in rows} == expected and len(rows) == len(expected), "station facet mismatch")
            domains = 0
            for f in rows:
                record(f, ["id", "domain", "tier"])
                domain = {"AIO": 0, "AIN": 1, "AII": 2}[objects[f["id"].split("/")[0]]["exam"]]
                require(type(f["domain"]) is int and f["domain"] == domain and type(f["tier"]) is int and f["tier"] == 4, "station domain/tier drift")
                domains |= 1 << domain
            # Domain inventory is a schema check; it creates no baseline/holdout evidence.
            obligations.check(f"domains/{e['id']}", "report_complete", 7, domains, 0)
            obligations.check(f"order/{e['id']}", "next_stage", s["id"], 2 * (n - 1) + j, len(bank))
    learning_graph(items + courses + projects + exercises, obligations)
