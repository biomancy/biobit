from .mates_orientation import MatesOrientation
from .strandedness import Strandedness


class Layout:
    class Single(Layout):
        strandedness: Strandedness

        def __init__(self, value: Strandedness) -> None: ...

        def __repr__(self) -> str: ...

        def __hash__(self) -> int: ...

        def __eq__(self, other: object) -> bool: ...

        def __ne__(self, other: object) -> bool: ...

        def __lt__(self, other: object) -> bool: ...

        def __le__(self, other: object) -> bool: ...

        def __gt__(self, other: object) -> bool: ...

        def __ge__(self, other: object) -> bool: ...

    class Paired(Layout):
        strandedness: Strandedness
        orientation: MatesOrientation

        def __init__(self, strandedness: Strandedness, orientation: MatesOrientation) -> None: ...

        def __repr__(self) -> str: ...

        def __hash__(self) -> int: ...

        def __eq__(self, other: object) -> bool: ...

        def __ne__(self, other: object) -> bool: ...

        def __lt__(self, other: object) -> bool: ...

        def __le__(self, other: object) -> bool: ...

        def __gt__(self, other: object) -> bool: ...

        def __ge__(self, other: object) -> bool: ...