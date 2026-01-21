try:
    from importlib.metadata import version,PackageNotFoundError
except ImportError:
    from importlib_metadata import version,PackageNotFoundError

def get_version():
    try:
        return version("mirascript")
    except PackageNotFoundError:
        return "0.0.0+unknown"

__version__ = get_version()