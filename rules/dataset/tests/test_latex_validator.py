from magika_datasets.validators.text import latex

TEX = b"\\documentclass{article}\n% comment {\n\\begin{document}\n\\section{Hi}\nText \\verb|{| and \\emph{x}.\n\\begin{verbatim}\n\\end{nothing} {\n\\end{verbatim}\n\\end{document}\n"
HINT = frozenset({"latex"})


def test_source_passes_only_when_hinted():
    assert latex.validate(TEX, frozenset()) is None
    assert latex.validate(TEX, HINT).status == "pass"


def test_unbalanced_groups_and_environments_are_inconclusive():
    assert latex.validate(TEX + b"}", HINT).status == "inconclusive"
    assert (
        latex.validate(TEX.replace(b"\\end{document}", b"\\end{article}"), HINT).status
        == "inconclusive"
    )
    assert latex.validate(b"\\begin{itemize}\\item a", HINT).status == "inconclusive"


def test_plain_tex_end_and_macro_built_environments_are_allowed():
    assert latex.validate(b"\\input x \\bye \\end", HINT).status == "pass"
    assert (
        latex.validate(b"\\newenvironment{x}{\\begin{#1}}{\\end{\\foo}} \\a \\b", HINT).status
        == "pass"
    )
