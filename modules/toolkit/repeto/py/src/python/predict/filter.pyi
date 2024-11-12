from biobit.core.loc import IntoInterval


class Filter:
    """Filter to apply to predicted inverted repeats."""

    def __init__(self) -> None: ...

    def set_min_score(self, min_score: int) -> Filter:
        """
        Set the minimum alignment score for a predicted inverted repeat.
        """
        pass

    def set_rois(self, rois: list[IntoInterval]) -> Filter:
        """
        Filter predicted inverted repeats relative the given set of regions of interest (ROIs).
        """
        pass

    def set_min_roi_overlap(self, total: int, ungapped: int) -> Filter:
        """
        Set the minimum overlap between a predicted inverted repeat and a region of interest. Defined in terms of
        the total number of matches inside the ROIs and the minimum overlap between an ungapped run of matches and ROIs.
        """
        pass

    def set_min_matches(self, total: int, ungapped: int) -> Filter:
        """
        Set the minimum number of matches for a predicted inverted repeat.
        Defined in terms of the total number of matches and the minimum number of ungapped matches.
        """
        pass
