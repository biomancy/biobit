from collections.abc import Callable

from biobit.analysis import seqproj

__all__ = ["from_seqexp", "to_expind"]


def from_seqexp(
        experiment: seqproj.Experiment, *,
        title_builder: Callable[[seqproj.Experiment], str] | str = "title",
        separator: str = "+"
) -> str:
    """
    Converts a seqproj experiment into a human-readable descriptor that can be used for
    nf-core/rnaseq pipeline design files.

    :param experiment: The seqproj.Experiment object to be converted.
    :param title_builder: A function that converts a seqproj.Experiment object into a human-readable title. If string,
        the title is set to the experiment attribute with the given name.
    :param separator: The string used to separate the experiment index and its title in the resulting descriptor.
    :return: A string descriptor, composed of the experiment index and its title, separated by the specified separator.
    """
    if callable(title_builder):
        title = title_builder(experiment)
    elif isinstance(title_builder, str):
        if title_builder not in experiment.attributes:
            raise ValueError(f"Attribute '{title_builder}' not found in the attributes: {experiment}")
        title = experiment.attributes[title_builder]
    else:
        raise ValueError(f"Invalid title_builder: {title_builder}")

    descriptor = f"{experiment.ind}{separator}{title}"
    for char in [",", "|", "/", "[", "]"]:
        if char in descriptor:
            raise ValueError(f"Descriptor '{descriptor}' contains prohibited character '{char}'")

    return descriptor


def to_expind(descriptor: str, separator: str = "+") -> str:
    """
    Extracts the experiment index from a descriptor string.

    :param descriptor: The descriptor string from which to extract the experiment index.
    :param separator: The string used to separate the experiment index and the title in the descriptor.
    :return: A string representing the experiment index.
    """
    return descriptor.split(separator)[0]
