import numpy as np
import pandas as pd

from .median_of_ratios import MedianOfRatiosNormalization


def test_median_of_ratios():
    raw = {
        "Region": ["Region 1", "Region 2", "Region 3", "Region 4", "Region 5", "Region 6"],
        "Sample 1": [0, 50, 7, 2, 58, 0],
        "Sample 2": [2, 52, 9, 1, 76, 1],
        "Sample 3": [3, 86, 12, 4, 57, 0],
        "Sample 4": [3, 80, 10, 7, 52, 1],
        "Sample 5": [2, 66, 11, 7, 67, 1],
        "Sample 6": [4, 94, 18, 1, 97, 0],
        "Sample 7": [6, 67, 7, 1, 101, 0],
        "Sample 8": [2, 83, 8, 7, 98, 0],
    }
    data = pd.DataFrame(raw).set_index("Region").astype(np.float32)

    _, _, scaling = MedianOfRatiosNormalization(data).scaling_factors({x: [x] for x in data.columns})

    normalized = data.div(scaling, axis=1)

    raw = {
        "Region": ["Region 1", "Region 2", "Region 3", "Region 4", "Region 5", "Region 6"],
        "Sample 1": [0.000000, 68.551338, 9.597187, 2.742054, 79.519554, 0.000000],
        "Sample 2": [2.429501, 63.167027, 10.932755, 1.214751, 92.321037, 1.214751],
        "Sample 3": [2.454096, 70.350739, 9.816382, 3.272127, 46.627815, 0.000000],
        "Sample 4": [2.787316, 74.328438, 9.291055, 6.503738, 48.313484, 0.929105],
        "Sample 5": [1.950615, 64.370308, 10.728385, 6.827154, 65.345619, 0.975308],
        "Sample 6": [3.015028, 70.853165, 13.567628, 0.753757, 73.114441, 0.000000],
        "Sample 7": [7.280727, 81.301453, 8.494182, 1.213454, 122.558907, 0.000000],
        "Sample 8": [1.596096, 66.237999, 6.384386, 5.586338, 78.208725, 0.000000],
    }
    expected = pd.DataFrame(raw).set_index("Region").astype(np.float32)

    pd.testing.assert_frame_equal(normalized, expected, atol=1e-6)
