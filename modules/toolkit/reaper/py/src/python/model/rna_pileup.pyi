from typing import Self, final

from biobit.core.loc import IntoChainInterval, IntoOrientation

@final
class RNAPileup:
    def __new__(cls) -> Self: ...
    def set_sensitivity(self, sensitivity: float) -> RNAPileup: ...
    def set_control_baseline(self, control_baseline: float) -> RNAPileup: ...
    def set_min_signal(self, min_signal: float) -> RNAPileup: ...
    def add_control_model(
        self,
        orientation: IntoOrientation,
        regions: list[IntoChainInterval],
        uniform_baseline: bool,
        winsizes: list[int],
    ) -> RNAPileup: ...

__all__ = ["RNAPileup"]
