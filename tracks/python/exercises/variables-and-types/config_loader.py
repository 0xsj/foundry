from dataclasses import dataclass


@dataclass
class Config:
    """Typed service configuration.

    Python is dynamically typed — the type hints here are documentation
    and tooling aids, not runtime enforcement. YOUR parsing logic is
    what actually enforces types.
    """

    host: str
    port: int
    max_retries: int
    timeout: float
    debug: bool
    service_name: str


def defaults() -> Config:
    """Return a Config with application-level defaults."""
    # TODO: implement
    raise NotImplementedError


def load_config(raw: dict[str, str]) -> Config:
    """Parse raw string key-value pairs into a typed Config.

    Missing keys should fall back to defaults.
    Invalid values should raise a ValueError.

    Hint: int(), float() do string conversion.
    For bool, think carefully — what should "true", "false", "1", "0" do?
    bool("false") is True in Python. That's a trap.
    """
    # TODO: implement
    raise NotImplementedError
