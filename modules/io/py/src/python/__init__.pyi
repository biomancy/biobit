from . import bam as bam
from . import bed as bed
from . import fasta as fasta
from . import protocols as protocols
from .protocols import ReadRecord as ReadRecord

__all__ = ["ReadRecord", "bam", "bed", "fasta"]
