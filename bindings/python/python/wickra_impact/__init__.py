"""Wickra Impact — a market-impact backtester on the real historical L2 book.

Construct an :class:`Impact` from an :class:`ImpactSpec` JSON, drive it with
command JSONs (``set_spec``, ``run``, ``version``), and read back the response
JSON. The same command protocol crosses every language binding, so this Python
front-end drives the exact same core — and returns the byte-identical report —
as the native CLI.
"""

from ._wickra_impact import Impact, __version__

__all__ = ["Impact", "__version__"]
