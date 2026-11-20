# SPDX-License-Identifier: GPL-3.0-or-later
"""Cruncha's Python worker (ADR-0005): runs a document's Python cells out of process.

The app starts one worker per document and talks to it over the worker's stdin and stdout
with length-prefixed frames (``protocol.py``). User code never writes to that channel:
``sys.stdout`` is captured per cell, and file descriptor 1 is pointed at stderr.
"""

PROTOCOL_VERSION = 1
