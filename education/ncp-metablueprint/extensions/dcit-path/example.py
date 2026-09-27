from integrations.incus.diagnostics import Endpoint, Flow, path_study

@path_study
def dataset_dependency(source="compute-a", target="compute-b"):
    return (
        Flow(Endpoint(source, "data0"), Endpoint(target, "data0")),
        Flow(Endpoint("control", "data0"), Endpoint("services", "data0")),
    )

def investigate(fabric, output):
    # fabric is already compiled from NetBox and deployed by the trainer.
    # Planning resolves names to immutable NetBox IDs before effects.
    study = dataset_dependency(fabric)
    return study.run(output)
