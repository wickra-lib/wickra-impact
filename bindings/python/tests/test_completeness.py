"""The Python surface exposes exactly the documented API."""

import wickra_impact
from wickra_impact import Impact


def test_module_exports() -> None:
    assert set(wickra_impact.__all__) == {"Impact", "__version__"}


def test_impact_methods() -> None:
    for name in ("command", "version"):
        assert hasattr(Impact, name)


def test_version_is_a_string() -> None:
    assert isinstance(wickra_impact.__version__, str)
    assert wickra_impact.__version__
