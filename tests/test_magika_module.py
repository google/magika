from pathlib import Path
import random
import tempfile

from magika.magika import Magika

from tests.utils import get_random_ascii_bytes


def test_extract_features_with_ascii():
    random.seed(42)

    _test_extract_features_with_content(get_random_ascii_bytes(0))
    _test_extract_features_with_content(get_random_ascii_bytes(1))
    _test_extract_features_with_content(get_random_ascii_bytes(2))
    _test_extract_features_with_content(get_random_ascii_bytes(8))
    _test_extract_features_with_content(get_random_ascii_bytes(10))
    _test_extract_features_with_content(get_random_ascii_bytes(256))
    _test_extract_features_with_content(get_random_ascii_bytes(511))
    _test_extract_features_with_content(get_random_ascii_bytes(512))
    _test_extract_features_with_content(get_random_ascii_bytes(513))
    _test_extract_features_with_content(get_random_ascii_bytes(1024))
    _test_extract_features_with_content(get_random_ascii_bytes(20_000))
    _test_extract_features_with_content(get_random_ascii_bytes(1_000_000))


def test_extract_features_with_spaces():
    random.seed(42)

    whitespace_sizes = [
        1, 2, 4, 10, 400, 511, 512, 513, 1023, 1024, 1025, 2000,
    ]

    for whitespace_size in whitespace_sizes:
        _test_extract_features_with_content((b' ' * whitespace_size) + get_random_ascii_bytes(128))
        _test_extract_features_with_content(get_random_ascii_bytes(128) + (b' ' * whitespace_size))
        _test_extract_features_with_content((b' ' * whitespace_size) + get_random_ascii_bytes(128) + (b' ' * whitespace_size))

    _test_extract_features_with_content(b' ' * 1000)


def _test_extract_features_with_content(content: bytes):
    print(f'Testing with content of len {len(content)}: {content[:min(128, len(content))]}...{content[-min(128, len(content)):]}')
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        tf_path = td_path / 'file.dat'

        with open(tf_path, 'wb') as f:
            f.write(content)

        content_stripped = content.strip()

        m = Magika()

        # beg_size, mid_size, end_size
        test_parts_sizes = [
            (512, 0, 512),
            (512, 0, 0),
            (0, 0, 512),
            (512, 0, 256),
            (256, 0, 512),
            (8, 0, 0),
            (8, 0, 8),
            (0, 0, 8),
            (1, 0, 0),
            (1, 0, 1),
            (0, 0, 1),
            (8, 0, 1),
            (8, 0, 2),
            (1, 0, 8),
            (2, 0, 8),
        ]

        for beg_size, mid_size, end_size in test_parts_sizes:
            print(f'Testing with {beg_size=}, {mid_size=}, {end_size=}, {len(content)=}')
            features = m.extract_features(tf_path, beg_size=beg_size, mid_size=mid_size, end_size=end_size)

            beg_ints_out = features['beg']
            mid_ints_out = features['mid']
            end_ints_out = features['end']

            beg_out = bytes(beg_ints_out)
            mid_out = bytes(mid_ints_out)
            end_out = bytes(end_ints_out)

            if beg_size > 0:
                if len(content_stripped) >= beg_size:
                    assert beg_out == content_stripped[:beg_size]
                else:
                    assert beg_out == content_stripped + (b'\x00' * (beg_size - len(content_stripped)))
            else:
                assert beg_out == b''

            # mid is not supported
            assert mid_size == 0
            assert mid_out == b''

            if end_size > 0:
                if len(content_stripped) >= end_size:
                    assert end_out == content_stripped[-end_size:]
                else:
                    assert end_out == (b'\x00' * (end_size - len(content_stripped))) + content_stripped
            else:
                assert end_out == b''