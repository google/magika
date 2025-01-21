import hashlib
from pathlib import Path

import magika

test_path = Path(__file__).parent.parent.parent / "tests_data/basic/python/code.py"

m = magika.Magika()

fs = m._extract_features_from_path(
    test_path,
    beg_size=1024,
    mid_size=0,
    end_size=1024,
    padding_token=256,
    block_size=4096,
    use_inputs_at_offsets=False,
)


def serialize(fs):
    return hashlib.sha256(str(fs.beg + fs.end).encode("ascii")).hexdigest()


print(fs)
print(serialize(fs))

f = open(test_path, "rb")
content_bytes = f.read()
print(f"content bytes len: {len(content_bytes)}")
f.close()

content_bytes_2 = test_path.read_bytes()
print(f"content bytes 2 len: {len(content_bytes_2)}")

print(f"file size: {test_path.stat().st_size}")

content_bytes_ints = list(map(int, content_bytes[:64]))
print(content_bytes_ints)
