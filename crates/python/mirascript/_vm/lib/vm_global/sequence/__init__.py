from .map_filter import map, filter, filter_map
from .len import len
from .entries import keys, values, entries
from .all_any import all, any
from .find import find
from .flatten import flatten
from .repeat import repeat
from .reverse import reverse
from .sort import sort, sort_by
from .unique import unique, unique_by
from .zip import zip
from .with_ import with_
from .new import new_array, new_record

globals()["with"] = with_
__all__ = [
    "with",  # pyright: ignore[reportUnsupportedDunderAll]
    "map",
    "filter",
    "filter_map",
    "len",
    "keys",
    "values",
    "entries",
    "all",
    "any",
    "find",
    "flatten",
    "repeat",
    "reverse",
    "sort",
    "sort_by",
    "unique",
    "unique_by",
    "zip",
    "new_array",
    "new_record",
]
