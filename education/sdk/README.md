# Operator code

Install this local package in the trainer's Python environment. `Twin` speaks
the existing Rust simulator's JSON API; it does not reimplement topology or
collectives. `algebra` uses immutable values and unpythonic pipelines for planning.
`require` is an mcpyrate expression macro. `Cluster` uses kr8s with an explicitly
selected Rusternetes kubeconfig and rejects adoption of a real worker.

Do not compress an effect's ownership, units, deadline or rollback into an
unreadable one-liner. Use functional composition for pure transformations;
keep Incus, DeepOps, NetBox and cluster API mutations explicit. Additional
functional libraries are welcome when they remove real boilerplate and their
contracts are checked; none is a prerequisite merely because it exists.

Import `mcpyrate.activate` in the launcher before importing a macro-enabled
module, or compile that module with `mcpyrate.compiler.run`. Macro expansion is
part of the reviewable source; inspect it before applying generated infrastructure.
This teaching macro is not part of the trusted grading kernel.

The native `integrations.incus.lab.Lab` API runs real pinned DeepOps tasks and
records its report. It needs a qualified active Incus journal. Do not copy the
trainer's Incus socket, secrets or assessment evidence into a learner container.
Privileged operator examples belong to a training account or reviewed trainer
automation; production assessment requires separate OS identities.
