from magika_datasets.validators.contract import Observation

NO_HINTS = frozenset()


def status(module, data: bytes, hints=NO_HINTS) -> str:
    """'not_applicable' for None, otherwise the observation status."""
    result = module.validate(data, hints)
    if result is None:
        return "not_applicable"
    assert isinstance(result, Observation)
    return result.status
