use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::string::ToString;

use rayon::ThreadPoolBuilder;

use biobit_core_rs::loc::Segment;
use biobit_core_rs::source::Source;
use biobit_core_rs::{loc::Orientation, parallelism};
use biobit_countit_rs;
use biobit_io_rs::bam::{strdeductor, transform, ReaderBuilder};

const THREADS: isize = -1;
const BED: &str = "/home/alnfedorov/projects/biobit/resources/bed/manual.bed";

const PARTITIONS: &[(&str, usize, usize)] = &[
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

const BAM: &[&str] = &[
    "/home/alnfedorov/projects/biobit/resources/bam/A1+THP-1_mock_no-RNase_2.bam",
    // "/home/alnfedorov/projects/biobit/resources/bam/F1+THP-1_EMCV_RNase_3.bam",
    // "/home/alnfedorov/projects/biobit/resources/bam/G2+Calu-3_SARS-CoV-2_RNase_3.bam",
    // "/home/alnfedorov/projects/Z-DoTT/stories/nextflow/series/internal/B287138/results/star_salmon/2518960+MEF_Vector_1.markdup.sorted.bam",
    // "/home/alnfedorov/projects/Z-DoTT/stories/nextflow/series/internal/B287138/results/star_salmon/2518961+MEF_Vector_2.markdup.sorted.bam",
    // "/home/alnfedorov/projects/Z-DoTT/stories/nextflow/series/internal/B287138/results/star_salmon/2518962+MEF_ICP27_1.markdup.sorted.bam",
    // "/home/alnfedorov/projects/Z-DoTT/stories/nextflow/series/internal/B287138/results/star_salmon/2518963+MEF_ICP27_2.markdup.sorted.bam",
    // "/home/alnfedorov/projects/Z-DoTT/stories/nextflow/series/internal/B287138/results/star_salmon/2518964+MEF_ICP27-m15_1.markdup.sorted.bam",
    // "/home/alnfedorov/projects/Z-DoTT/stories/nextflow/series/internal/B287138/results/star_salmon/2518965+MEF_ICP27-m15_2.markdup.sorted.bam",
    // "/home/alnfedorov/projects/Z-DoTT/stories/nextflow/series/internal/B287138/results/star_salmon/2518966+MEF_ICP27-n504_1.markdup.sorted.bam",
    // "/home/alnfedorov/projects/Z-DoTT/stories/nextflow/series/internal/B287138/results/star_salmon/2518967+MEF_ICP27-n504_2.markdup.sorted.bam",
];

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn read_bed(path: &Path) -> Vec<(String, Vec<(String, Orientation, Vec<Segment<usize>>)>)> {
    let reader = File::open(path).unwrap();
    let mut reader = BufReader::new(reader);
    // let mut reader = BufReader::new(MultiGzDecoder::new(reader));

    let mut records = HashMap::new();
    let mut buf = String::new();
    while reader.read_line(&mut buf).expect("Failed to read BED file") != 0 {
        let line = buf.trim_end();
        if line.is_empty() {
            buf.clear();
            continue;
        }
        let split: Vec<&str> = line.split('\t').take(6).collect();
        assert!(split.len() >= 3);

        let start = split[1].parse().expect("Failed to filters string start");
        let end = split[2].parse().expect("Failed to filters string start");
        assert!(end > start, "{}", line);
        let segment = Segment::new(start, end).expect("Failed to create segment");

        let name = split.get(3).unwrap_or(&"").to_string();
        let orientation = Orientation::try_from(*split.get(5).unwrap()).unwrap();
        let contig = split.get(0).unwrap().to_string();

        records
            .entry(name)
            .or_insert_with(HashMap::new)
            .entry((contig, orientation))
            .or_insert_with(Vec::new)
            .push(segment);

        buf.clear();
    }

    records
        .into_iter()
        .map(|(name, items)| {
            let items = items
                .into_iter()
                .map(|((name, orientation), segments)| (name, orientation, segments))
                .collect();
            (name, items)
        })
        .collect()
}

fn main() {
    let threads = parallelism::available(THREADS).unwrap();
    println!("Threads: {}", threads);
    let pool = ThreadPoolBuilder::new()
        .num_threads(threads)
        .use_current_thread()
        .build()
        .unwrap();

    let mut builder = biobit_countit_rs::rigid::Engine::<String, usize, f64, String>::builder();
    builder = builder.set_thread_pool(pool);
    builder = builder.add_partitions(
        PARTITIONS
            .iter()
            .map(|(contig, start, end)| (contig.to_string(), Segment::new(*start, *end).unwrap())),
    );
    builder = builder.add_elements(read_bed(Path::new(BED)).into_iter());
    let mut engine = builder.build::<f64>();
    println!("Engine constructed");

    let mut sources = Vec::new();
    for path in BAM {
        let source = ReaderBuilder::new(path)
            .with_inflags(2)
            .with_exflags(2572)
            .with_minmapq(0)
            .build()
            .unwrap();

        let source = source
            .with_transform(transform::BundleMates::default(), ())
            .with_transform(
                transform::ExtractPairedAlignmentSegments::new(strdeductor::deduce::pe::reverse),
                (),
            );
        sources.push((path.to_string(), source));
    }
    // let resolution = biobit_countit_rs::rigid::resolution::OverlapWeighted::new(true);
    // let resolution = biobit_countit_rs::rigid::resolution::AnyOverlap::new(true);

    let priorities = vec![
        "1:81334878-81334926".to_string(),
        "1:81991464-81992577".to_string(),
        "1:81991464-81992577".to_string(),
        "1:81979818-81980040".to_string(),
    ];
    let resolution = biobit_countit_rs::rigid::resolution::TopRanked::new(
        move |mut ranks: Vec<usize>, _elements: &[String]| {
            let ranking: HashMap<&String, usize> =
                priorities.iter().enumerate().map(|(i, p)| (p, i)).collect();

            ranks.clear();
            for element in _elements {
                let rank = ranking.get(element).copied().unwrap();
                ranks.push(rank);
            }
            ranks
        },
        true,
    );

    // Run the countit
    let result = {
        #[cfg(feature = "dhat-heap")]
        let _profiler = dhat::Profiler::new_heap();
        engine
            .run(sources.into_iter(), Box::new(resolution))
            .unwrap()
    };

    // Print the result
    for r in result {
        println!("Source: {}", r.source);
        for (obj, cnt) in r.elements.iter().zip(r.counts) {
            println!("\t{}: {}", obj, cnt);
        }
        println!("stats:");
        for p in r.partitions {
            println!(
                "\t{:<3}:{:<25}\t{}\t{}",
                p.contig, p.segment, p.outcomes.resolved, p.outcomes.discarded
            )
        }
        println!()
    }
}
