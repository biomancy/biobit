Resources in this directory are either managed by git and committed to the repository, or are too large or numerous to
include and are downloaded from external sources.

Compressed files are always identical to their uncompressed versions and are therefore not listed here.

## Git-stored resources

* `bed`
    * `example.bed`: An example BED12 file containing four intervals.
* `fasta`
    * `example.fasta`: Contains two sequences designed to test basic validation and robustness of FASTA parsing.
    * `indexed.fa`: Contains four protein sequences from the UniProt database, used for testing FASTA parsing and
      random-access retrieval.
    * For the indexed FASTA file, the following additional files are generated:
        * `{fasta}.fai`: The FASTA index for `{fasta}.fa`, enabling efficient random access.
        * `{fasta}.fa.bgz.gzi`: The BGZF index for `{fasta}.fa.bgz`, enabling random access for BGZF-compressed FASTA
          files.

## External resources

None at this time.
