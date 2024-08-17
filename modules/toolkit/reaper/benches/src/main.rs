use std::string::ToString;

use biobit_core_rs::loc::Orientation;
use biobit_core_rs::parallelism;
use biobit_core_rs::source::Source;
use biobit_io_rs::bam::{strdeductor, transform, ReaderBuilder};
use biobit_reaper_rs as reaper;
use rayon::ThreadPoolBuilder;

const THREADS: isize = -1;
const QUERY: &[(&str, usize, usize)] = &[
    // GRCh38
    ("1", 0, 248956422),
    ("2", 0, 242193529),
    ("3", 0, 198295559),
    ("4", 0, 190214555),
    ("5", 0, 181538259),
    ("6", 0, 170805979),
    ("7", 0, 159345973),
    ("8", 0, 145138636),
    ("9", 0, 138394717),
    ("10", 0, 133797422),
    ("11", 0, 135086622),
    ("12", 0, 133275309),
    ("13", 0, 114364328),
    ("14", 0, 107043718),
    ("15", 0, 101991189),
    ("16", 0, 90338345),
    ("17", 0, 83257441),
    ("18", 0, 80373285),
    ("19", 0, 58617616),
    ("20", 0, 64444167),
    ("21", 0, 46709983),
    ("22", 0, 50818468),
    ("MT", 0, 16569),
    ("X", 0, 156040895),
    ("Y", 0, 57227415),
    // GRCm39
    // ("1", 0, 195154279),
    // ("2", 0, 181755017),
    // ("3", 0, 159745316),
    // ("4", 0, 156860686),
    // ("5", 0, 151758149),
    // ("6", 0, 149588044),
    // ("7", 0, 144995196),
    // ("8", 0, 130127694),
    // ("9", 0, 124359700),
    // ("10", 0, 130530862),
    // ("11", 0, 121973369),
    // ("12", 0, 120092757),
    // ("13", 0, 120883175),
    // ("14", 0, 125139656),
    // ("15", 0, 104073951),
    // ("16", 0, 98008968),
    // ("17", 0, 95294699),
    // ("18", 0, 90720763),
    // ("19", 0, 61420004),
    // ("MT", 0, 16299),
    // ("X", 0, 169476592),
    // ("Y", 0, 91455967),
];

const SOURCES: &[(&str, &[&str])] = &[
    (
        "RNase",
        &[
            "/home/alnfedorov/projects/biobit/resources/bam/F1+THP-1_EMCV_RNase_3.bam",
            "/home/alnfedorov/projects/biobit/resources/bam/G2+Calu-3_SARS-CoV-2_RNase_3.bam",
        ],
    ),
    (
        "Input",
        &["/home/alnfedorov/projects/biobit/resources/bam/A1+THP-1_mock_no-RNase_2.bam"],
    ),
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
        for path in paths.into_iter() {
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
            .unwrap()
            .set_boundaries(
                Orientation::Forward,
                vec![1_000, 10_000, 100_000, 1_000_000, 10_000_000],
            );

        let config = reaper::Config::new(model, enrichment, pcalling, nms);
        let mut workload = reaper::Workload::new();

        // Queries
        for (contig, start, end) in QUERY {
            workload.add_region(contig.to_string(), *start, *end, config.clone());
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
                region.segment(),
                region.orientation()
            );

            for peak in region.filtered_peaks() {
                println!("\t\t{} {} {}", peak.segment(), peak.signal(), peak.summit())
            }
        }
        println!()
    }
}
