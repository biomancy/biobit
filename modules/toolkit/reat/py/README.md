# biobit-reat-py

Python bindings for the Rust `biobit-reat-rs` toolkit crate.

## Behavioral Notes

- `Reat.add_sources(tag, sources, layout)` appends to an existing sample when `tag` compares equal
  to a previously registered tag. It does not replace the old sources.
- `Reat.run(tasks)` leaves registered samples and sources intact. Call `reset()` to clear them.
- Results are returned in first-registration order for sample tags.
- Unstranded layouts produce dual (`Orientation.Dual`, `"="`) pileups. Forward/reverse layouts use
  the standard biobit NGS orientation deduction for the supplied `Layout`.
- `min_phred` filters aligned read bases before counting. Deletions are counted from CIGAR
  operations rather than from base qualities.
- Task coordinates are stored as unsigned 64-bit Rust coordinates. Python interval accessors use
  `biobit.core.loc.Interval`, which is signed; practical Python-facing coordinates should fit in
  `i64`.
- `Task(seqid, intervals)` expects non-empty, sorted, non-overlapping intervals. Use
  `Task.from_intervals(...)` for arbitrary input.
- `Task.from_intervals(...)` groups by sequence ID, sorts sequence IDs, merges overlapping and
  touching intervals, and splits large merged intervals according to `max_task_size`.
- REAT fetches a task's full envelope, then excludes selected positions outside the task's retained
  intervals. This is why one task can contain multiple disjoint intervals.
- `RequiredSites` is strict about orientation. A required interval registered for `+` does not match
  `-` or `=` pileups; `=` only matches dual/unstranded pileups.
- Required intervals for the same `(seqid, orientation)` are sorted and merged. Empty interval lists
  are ignored.
- Required sites can only produce output for orientations that have an initialized pileup from reads
  in the task envelope. They can select zero-coverage positions inside an initialized pileup, but
  they do not synthesize a completely missing orientation.
- `SparsePileup.interval` spans the first through last selected position and may include unselected
  positions between them. Use `positions` for the exact selected coordinates.
- `SelectedPileup.pileups()` returns a newly built dictionary keyed by `(seqid, Orientation)`.
  Mutating that dictionary does not mutate the result.
- Pickling uses the core bitcode helpers through `__reduce__`; sample tags in `SelectedPileup` are
  pickled as normal Python objects, while pileup payloads are bitcode-encoded.
