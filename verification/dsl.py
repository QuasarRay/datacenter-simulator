"""A deliberately closed, typed expression language for total decision predicates.

One expression tree emits Rust execution, Verus execution and Kani assertions.
No raw code, assumptions, preconditions, recursion, loops or proof bypasses exist
in this language. The compiler and specifications are still trusted components.
"""
from dataclasses import dataclass
import re

TYPES = {"bool": 1, "u8": 8, "u16": 16, "u64": 64}
IDENT = re.compile(r"[a-z][a-z0-9_]*\Z")
RESERVED = {"result", "self", "super", "crate", "type", "fn", "mod", "pub", "use",
            "let", "mut", "ref", "match", "impl", "trait", "where", "loop", "move",
            "return", "true", "false", "if", "else", "const", "static", "unsafe",
            "as", "async", "await", "break", "continue", "dyn", "enum", "extern", "for", "in", "struct", "while",
            "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof", "unsized",
            "virtual", "yield", "try", "gen", "union", "macro_rules", "spec", "proof", "open", "tracked",
            "ghost", "verus", "requires", "ensures", "decreases", "recommends", "returns", "invariant",
            "assert", "assume", "forall", "exists", "choose"}


class ContractError(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise ContractError(message)


def identifier(name):
    require(type(name) is str and IDENT.fullmatch(name) and name not in RESERVED,
            f"invalid identifier: {name!r}")
    return name


def value_type(value, kind):
    require(kind in TYPES, f"unsupported type: {kind}")
    require(type(value) is (bool if kind == "bool" else int), f"expected {kind}")
    if kind != "bool":
        require(0 <= value < 1 << TYPES[kind], f"{kind} out of range: {value}")
    return value


@dataclass(frozen=True, eq=False)
class Expr:
    op: str
    kind: str
    args: tuple

    def __bool__(self):
        raise ContractError("use & and | to build boolean expressions; Python and/or would erase branches")

    def binary(self, op, other):
        other = other if isinstance(other, Expr) else literal(other, self.kind)
        return node(op, self, other)

    def __and__(self, other): return self.binary("and" if self.kind == "bool" else "band", other)
    def __or__(self, other): return self.binary("or" if self.kind == "bool" else "bor", other)
    def __invert__(self): return node("not", self)
    def eq(self, other): return self.binary("eq", other)
    def ne(self, other): return self.binary("ne", other)
    def lt(self, other): return self.binary("lt", other)
    def le(self, other): return self.binary("le", other)
    def implies(self, other): return (~self) | other


def literal(value, kind):
    return Expr("literal", kind, (value_type(value, kind),))


def variable(name, kind):
    identifier(name)
    require(kind in TYPES, f"unsupported type: {kind}")
    return Expr("var", kind, (name,))


def node(op, *args):
    require(all(isinstance(a, Expr) for a in args), "expression arguments required")
    if op == "not":
        require(len(args) == 1 and args[0].kind == "bool", "not requires one bool")
        kind = "bool"
    else:
        require(len(args) == 2 and args[0].kind == args[1].kind, "binary operand type mismatch")
        kind = args[0].kind
        if op in {"and", "or"}:
            require(kind == "bool", f"{op} requires bools")
        elif op in {"band", "bor", "lt", "le"}:
            require(kind != "bool", f"{op} requires unsigned integers")
        else:
            require(op in {"eq", "ne"}, f"unsupported operation: {op}")
        if op in {"eq", "ne", "lt", "le"}:
            kind = "bool"
    return Expr(op, kind, tuple(args))


def inspect(expr, bindings, depth=0, budget=None):
    """Recheck even hand-constructed Expr nodes; never trust a dataclass instance."""
    require(isinstance(expr, Expr) and depth <= 64, "invalid or excessive expression depth")
    if budget is None: budget = [4096]
    budget[0] -= 1
    require(budget[0] >= 0, "expanded expression exceeds the 4096-node proof budget")
    require(type(expr.args) is tuple and expr.kind in TYPES, "invalid expression shape")
    if expr.op == "literal":
        require(len(expr.args) == 1, "literal arity")
        value_type(expr.args[0], expr.kind)
        return set()
    if expr.op == "var":
        require(len(expr.args) == 1 and expr.args[0] in bindings, "unbound variable")
        require(bindings[expr.args[0]] == expr.kind, "variable type mismatch")
        return {expr.args[0]}
    names = set()
    for child in expr.args:
        names.update(inspect(child, bindings, depth + 1, budget))
    require(node(expr.op, *expr.args).kind == expr.kind, "forged result type")
    return names


def emit(expr):
    if expr.op == "var": return expr.args[0]
    if expr.op == "literal":
        return str(expr.args[0]).lower() if expr.kind == "bool" else f"{expr.args[0]}{expr.kind}"
    if expr.op == "not": return f"(!{emit(expr.args[0])})"
    operator = {"and": "&&", "or": "||", "band": "&", "bor": "|", "eq": "==",
                "ne": "!=", "lt": "<", "le": "<="}[expr.op]
    return f"({emit(expr.args[0])} {operator} {emit(expr.args[1])})"


def evaluate(expr, bindings):
    if expr.op == "var": return value_type(bindings[expr.args[0]], expr.kind)
    if expr.op == "literal": return expr.args[0]
    a = evaluate(expr.args[0], bindings)
    if expr.op == "not": return not a
    if expr.op == "and": return a and evaluate(expr.args[1], bindings)
    if expr.op == "or": return a or evaluate(expr.args[1], bindings)
    b = evaluate(expr.args[1], bindings)
    return {"band": lambda: a & b, "bor": lambda: a | b, "eq": lambda: a == b,
            "ne": lambda: a != b, "lt": lambda: a < b, "le": lambda: a <= b}[expr.op]()


@dataclass(frozen=True)
class Contract:
    name: str
    parameters: tuple
    body: Expr
    properties: tuple
    witnesses: tuple
    scope: str

    def validate(self):
        identifier(self.name)
        require(type(self.scope) is str and bool(self.scope.strip()), f"{self.name}: missing scope")
        require(1 <= len(self.parameters) <= 16 and 1 <= len(self.properties) <= 16
                and 2 <= len(self.witnesses) <= 32, "contract expansion bounds exceeded or empty")
        bindings = {}
        for name, kind in self.parameters:
            identifier(name)
            require(name not in bindings and kind in TYPES, "duplicate parameter or invalid type")
            bindings[name] = kind
        require(bool(bindings) and self.body.kind == "bool", "nonempty parameters and bool result required")
        require(inspect(self.body, bindings) == set(bindings), f"{self.name}: unused parameter")
        require(bool(self.properties), f"{self.name}: empty specification")
        used = set()
        for p in self.properties:
            require(p.kind == "bool", "postcondition must be boolean")
            names = inspect(p, {**bindings, "result": "bool"})
            require("result" in names, "postcondition must constrain the result")
            used.update(names)
        require(set(bindings) <= used, "specification omits an input")
        results = set()
        for inputs, expected in self.witnesses:
            require(len(inputs) == len(bindings) and type(expected) is bool, "bad witness shape")
            env = {name: value_type(value, kind) for (name, kind), value in zip(self.parameters, inputs)}
            actual = evaluate(self.body, env)
            require(actual == expected, f"{self.name}: witness contradicts implementation")
            require(all(evaluate(p, {**env, "result": expected}) for p in self.properties), "witness contradicts specification")
            # A trivial postcondition (result == result) must not pass this witness.
            require(not all(evaluate(p, {**env, "result": not expected}) for p in self.properties),
                    "specification does not distinguish the incorrect result")
            results.add(expected)
        require(results == {True, False}, f"{self.name}: both accepting and rejecting witnesses required")
        return self

    def call(self, *values):
        require(len(values) == len(self.parameters), f"{self.name}: argument count")
        bindings = {n: value_type(v, t) for (n, t), v in zip(self.parameters, values)}
        return evaluate(self.body, bindings)


class Registry:
    def __init__(self): self.contracts = {}

    def predicate(self, *, parameters, witnesses, scope):
        """The decorated builder returns (implementation, independent postconditions)."""
        def register(builder):
            require(builder.__name__ not in self.contracts, "duplicate contract")
            params = tuple(parameters.items())
            body, properties = builder(*(variable(n, t) for n, t in params), Expr("var", "bool", ("result",)))
            c = Contract(builder.__name__, params, body, tuple(properties), tuple(witnesses), scope).validate()
            self.contracts[c.name] = c
            return c
        return register
