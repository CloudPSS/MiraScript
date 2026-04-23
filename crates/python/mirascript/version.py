# try:
#     from importlib.metadata import version,PackageNotFoundError
# except ImportError:
#     from importlib_metadata import version,PackageNotFoundError


def get_version():
    try:

        try:
            from importlib.metadata import version, PackageNotFoundError

            return version("mirascript")
        except ImportError:
            # from importlib_metadata import version,PackageNotFoundError
            import pkg_resources

            version = pkg_resources.get_distribution("mirascript").version
            return version
    except:
        return "0.0.0+unknown"


__version__ = get_version()
