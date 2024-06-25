import pytest

from biobit.ds.gindex import Overlap
from biobit.core import Range
from .resolve import _Chain, _Lambda, Proportional, Binary, Priority, Normalize


# Test ResolutionChain
def test_resolution_chain():
    # Define two simple resolutions
    res1 = _Lambda(lambda x: x + 1)
    res2 = _Lambda(lambda x: x * 2)

    # Chain them
    chain = _Chain(res1, res2)

    # Test
    assert chain(2) == 6  # (2 + 1) * 2
    assert res1.apply(res2)(2) == 6
    assert (res1 | res2)(2) == 6


# Test ResolutionLambda
def test_resolution_lambda():
    # Define a simple resolution
    res = _Lambda(lambda x: x + 1)

    def fn(x):
        return x * 2

    # Test
    assert res(2) == 3
    assert res.apply(fn)(2) == 6
    assert (res | fn)(2) == 6


# Test Proportional
def test_proportional():
    # Define a simple proportional resolution
    prop = Proportional(empty="None")

    # Test with no overlaps
    assert prop([]) == {"None": 1}
    assert prop([Overlap(Range(0, 10), [], [])]) == {"None": 10}

    # Test with a single overlap
    overlaps = [Overlap(Range(0, 10), [Range(0, 7)], ['a'])]
    assert prop(overlaps) == {"a": 7, "None": 3}

    overlaps = [Overlap(Range(0, 7), [Range(0, 7)], ['a'])]
    assert prop(overlaps) == {"a": 7}

    # Test with several overlaps
    overlaps = [
        Overlap(Range(0, 10), [Range(0, 7), Range(3, 9)], ['a', 'b']),
        Overlap(Range(11, 15), [Range(14, 15)], ['a']),
    ]
    weights = {"a": 8, "b": 6, "None": 4}
    assert prop(overlaps) == weights


def test_binary():
    binary = Binary(empty=None)

    # Test with no overlaps
    assert binary([]) == {None: 1}
    assert binary([Overlap(Range(0, 10), [], [])]) == {None: 1}

    # Test with a single overlap
    overlaps = [Overlap(Range(0, 10), [Range(0, 7)], ['a'])]
    assert binary(overlaps) == {"a": 1, None: 1}

    overlaps = [Overlap(Range(0, 7), [Range(0, 7)], ['a'])]
    assert binary(overlaps) == {"a": 1}

    # Test with several overlaps
    overlaps = [
        Overlap(Range(0, 10), [Range(0, 7), Range(3, 9)], ['a', 'b']),
        Overlap(Range(11, 15), [Range(14, 15)], ['a']),
    ]
    assert binary(overlaps) == {"a": 1, "b": 1, None: 1}


def test_priority():
    priority = Priority(['a', 'b'])

    # Test unknown category
    for weights in [{}, {'c': 1}]:
        with pytest.raises(ValueError):
            priority(weights)

    # Test with a single category
    assert priority({'a': 1}) == {'a': 1}

    # Test with several categories
    weights = {'a': 0.5, 'b': 0.5}
    assert priority(weights) == {'a': 0.5}


def test_normalize():
    normalize = Normalize(total=10)

    # Test with no weights
    assert normalize({}) == {}

    # Test with a single weight
    assert normalize({'a': 12}) == {'a': 10}
    assert normalize({'a': 10}) == {'a': 10}

    # Special case
    with pytest.raises(ValueError):
        normalize({'a': 0})

    # Test with several weights
    weights = {'a': 50, 'b': 25, 'c': 25}
    assert normalize(weights) == {'a': 5, 'b': 2.5, 'c': 2.5}

    normalize.total = 1
    weights = {'a': 1, 'b': 2, 'c': 3}
    assert normalize(weights) == {'a': 1 / 6, 'b': 2 / 6, 'c': 3 / 6}


def test_priority_with_key():
    priority = Priority(['a', 'b', 'c'], key=lambda x: x[0])

    # Test unknown category
    for weights in [{}, {'da': 1}]:
        with pytest.raises(ValueError):
            priority(weights)

    # Test with a single category
    assert priority({'a': 1}) == {'a': 1}
    assert priority({('a', 'b'): 1}) == {('a', 'b'): 1}
    assert priority({'ba': 1, 'bc': 2}) == {'ba': 1, 'bc': 2}

    # Test with several categories
    assert priority({('a', 1): 0.5, ('b', 2): 1}) == {('a', 1): 0.5}
    assert priority({'bac': 1, 'ca': 2}) == {'bac': 1}
    assert priority({'bac': 1, 'ca': 2, 'b': 3}) == {'bac': 1, 'b': 3}
