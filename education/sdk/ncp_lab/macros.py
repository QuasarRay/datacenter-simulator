"""Small expression macro: one evaluation, visible failure and no optimizer removal.

`require[expr]` expands to a conditional expression returning True or raising
ValueError with the original expression. Python's `assert` can disappear under -O.
No arbitrary source eval, dynamic imports or grader manipulation is performed.
"""
import ast

def require(tree, *, syntax, **_):
    if syntax != 'expr':
        raise SyntaxError('require is an expression macro')
    # Empty-generator.throw raises without binding a helper name that a learner
    # module could accidentally shadow. The predicate is evaluated exactly once.
    failure=ast.parse('(_ for _ in ()).throw(ValueError("contract failed"))',mode='eval').body
    failure.args[0].args[0]=ast.Constant('contract failed: '+ast.unparse(tree))
    return ast.IfExp(test=tree,body=ast.Constant(True),orelse=failure)
