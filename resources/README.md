* `bed`
    * `example.bed`: An example BED12 file containing 4 intervals.
* `fasta`
    * `example.fasta`: Contains 2 sequences with characteristics designed to test the basic validation and robustness of
      FASTA parsing.
    * `indexed.fa`: Contains 4 protein sequences from the UniProt database. Used for testing FASTA parsing and
      random-access retrieval.
    * For the indexed FASTA file, the following additional files are generated:
        * `{fasta}.fai`: The FASTA index associated with `{fasta}.fa`, enabling efficient random access.
        * `{fasta}.gz.gzi`: The BGZF index associated with `{fasta}.fa.bgz`, enabling random access for BGZF-compressed
          FASTA files.

Compressed files are always identical to their uncompressed versions, so they are not listed here.
