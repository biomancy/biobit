import pickle

from biobit.toolkit.reaper import cmp


def test_reaper_enrichment():
    enrichment1 = cmp.Enrichment().set_scaling(0.5, 2)
    enrichment2 = cmp.Enrichment().set_scaling(0.5, 2)

    assert enrichment1 == enrichment2

    assert pickle.loads(pickle.dumps(enrichment1)) == enrichment1 == enrichment2
