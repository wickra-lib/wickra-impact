"""Type stubs for the wickra_impact package."""

__version__: str

class Impact:
    """A market-impact backtest driven by JSON commands."""

    def __init__(self, spec_json: str) -> None:
        """Construct a backtest handle from a spec JSON (``"{}"`` to defer).

        Raises ``ValueError`` on an invalid spec.
        """
        ...

    def command(self, cmd_json: str) -> str:
        """Apply a command JSON and return the response JSON.

        Raises ``RuntimeError`` on a command failure (a missing spec, bad data).
        """
        ...

    @staticmethod
    def version() -> str:
        """The library version."""
        ...
