"""Reject eager ML/framework linkage in the deferred-backend distribution."""
import subprocess
import sys
from pathlib import Path


def check(path):
    output = subprocess.check_output(['otool', '-L', str(path)], text=True)
    forbidden = ('Metal.framework', 'CoreGraphics.framework', 'libmagika_runtime')
    identity = subprocess.check_output(['otool', '-D', str(path)], text=True).splitlines()[1:]
    dependencies = [line.split()[0] for line in output.splitlines()[1:] if line.strip()]
    dependencies = [dep for dep in dependencies if dep not in identity]
    assert not any(name in dep for name in forbidden for dep in dependencies), output
    return output


if __name__ == '__main__':
    for arg in sys.argv[1:]:
        print(check(Path(arg)))
