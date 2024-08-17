from biobit.core.loc import IntoOrientation


class NMS:
    def __init__(self) -> None: ...

    def set_fecutoff(self, fecutoff: float) -> NMS: ...

    def set_group_within(self, group_within: int) -> NMS: ...

    def set_slopfrac(self, slopfrac: float) -> NMS: ...

    def set_sloplim(self, minslop: int, maxslop: int) -> NMS: ...

    def set_boundaries(self, orientation: IntoOrientation, boundaries: list[int]) -> NMS: ...
