import inspect
import warnings
from functools import wraps


def unstable():
    """Decorator to mark a function as unstable."""

    def decorate(function):
        @wraps(function)
        def wrapper(*args, **kwargs):
            warnings.warn(
                f"`{function.__name__}` is considered unstable.",
                stacklevel=0,
                category=UserWarning,
            )
            return function(*args, **kwargs)

        wrapper.__signature__ = inspect.signature(function)
        return wrapper

    return decorate
