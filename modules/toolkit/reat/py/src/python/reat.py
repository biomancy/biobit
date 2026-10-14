from biobit.rs.toolkit.reat import Reat

from .selection import Mismatches, RequiredOrMismatches, RequiredSites

Selector = Mismatches | RequiredSites | RequiredOrMismatches

__all__ = ["Reat", "Selector"]
