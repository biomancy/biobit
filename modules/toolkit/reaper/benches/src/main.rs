use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::string::ToString;

use biobit_core_rs::loc::{ChainInterval, Interval, Orientation};
use biobit_core_rs::parallelism;
use biobit_core_rs::source::Source;
use biobit_io_rs::bam::{strdeductor, transform, ReaderBuilder};
use biobit_reaper_rs as reaper;
use rayon::ThreadPoolBuilder;

const THREADS: isize = 0;
const QUERY: &[(&str, usize, usize)] = &[
    // CHM13v2
    ("chr1", 0, 248387328),
    ("chr2", 0, 242696752),
    ("chr3", 0, 201105948),
    ("chr4", 0, 193574945),
    ("chr5", 0, 182045439),
    ("chr6", 0, 172126628),
    ("chr7", 0, 160567428),
    ("chr8", 0, 146259331),
    ("chr9", 0, 150617247),
    ("chr10", 0, 134758134),
    // ("chr11", 0, 135127769),
    // ("chr12", 0, 133324548),
    // ("chr13", 0, 113566686),
    // ("chr14", 0, 101161492),
    // ("chr15", 0, 99753195),
    // ("chr16", 0, 96330374),
    // ("chr17", 0, 84276897),
    // ("chr18", 0, 80542538),
    // ("chr19", 0, 61707364),
    // ("chr20", 0, 66210255),
    // ("chr21", 0, 45090682),
    // ("chr22", 0, 51324926),
    // ("chrX", 0, 154259566),
    // ("chrY", 0, 62460029),
    // ("chrM", 0, 16569),
];

const TRANSCRIPTOME_MODEL: &str =
    "/home/alnfedorov/projects/biobit/resources/bed/CHM13v2_models.bed";

const SOURCES: &[(&str, &[&str])] = &[
    ("RNase", &[
        "/home/alnfedorov/projects/biobit/resources/bam/F1+THP-1_EMCV_RNase_3.markdup.sorted.bam",
    ]),
    ("Input", &[
        "/home/alnfedorov/projects/biobit/resources/bam/G1+THP-1_EMCV_no-RNase_3.markdup.sorted.bam"
    ]),
];

const COMPARISONS: &[(&str, &str, &str)] = &[("RNase vs Input", "RNase", "Input")];

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    let threads = parallelism::available(THREADS).unwrap();
    let pool = ThreadPoolBuilder::new()
        .num_threads(threads)
        .use_current_thread()
        .build()
        .unwrap();

    let mut rp = reaper::Reaper::new(pool);

    // Sources
    for (name, paths) in SOURCES {
        for path in *paths {
            let source = ReaderBuilder::new(path)
                .with_inflags(2)
                .with_exflags(2572)
                .with_minmapq(0)
                .build()
                .unwrap();

            let source = source
                .with_transform(transform::BundleMates::default(), ())
                .with_transform(
                    transform::ExtractPairedAlignmentSegments::new(
                        strdeductor::deduce::pe::reverse,
                    ),
                    (),
                );

            rp.add_source(name.to_string(), source);
        }
    }

    // Transcriptome model
    let mut trmodel = HashMap::new();
    for line in BufReader::new(File::open(TRANSCRIPTOME_MODEL).unwrap()).lines() {
        let line = line.unwrap();
        let fields: Vec<&str> = line.split('\t').collect();
        let contig = fields[0];
        let allstart = fields[1].parse::<usize>().unwrap();
        let orientation = Orientation::try_from(fields[5].parse::<char>().unwrap()).unwrap();

        // Parse the rest of BED12 fields here
        let sizes = fields[10].split(',').collect::<Vec<&str>>();
        let starts = fields[11].split(',').collect::<Vec<&str>>();
        let mut intervals = Vec::new();
        for (size, start) in sizes.iter().zip(starts.iter()) {
            let size = size.parse::<usize>().unwrap();
            let start = start.parse::<usize>().unwrap();
            intervals.push(Interval::new(allstart + start, allstart + start + size).unwrap());
        }

        let chain = ChainInterval::try_from_iter(intervals.into_iter()).unwrap();

        trmodel
            .entry((contig.to_string(), orientation))
            .or_insert_with(Vec::new)
            .push(chain);
    }

    // Comparisons
    for (name, signal, control) in COMPARISONS {
        let mut model = reaper::model::RNAPileup::new();
        model
            .set_sensitivity(1e-3f32)
            .set_control_baseline(1e-3)
            .set_min_signal(10.0);

        let mut enrichment = reaper::cmp::Enrichment::new();
        enrichment.set_scaling(1.0, 1.0);

        let mut pcalling = reaper::pcalling::ByCutoff::new();
        pcalling
            .set_cutoff(4.0)
            .set_min_length(25)
            .set_merge_within(25);

        let mut nms = reaper::postfilter::NMS::new();
        nms.set_sloplim(100, 1_000)
            .unwrap()
            .set_fecutoff(2.0)
            .unwrap()
            .set_group_within(1_000)
            .unwrap();

        // Workload
        let mut workload = reaper::Workload::new();
        for (seqid, start, end) in QUERY {
            // Attach the transcriptome model
            let mut seqmodel = model.clone();
            let mut seqnms = nms.clone();

            for orientation in [Orientation::Forward, Orientation::Reverse] {
                if let Some(chains) = trmodel.remove(&(seqid.to_string(), orientation)) {
                    let control = reaper::model::ControlModel::new(
                        chains.clone(),
                        false,
                        vec![256, 512, 1024],
                    )
                    .unwrap();
                    seqmodel.add_modeling(orientation, control);

                    // let nms = reaper::postfilter::NMSRegions::new(chains, false).unwrap();
                    let nms = reaper::postfilter::NMSRegions::new(chains, true).unwrap();
                    seqnms.add_regions(orientation, nms);
                }
            }

            let config =
                reaper::Config::new(seqmodel, enrichment.clone(), pcalling.clone(), seqnms);

            workload.add_region(seqid.to_string(), *start, *end, config);
        }

        rp.add_comparison(
            name.to_string(),
            &signal.to_string(),
            &control.to_string(),
            workload,
        )
        .unwrap();
    }

    // Run the countit
    let result = {
        #[cfg(feature = "dhat-heap")]
        let _profiler = dhat::Profiler::new_heap();
        rp.run().unwrap()
    };

    // Print the result
    for r in result {
        println!("Comparison: {}", r.comparison());

        for region in r.regions() {
            println!(
                "\tRegion: {}:{}[{}]",
                region.contig(),
                region.interval(),
                region.orientation()
            );

            for peak in region.filtered_peaks() {
                println!(
                    "\t\t{} {} {}",
                    peak.interval(),
                    peak.signal(),
                    peak.summit()
                )
            }
        }
        println!()
    }
}
