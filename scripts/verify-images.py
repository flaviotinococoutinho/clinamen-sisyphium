#!/usr/bin/env python3
"""Validate Tabula PNG/PPM with independent standard-library decoding (REQ-H-02)."""
import argparse
import binascii
import os
from pathlib import Path
import struct
import subprocess
import tempfile
import zlib


def require(condition, message):
    """Reject malformed images even when Python assertions are disabled."""
    if not condition:
        raise ValueError(message)


def decode_png(data):
    """Check the exact indexed PNG contract, chunks, DEFLATE and both checksums."""
    require(data[:8] == b"\x89PNG\r\n\x1a\n", "PNG signature")
    cursor = 8
    chunks = []
    while cursor < len(data):
        require(cursor + 12 <= len(data), "truncated chunk")
        length = struct.unpack_from(">I", data, cursor)[0]
        end = cursor + 12 + length
        require(end <= len(data), "truncated payload")
        kind = data[cursor + 4:cursor + 8]
        payload = data[cursor + 8:end - 4]
        crc = struct.unpack_from(">I", data, end - 4)[0]
        require(crc == binascii.crc32(kind + payload) & 0xffffffff, "chunk CRC")
        chunks.append((kind, payload))
        cursor = end
        if kind == b"IEND":
            break
    require(cursor == len(data), "bytes after IEND")
    kinds = [kind for kind, _ in chunks]
    require(len(chunks) >= 4 and kinds[:2] == [b"IHDR", b"PLTE"]
            and kinds[-1] == b"IEND"
            and all(kind == b"IDAT" for kind in kinds[2:-1]), "chunk order")
    require(len(chunks[0][1]) == 13 and not chunks[-1][1], "IHDR/IEND length")
    width, height, depth, color, compression, filtering, interlace = struct.unpack(">IIBBBBB", chunks[0][1])
    require(1 <= width <= 65535 and 1 <= height <= 65535
            and width * height <= 33554432, "dimensions")
    require((depth, color, compression, filtering, interlace) == (8, 3, 0, 0, 0), "IHDR fields")
    palette = chunks[1][1]
    require(len(palette) == 48, "palette length")
    stream = b"".join(payload for _, payload in chunks[2:-1])
    require(stream[:2] == b"\x78\x01", "zlib header")
    decoder = zlib.decompressobj()
    raw = decoder.decompress(stream) + decoder.flush()
    require(decoder.eof and not decoder.unused_data and not decoder.unconsumed_tail, "zlib termination")
    require(len(raw) == height * (width + 1), "filtered raster length")
    require(struct.unpack(">I", stream[-4:])[0] == zlib.adler32(raw) & 0xffffffff, "Adler-32")
    # Inspect stored framing independently of zlib, including the exact 65535 boundary.
    cursor, blocks, stored = 2, [], bytearray()
    while True:
        require(cursor + 5 <= len(stream) - 4, "stored header")
        header, length, complement = struct.unpack_from("<BHH", stream, cursor)
        require(header in (0, 1) and length ^ complement == 65535, "stored LEN/NLEN")
        cursor += 5
        require(cursor + length <= len(stream) - 4, "stored payload")
        stored.extend(stream[cursor:cursor + length])
        blocks.append(length)
        cursor += length
        if header == 1:
            break
    require(cursor == len(stream) - 4 and bytes(stored) == raw, "stored termination")
    require(blocks == [65535] * (len(raw) // 65535) + ([len(raw) % 65535] if len(raw) % 65535 else []), "stored partition")
    rgb = bytearray()
    for row in range(height):
        start = row * (width + 1)
        require(raw[start] == 0, "row filter")
        for index in raw[start + 1:start + width + 1]:
            require(index < 16, "palette index")
            rgb.extend(palette[index * 3:index * 3 + 3])
    return width, height, bytes(rgb), len(blocks)


def decode_ppm(data):
    """Parse the encoder's binary P6 header without interpreting raster whitespace."""
    fields = data.split(b"\n", 3)
    require(len(fields) == 4 and fields[0] == b"P6" and fields[2] == b"255", "PPM header")
    dimensions = fields[1].split(b" ")
    require(len(dimensions) == 2, "PPM dimensions")
    width, height = map(int, dimensions)
    require(len(fields[3]) == width * height * 3, "PPM raster length")
    return width, height, fields[3]


def verify(binary):
    """Generate deterministic independent PNG/PPM pairs without any OS integration."""
    cases = [(1, 1, 255, 0), (320, 180, 255, 4096), (254, 257, 255, 4096),
             (257, 256, 255, 4096), (320, 180, 0, 4096)]
    with tempfile.TemporaryDirectory(prefix="sisyphium-images-") as folder:
        for width, height, energy, ticks in cases:
            common = [str(binary), "--width", str(width), "--height", str(height),
                      "--seed", "1", "--epoch", "1782054000000", "--ticks", str(ticks),
                      "--energy", str(energy)]
            png = subprocess.run(common, check=True, capture_output=True, cwd=folder)
            ppm = subprocess.run(common + ["--format", "ppm"], check=True, capture_output=True, cwd=folder)
            require(not png.stderr and not ppm.stderr, "unexpected diagnostics")
            decoded = decode_png(png.stdout)
            require(decoded[:3] == decode_ppm(ppm.stdout), "PNG/PPM RGB mismatch")
            require(decoded[:2] == (width, height), "requested dimensions")
            # Exercise strict decoder failure rather than trusting its happy path alone.
            for damaged in (png.stdout[:-1], png.stdout + b"x", png.stdout[:40] + bytes([png.stdout[40] ^ 1]) + png.stdout[41:]):
                try:
                    decode_png(damaged)
                except (ValueError, zlib.error):
                    pass
                else:
                    raise ValueError("decoder accepted corruption")
            print("PNG/PPM {}x{} energy={} ticks={}: RGB/CRC/Adler/filter/{} blocks OK".format(width, height, energy, ticks, decoded[3]))
        require(not os.listdir(folder), "state or other file created without --state/--output")


def main():
    """Resolve the binary before isolating all generated artifacts in a temporary folder."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=Path("target/debug/sisyphium-tabula"))
    options = parser.parse_args()
    binary = options.binary
    if os.name == "nt" and not binary.is_file() and binary.suffix.lower() != ".exe":
        binary = Path(str(binary) + ".exe")
    verify(binary.resolve(strict=True))


if __name__ == "__main__":
    main()
