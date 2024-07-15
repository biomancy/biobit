import gzip
import tempfile
from pathlib import Path

from biobit.resources.repmasker import RepmaskerClassification


def test_repmasker_classification():
    # Create a temporary file with some test data
    with tempfile.NamedTemporaryFile(delete=False, mode='w') as temp:
        temp.close()
        path = Path(temp.name)
        with gzip.open(path, "wt") as stream:
            stream.write("name1 class1 family1\n")
            stream.write("name2 class2 family2\n")
            stream.write("name3 class3 family3\n")

        # Initialize the RepmaskerClassification with the temporary file
        repmasker = RepmaskerClassification(path)

        # Test the classify method
        assert repmasker.classify("name1") == ("name1", "family1", "class1")
        assert repmasker.classify("name2") == ("name2", "family2", "class2")
        assert repmasker.classify("name3") == ("name3", "family3", "class3")
        assert repmasker.classify("name4") is None  # Test with a non-existent key

        # Test the names method
        assert repmasker.names() == {"name1", "name2", "name3"}

        # Test the families method
        assert repmasker.families() == {"family1", "family2", "family3"}

        # Test the classes method
        assert repmasker.classes() == {"class1", "class2", "class3"}


def test_repmasker_classification_empty():
    # Create a temporary file with no data
    with tempfile.NamedTemporaryFile(delete=False, mode='w') as temp:
        temp.close()

        # Initialize the RepmaskerClassification with the empty file
        repmasker = RepmaskerClassification(Path(temp.name))

        # Test the classify method with an empty file
        assert repmasker.classify("name1") is None

        # Test the names method with an empty file
        assert repmasker.names() == set()

        # Test the families method with an empty file
        assert repmasker.families() == set()

        # Test the classes method with an empty file
        assert repmasker.classes() == set()
