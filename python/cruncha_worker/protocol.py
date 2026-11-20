# SPDX-License-Identifier: GPL-3.0-or-later
"""Framing: each message is a 4-byte big-endian length and a UTF-8 JSON header. A header may
list ``blobs`` (byte lengths); that many raw byte frames follow, each with its own 4-byte
length. Blobs carry images and Arrow IPC streams without base64 or JSON escaping."""

import json
import struct

_LEN = struct.Struct(">I")
MAX_FRAME = 1 << 31


class Closed(Exception):
    """The other end closed the channel."""


def _read_exact(f, n):
    buf = bytearray(n)
    view = memoryview(buf)
    got = 0
    while got < n:
        k = f.readinto(view[got:])
        if not k:
            raise Closed()
        got += k
    return bytes(buf) if n < 4096 else buf


def _read_frame(f):
    (n,) = _LEN.unpack(_read_exact(f, 4))
    if n >= MAX_FRAME:
        raise ValueError(f"frame too large: {n}")
    return _read_exact(f, n)


def read_message(f):
    """Read one message: ``(header: dict, blobs: list[bytes])``."""
    header = json.loads(_read_frame(f))
    blobs = [_read_frame(f) for _ in range(int(header.get("blobs", 0)))]
    return header, blobs


def write_message(f, header, blobs=()):
    blobs = list(blobs)
    header = dict(header)
    if blobs:
        header["blobs"] = len(blobs)
    data = json.dumps(header, separators=(",", ":"), allow_nan=False, default=str).encode()
    f.write(_LEN.pack(len(data)))
    f.write(data)
    for b in blobs:
        f.write(_LEN.pack(len(b)))
        f.write(b)
    f.flush()
