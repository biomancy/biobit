from collections.abc import Callable

from biobit.analysis import seqproj

__all__ = ["from_bioexp", "to_expind"]


def from_bioexp(
        experiment: seqproj.Experiment, *,
        title_builder: Callable[[seqproj.Experiment], str] | str | None = None,
        separator: str = "+"
) -> str:
    """
    Converts a bioproj experiment into a human-readable descriptor that can be used for
    nf-core/rnaseq pipeline design files.

    :param experiment: The bioproj.Experiment object to be converted.
    :param title_builder: A function that converts a bioproj.Experiment object into a human-readable title. If string,
        the title is set to the attribute with the given name. If None, the title is built by concatenating all
        experiment sample attributes. If the sample has no attributes, the sample description is used as the title.
    :param separator: The string used to separate the experiment index and its title in the resulting descriptor.
    :return: A string descriptor, composed of the experiment index and its title, separated by the specified separator.
    """
    if callable(title_builder):
        title = title_builder(experiment)
    elif isinstance(title_builder, str):
        if title_builder not in experiment.sample.attributes:
            raise ValueError(f"Attribute '{title_builder}' not found in the sample attributes: {experiment.sample}")
        title = experiment.sample.attributes[title_builder]
    else:
        attributes: list[str] = []
        for atrribute, value in sorted(experiment.sample.attributes.items()):
            attributes.append(f"{atrribute}={value}")

        # If no tags are present, use the sample description as a descriptor
        if len(attributes) == 0:
            if experiment.sample.description is None:
                raise ValueError("No tags or description available for the sample.")
            attributes.append(experiment.sample.description)

        title = "_".join(attributes)

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
