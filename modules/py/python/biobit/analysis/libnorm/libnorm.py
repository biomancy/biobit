from abc import ABC, abstractmethod

import pandas as pd


class LibraryNormalization(ABC):
    """
    The LibraryNormalization protocol defines the interface for normalization methods used to normalize a diverse set of sequencing libraries. It makes the following assumptions:
        * Each library is uniquely identified by a string.
        * Library elements are identical across all libraries (e.g., transcripts, regions, exons, etc.).
        * Each element in each library is represented by a float value (e.g., counts, abundance, CT-value, etc.).

    The protocol includes a single method, `normalize`, which takes a DataFrame where library elements are rows and libraries are columns.
    The `normalize` method should return a DataFrame of the same shape and order as the input DataFrame, but with normalized values.
    """

    @abstractmethod
    def normalize(self, elements: pd.DataFrame, inplace: bool = False) -> pd.DataFrame:
        """
        Normalize the values of the input DataFrame. In the DataFrame, rows represent elements and columns represent individual libraries.
        :param elements: A DataFrame with elements as rows and libraries as columns.
        :param inplace: A boolean value. If True, the normalization will be performed in-place on the input DataFrame.
        :return: A DataFrame with normalized values. The shape and order of the DataFrame is identical to the input DataFrame.
        """
        ...
