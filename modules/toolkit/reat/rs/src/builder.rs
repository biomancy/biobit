use crate::core::mismatches::prefilters;
use crate::core::mismatches::prefilters::RetainSitesFromIntervals;
use crate::core::ref_fetcher::{FastaRefFetcher, RefFetcher};
use eyre::Result;
use std::path::PathBuf;

pub struct ReadsFilter {
    pub min_mapq: u8,
    pub ban_mapq_255: bool, // 255 = mapping quality is not available
    pub min_phread: u8,
    pub include_flags: u16,
    pub exclude_flags: u16,
}

pub struct EngineBuilder {
    // pub bamfiles: Vec<PathBuf>,
    // pub reference: PathBuf,
    // pub payload: Vec<SiteWorkload>,
    // pub maxwsize: usize,
    pub threads: usize,
    pub reference: Option<Box<dyn RefFetcher>>,
    pub readfilter: Option<ReadsFilter>,
    pub prefilter: prefilters::ByMismatches,
    // pub stranding: REATStrandingEngine<SiteMismatchesVec>,
    pub retain: Option<RetainSitesFromIntervals>,
}

impl EngineBuilder {
    pub fn with_threads(mut self, threads: usize) -> Self {
        self.threads = threads;
        self
    }

    pub fn with_reference(mut self, reference: impl Into<PathBuf>) -> Result<Self> {
        self.reference = Some(Box::new(FastaRefFetcher::new(reference.into().clone())?));
        Ok(self)
    }

    pub fn with_reads_filtering(mut self, filter: ReadsFilter) -> Self {
        self.readfilter = Some(filter);
        self
    }
}

// impl Args {
//     pub fn new(args: &ArgMatches, factory: impl Fn() -> ProgressBar) -> Self {
//         let threads = biobit_core_rs::parallelism::available(-1).unwrap();
//
//
//         let reference: PathBuf = AutoRef::new(Box::new(fasta::BasicFastaReader::new(args.reference.clone())))
//         let refreader = BasicFastaReader::new(reference);
//
//         let stranding = todo!();
//         let readfilter: filters::Sequential<Record, filters::ByQuality, filters::ByFlags> = todo!();
//
//
//         let filter: prefilters::ByMismatches = todo!();
//
//         let mut stranding = REATStrandingEngine::new();
//         let mut payload: Option<Vec<SiteWorkload>> = Default::default();
//         let mut maxsize: Option<usize> = Default::default();
//         let mut retain: Option<RetainSitesFromIntervals> = Default::default();
//
//         let (pbarw, pbars, pbarf) = (factory(), factory(), factory());
//         rayon::scope(|s| {
//             s.spawn(|_| {
//                 // let (w, m) = parse::work(pbarw, &core.bamfiles, core.excluded.take(), args);
//                 // let (w, m) = parse::work(pbarw, &core.bamfiles, args);
//
//                 payload = Some(todo!());
//                 maxsize = Some(todo!())
//             });
//             s.spawn(|_| {
//                 stranding = REATStrandingEngine::new();
//             });
//             s.spawn(|_| retain = None);
//         });
//
//         Self {
//             threads,
//             bamfiles: todo!(),
//             refnucpred: Box::new(AutoRef::new(Box::new(refreader))),
//             readfilter,
//             // excluded: parse::excluded(factory(), args),
//             payload: payload.unwrap(),
//             maxwsize: maxsize.unwrap(),
//             prefilter: filter,
//             stranding,
//             retain,
//             saveto: todo!(),
//         }
//     }
// }
