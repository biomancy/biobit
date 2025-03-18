import pickle


def ensure_pickable(obj, *fields: str):
    pickled = pickle.loads(pickle.dumps(obj))
    assert obj == pickled
    assert obj is not pickled

    for field in fields:
        left, right = getattr(obj, field), getattr(pickled, field)
        assert left == right
        assert left is not right or (isinstance(left, (int, str, None)) and left is right)
