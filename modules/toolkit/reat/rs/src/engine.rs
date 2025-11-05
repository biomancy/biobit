use std::path::PathBuf;

use crate::core::mismatches::prefilters;
use crate::core::mismatches::prefilters::RetainSitesFromIntervals;
use crate::core::payload::Payload;

pub struct Engine {
    pub threads: usize,
    pub bamfiles: Vec<PathBuf>,
    pub reference: PathBuf,
    pub min_phread: u8,

    pub workload: Vec<Payload>,
    pub maxwsize: usize,
    pub prefilter: prefilters::ByMismatches,
    pub stranding: Box<()>,
    pub retain: Option<RetainSitesFromIntervals>,
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
