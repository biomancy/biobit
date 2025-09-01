Resources in this directory are either managed by git and committed to the repository, or are too large or numerous to
include and are downloaded from external sources.

Compressed files are always identical to their uncompressed versions and are therefore not listed here.

## Git-stored resources

* `bam`
    * `RNA-seq.CHM13v2.21-22.bam`: A BAM file containing a subsampled RNA-seq dataset from the CHM13v2 assembly for
      chromosomes 21 and 22.
    * All BAM files are indexed with `samtools index` using both the `.bai` and `.csi` formats.
* `bed`
    * `example.bed`: An example BED12 file containing four intervals.
    * `gencode-basic-47.CHM13v2.bed.bgz`: A BGZF-compressed BED file containing the GENCODE Basic v47 annotations for
      the full CHM13v2 assembly.
    * All BGZF-compressed BED files are indexed with `tabix` using the `.tbi` format.
* `fasta`
    * `example.fasta`: Contains two sequences designed to test basic validation and robustness of FASTA parsing.
    * `indexed.fa`: Indexed fasta with four protein sequences from the UniProt database.
    * `CHM13v2.M-21-22.fa`: CHM13v2 assembly for mitochondrion and chromosomes 21 and 22.
    * For the indexed FASTA file, the following additional files are generated:
        * `{fasta}.fai`: The FASTA index for `{fasta}.fa`, enabling efficient random access.
        * `{fasta}.fa.bgz.gzi` & `{fasta}.fa.bgz.fai`: The BGZF index for `{fasta}.fa.bgz`, enabling random access for
          BGZF-compressed FASTA files.

## External resources

None at this time.
