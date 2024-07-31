import numpy as np
import pandas as pd
from attrs import define

from .libnorm import LibraryNormalization


@define(slots=True, frozen=True)
class PreComputedMedianOfRatiosNormalization(LibraryNormalization):
    scaling_factors: dict[str, float]

    def normalize(self, elements: pd.DataFrame, inplace: bool = False) -> pd.DataFrame:
        if inplace:
            raise NotImplementedError("Inplace normalization is not supported for precomputed normalization.")

        factors = {}
        for column in elements.columns:
            if column not in self.scaling_factors:
                raise ValueError(f"Scaling factor for the library {column} is missing.")
            factors[column] = self.scaling_factors[column]
        scaling_factors = pd.Series(factors)

        # print(f"Final scaling factors: {scaling_factors}")

        elements = elements.div(scaling_factors, axis=1)
        return elements


@define(slots=True, frozen=True)
class MedianOfRatiosNormalization(LibraryNormalization):
    def _calculate_scaling_factors(self, elements: pd.DataFrame) -> pd.Series:
        with np.errstate(divide='ignore'):
            logdata = elements.apply(np.log)
            average = logdata.mean(axis=1)

            mask = np.isfinite(average)
            average, logdata = average[mask], logdata[mask]

        ratio = logdata.sub(average, axis=0)
        median = ratio.median(axis=0)
        scaling_factors = np.exp(median)
        return scaling_factors

    def scaling_factors(self, elements: pd.DataFrame) -> pd.Series:
        return self._calculate_scaling_factors(elements)

    def normalize(self, elements: pd.DataFrame, inplace: bool = False) -> pd.DataFrame:
        if inplace:
            raise NotImplementedError("Inplace normalization is not supported for median of ratios normalization.")

        scaling_factors = self._calculate_scaling_factors(elements)
        elements = elements.div(scaling_factors, axis=1)
        return elements

    def to_precomputed(self, elements: pd.DataFrame) -> PreComputedMedianOfRatiosNormalization:
        scaling_factors = self._calculate_scaling_factors(elements)
        return PreComputedMedianOfRatiosNormalization(scaling_factors.to_dict())
