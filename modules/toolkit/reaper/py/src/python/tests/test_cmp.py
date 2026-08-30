import pickle

import pytest

from biobit.toolkit.reaper import cmp


def test_reaper_enrichment():
    enrichment1 = cmp.Enrichment().set_scaling(0.5, 2)
    enrichment2 = cmp.Enrichment().set_scaling(0.5, 2)

    assert enrichment1 == enrichment2

    assert pickle.loads(pickle.dumps(enrichment1)) == enrichment1 == enrichment2


@pytest.mark.parametrize("signal, control", [(0, 1), (1, 0), (float("nan"), 1), (1, float("inf"))])
def test_reaper_enrichment_rejects_invalid_scaling(signal, control):
    with pytest.raises(RuntimeError):
        cmp.Enrichment().set_scaling(signal, control)
