from typing import Hashable, Any

import numpy as np
import pandas as pd
from attrs import define


@define(slots=True, frozen=True)
class MedianOfRatiosNormalization:
    data: pd.DataFrame

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

    def scaling_factors(
            self, samples: dict[Hashable, list[Any]], minval: int = 0
    ) -> tuple[pd.DataFrame, pd.DataFrame, pd.Series]:
        columns = {}
        for sample, cols in samples.items():
            columns[sample] = self.data[cols].sum(axis=1)
        rawdf = pd.DataFrame(columns)

        mask = (rawdf >= minval).any(axis=1)
        df = rawdf[mask].copy()

        return rawdf, df, self._calculate_scaling_factors(df)

# @define(slots=True, frozen=True)
# class MedianOfRatiosNormalization:
#     data: pd.DataFrame
#
#     def _calculate_scaling_factors(self, elements: pd.DataFrame) -> pd.Series:
#         with np.errstate(divide='ignore'):
#             logdata = elements.apply(np.log)
#             average = logdata.mean(axis=1)
#
#             mask = np.isfinite(average)
#             average, logdata = average[mask], logdata[mask]
#
#         ratio = logdata.sub(average, axis=0)
#         median = ratio.median(axis=0)
#         scaling_factors = np.exp(median)
#         return scaling_factors
#
#     def scaling_factors(self, elements: pd.DataFrame) -> pd.Series:
#         return self._calculate_scaling_factors(elements)

# def normalize(self, elements: pd.DataFrame, inplace: bool = False) -> pd.DataFrame:
#     if inplace:
#         raise NotImplementedError("Inplace normalization is not supported for median of ratios normalization.")
#
#     scaling_factors = self._calculate_scaling_factors(elements)
#     elements = elements.div(scaling_factors, axis=1)
#     return elements

# """
# The LibraryNormalization protocol defines the interface for normalization methods used to normalize a diverse set of sequencing libraries. It makes the following assumptions:
#     * Each library is uniquely identified by a string.
#     * Library elements are identical across all libraries (e.g., transcripts, regions, exons, etc.).
#     * Each element in each library is represented by a float value (e.g., counts, abundance, CT-value, etc.).
#
# The protocol includes a single method, `normalize`, which takes a DataFrame where library elements are rows and libraries are columns.
# The `normalize` method should return a DataFrame of the same shape and order as the input DataFrame, but with normalized values.
# """
#
# """
# Normalize the values of the input DataFrame. In the DataFrame, rows represent elements and columns represent individual libraries.
# :param elements: A DataFrame with elements as rows and libraries as columns.
# :param inplace: A boolean value. If True, the normalization will be performed in-place on the input DataFrame.
# :return: A DataFrame with normalized values. The shape and order of the DataFrame is identical to the input DataFrame.
# """
