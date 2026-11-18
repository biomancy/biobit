use crate::Alignable;
use crate::pairwise::dp::{
    ChunkSize, Cursor, Scan,
    kernel::LocalAffine,
    layout::Layout,
    tracer::{CellScores, EndpointTracker, Trace, Traceback, Tracer},
};
use crate::pairwise::{alignment, scoring};

struct Tracers<E, M> {
    endpoints: E,
    traceback: M,
}

impl<E: EndpointTracker, M: Traceback<Score = E::Score>> Tracer for Tracers<E, M> {
    type Score = E::Score;

    fn initialize(&mut self, layout: &impl Layout) {
        self.traceback.initialize(layout);
        self.endpoints.initialize(layout);
    }

    #[inline]
    fn chunk<const N: usize>(
        &mut self,
        layout: &impl Layout,
        cursor: &Cursor,
        scores: &[CellScores<Self::Score>; N],
        traces: &[Trace; N],
    ) {
        self.traceback.chunk(layout, cursor, scores, traces);
        self.endpoints.chunk(layout, cursor, scores, traces);
    }

    #[inline]
    fn tail(
        &mut self,
        layout: &impl Layout,
        cursor: &Cursor,
        scores: &[CellScores<Self::Score>],
        traces: &[Trace],
    ) {
        self.traceback.tail(layout, cursor, scores, traces);
        self.endpoints.tail(layout, cursor, scores, traces);
    }

    fn col_start(&mut self, layout: &impl Layout, col: usize) {
        self.traceback.col_start(layout, col);
        self.endpoints.col_start(layout, col);
    }

    fn col_end(&mut self, layout: &impl Layout, col: usize) {
        self.traceback.col_end(layout, col);
        self.endpoints.col_end(layout, col);
    }
}

/// Runs the scalar local affine-gap kernel and reconstructs the alignments selected by its endpoint tracker.
pub struct Engine<E: EndpointTracker, M, Scheme> {
    kernel: LocalAffine<E::Score>,
    scoring: Scheme,
    tracers: Tracers<E, M>,
}

impl<E, M, Scheme> Engine<E, M, Scheme>
where
    E: EndpointTracker,
    M: Traceback<Score = E::Score>,
    Scheme: scoring::Scheme<Score = E::Score>,
{
    pub fn new(endpoints: E, traceback: M, scoring: Scheme) -> Self {
        Self {
            kernel: LocalAffine::new(0),
            scoring,
            tracers: Tracers {
                endpoints,
                traceback,
            },
        }
    }

    /// Recovers the kernel, scoring scheme, endpoint tracker, and traceback storage.
    pub fn into_parts(self) -> (LocalAffine<E::Score>, Scheme, E, M) {
        (
            self.kernel,
            self.scoring,
            self.tracers.endpoints,
            self.tracers.traceback,
        )
    }

    pub fn with_scoring(&mut self, scoring: Scheme) {
        self.scoring = scoring;
    }

    /// Selects the number of rows in a full tracer callback.
    pub fn set_chunk_size(&mut self, size: ChunkSize) {
        self.kernel.set_chunk_size(size);
    }

    pub fn endpoints(&mut self) -> &mut E {
        &mut self.tracers.endpoints
    }

    /// Scores the cells in the scan and traces the endpoints selected by the tracker.
    ///
    /// ```
    /// use biobit_alignment_rs::pairwise::{dp::{Engine, Scan, tracer}, scoring};
    ///
    /// let scan = Scan::above_main_diagonal(b"ACG".as_slice(), b"TTACG".as_slice(), 2)?;
    /// let scores = scoring::compose(
    ///     scoring::symbols::Equality::new(3, -2),
    ///     scoring::gaps::Affine { open: -5, extend: -1 },
    ///     scoring::equiv::Equality {},
    /// );
    /// let mut engine = Engine::new(tracer::Best::new(), tracer::TraceMatrix::new(), scores);
    /// let hits = engine.scan(&scan);
    /// assert_eq!((hits[0].seq1(), hits[0].seq2()), (&(0..3), &(2..5)));
    /// # Ok::<(), biobit_alignment_rs::pairwise::dp::layout::LayoutError>(())
    /// ```
    pub fn scan<S1, S2>(
        &mut self,
        scan: &Scan<S1, S2, impl Layout>,
    ) -> Vec<alignment::Alignment<E::Score, u8, usize, usize>>
    where
        S1: Alignable<Symbol = <Scheme as scoring::Scheme>::Symbol>,
        S2: Alignable<Symbol = S1::Symbol>,
    {
        self.kernel.scan(scan, &mut self.scoring, &mut self.tracers);
        let (seq1, seq2) = scan.sequences();
        self.tracers
            .endpoints
            .finalize()
            .into_iter()
            .map(|x| {
                let trace = self
                    .tracers
                    .traceback
                    .trace(scan.layout(), x.col, x.row)
                    .unwrap();
                debug_assert_eq!(trace.seq1.end, x.row + 1);
                debug_assert_eq!(trace.seq2.end, x.col + 1);
                let ops = alignment::utils::disambiguate(
                    trace.ops,
                    &self.scoring,
                    seq1,
                    trace.seq1.start,
                    seq2,
                    trace.seq2.start,
                );

                alignment::Alignment::new(x.score, ops, trace.seq1, trace.seq2)
            })
            .collect()
    }
}
