from helpers import status

from magika_datasets.validators.text import yaml_text


def test_yaml_documents_require_a_hint():
    assert yaml_text.REQUIRES_HINT and yaml_text.CONTEXT_REQUIRED
    document = b"name: CI\non:\n  push:\n    branches: [main]\njobs:\n  build:\n    steps:\n      - uses: actions/checkout@v4\n      - run: echo hi\n"
    observation = yaml_text.validate(document, frozenset({"yaml"}))
    assert (observation.status, observation.format_id) == ("pass", "yaml")
    multi = b"---\na: 1\n---\nb: 2\n...\n"
    assert status(yaml_text, multi, frozenset({"yaml"})) == "pass"
    assert status(yaml_text, b"key: [unclosed\n", frozenset({"yaml"})) == "fail"
    assert status(yaml_text, b"just a scalar line\n", frozenset({"yaml"})) == "inconclusive"
    assert (
        status(yaml_text, b"a: !!python/object/apply:os.system ['x']\n", frozenset({"yaml"}))
        == "fail"
    )
    assert status(yaml_text, b"\xff\xfe\x00\xd8", frozenset({"yaml"})) == "fail"
    bomb = b"a: &a [x, x, x, x, x, x, x, x, x, x]\n" + b"".join(
        b"%s: &%s [*%s, *%s, *%s, *%s, *%s, *%s, *%s, *%s]\n"
        % ((chr(98 + i).encode(),) * 2 + (chr(97 + i).encode(),) * 8)
        for i in range(8)
    )
    assert status(yaml_text, bomb, frozenset({"yaml"})) in ("pass", "inconclusive")
