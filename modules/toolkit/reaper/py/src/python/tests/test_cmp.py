import pickle

import pytest
from biobit.toolkit.reaper import cmp


def test_reaper_pseudocount_roundtrip():
    enrichment = cmp.Enrichment().set_pseudocount(0.5)
    assert enrichment == cmp.Enrichment().set_pseudocount(0.5)
    assert enrichment != cmp.Enrichment()
    assert pickle.loads(pickle.dumps(enrichment)) == enrichment
    assert enrichment.set_pseudocount(0.0) == cmp.Enrichment().set_pseudocount(0.0)


@pytest.mark.parametrize("value", [-1.0, float("nan"), float("inf"), -float("inf")])
def test_reaper_pseudocount_rejects_invalid_values(value):
    with pytest.raises(
        RuntimeError, match="Pseudocount must be finite and non-negative"
    ):
        cmp.Enrichment().set_pseudocount(value)
