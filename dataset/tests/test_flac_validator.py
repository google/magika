import struct

from helpers import status

from magika_datasets.validators.media import flac


def streaminfo(total_samples=8):
    body = struct.pack(">HH", 4096, 4096) + (0).to_bytes(3, "big") * 2
    packed = (
        (44100 << 44) | (1 << 41) | (15 << 36) | total_samples
    )  # rate, channels-1, bps-1, samples
    body += packed.to_bytes(8, "big") + b"\0" * 16
    return bytes([0x80]) + len(body).to_bytes(3, "big") + body


def frame(blocking=0, number=0, samples=8):
    header = bytes(
        [0xFF, 0xF8 | blocking, 0x1A, 0x08, number]
    )  # 192 samples, 44.1 kHz, mono, 16-bit
    header += bytes([flac.crc8(header)])
    body = b"\x00" * 6  # constant subframe payload
    data = header + body
    return data + struct.pack(">H", flac.crc16(data))


def audio(frames=2, total=None):
    total = 192 * frames if total is None else total
    return b"fLaC" + streaminfo(total) + b"".join(frame(number=n) for n in range(frames))


def test_flac_blocks_and_frame_crcs():
    observation = flac.validate(audio(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "flac")
    assert status(flac, audio()[:-1]) == "fail"
    assert status(flac, audio() + b"\x55") == "fail"  # a zero trailer is CRC-invisible
    corrupt = bytearray(audio())
    corrupt[-4] ^= 1
    assert status(flac, bytes(corrupt)) == "fail"
    assert status(flac, audio(total=999)) == "fail"
    assert status(flac, audio(total=0)) == "pass"
    assert status(flac, b"fLaC" + streaminfo()[:-3]) == "fail"
    assert status(flac, b"fLaX" + b"\0" * 40) == "not_applicable"


def test_flac_crc_tables_match_bitwise_reference():
    def bitwise(data, polynomial, width):
        register, top, mask = 0, 1 << (width - 1), (1 << width) - 1
        for byte in data:
            register ^= byte << (width - 8)
            for _ in range(8):
                register = (
                    ((register << 1) ^ polynomial) & mask
                    if register & top
                    else (register << 1) & mask
                )
        return register

    sample = bytes(range(256)) * 3
    assert flac.crc8(sample) == bitwise(sample, 0x07, 8)
    assert flac.crc16(sample) == bitwise(sample, 0x8005, 16)
