from biobit.rs.toolkit.countit.rigid.resolution import (
    AnyOverlap,
    OverlapWeighted,
    TopRanked,
)

type IntoResolution = AnyOverlap | OverlapWeighted | TopRanked

__all__ = ["AnyOverlap", "IntoResolution", "OverlapWeighted", "TopRanked"]
