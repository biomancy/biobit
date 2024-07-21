use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

use flate2::bufread::MultiGzDecoder;
use rayon::ThreadPoolBuilder;

use biobit_core_rs::{loc::Orientation, parallelism};
use biobit_core_rs::loc::{Locus, Segment};
use biobit_core_rs::source::Source;
use biobit_countit_rs::CountIt;
use biobit_io_rs::bam::{ReaderBuilder, strdeductor, transform};

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn read_bed() -> Vec<(String, Vec<(String, Orientation, Vec<Segment<usize>>)>)> {
    let reader =
        File::open("/home/alnfedorov/projects/biobit/resources/bed/GRCh38.bed.gz").unwrap();
    let reader = BufReader::new(reader);
    let mut reader = BufReader::new(MultiGzDecoder::new(reader));

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
    let threads = parallelism::available(-1).unwrap();
    let pool = ThreadPoolBuilder::new()
        .num_threads(threads)
        .use_current_thread()
        .build()
        .unwrap();

    let mut countit: CountIt<String, usize, f64, String, String, _> = CountIt::new(pool);
    let workload = [
        // GRCh38
        ("1", 248956422),
        ("2", 242193529),
        ("3", 198295559),
        ("4", 190214555),
        ("5", 181538259),
        ("6", 170805979),
        ("7", 159345973),
        ("8", 145138636),
        ("9", 138394717),
        ("10", 133797422),
        ("11", 135086622),
        ("12", 133275309),
        ("13", 114364328),
        ("14", 107043718),
        ("15", 101991189),
        ("16", 90338345),
        ("17", 83257441),
        ("18", 80373285),
        ("19", 58617616),
        ("20", 64444167),
        ("21", 46709983),
        ("22", 50818468),
        ("MT", 16569),
        ("X", 156040895),
        ("Y", 57227415),
        // GRCm39
        // ("1", 195154279),
        // ("2", 181755017),
        // ("3", 159745316),
        // ("4", 156860686),
        // ("5", 151758149),
        // ("6", 149588044),
        // ("7", 144995196),
        // ("8", 130127694),
        // ("9", 124359700),
        // ("10", 130530862),
        // ("11", 121973369),
        // ("12", 120092757),
        // ("13", 120883175),
        // ("14", 125139656),
        // ("15", 104073951),
        // ("16", 98008968),
        // ("17", 95294699),
        // ("18", 90720763),
        // ("19", 61420004),
        // ("MT", 16299),
        // ("X", 169476592),
        // ("Y", 91455967),
    ];

    for (contig, size) in workload {
        countit.add_partition(Locus::new(
            contig.to_string(),
            Segment::new(0, size).unwrap(),
            Orientation::Dual,
        ))
    }

    // Annotation from the BAM file
    for (item, items) in read_bed() {
        countit.add_annotation(
            item,
            items
                .into_iter()
                .map(|(name, orientation, segments)| (name, orientation, segments.into_iter())),
        );
    }

    for path in [
        "/home/alnfedorov/projects/biobit/resources/bam/A1+THP-1_mock_no-RNase_2.bam",
        "/home/alnfedorov/projects/biobit/resources/bam/F1+THP-1_EMCV_RNase_3.bam",
        "/home/alnfedorov/projects/biobit/resources/bam/G2+Calu-3_SARS-CoV-2_RNase_3.bam",
        // "/home/alnfedorov/projects/Z-DoTT/stories/nextflow/series/internal/B287138/results/star_salmon/2518960+MEF_Vector_1.markdup.sorted.bam",
        // "/home/alnfedorov/projects/Z-DoTT/stories/nextflow/series/internal/B287138/results/star_salmon/2518961+MEF_Vector_2.markdup.sorted.bam",
        // "/home/alnfedorov/projects/Z-DoTT/stories/nextflow/series/internal/B287138/results/star_salmon/2518962+MEF_ICP27_1.markdup.sorted.bam",
        // "/home/alnfedorov/projects/Z-DoTT/stories/nextflow/series/internal/B287138/results/star_salmon/2518963+MEF_ICP27_2.markdup.sorted.bam",
        // "/home/alnfedorov/projects/Z-DoTT/stories/nextflow/series/internal/B287138/results/star_salmon/2518964+MEF_ICP27-m15_1.markdup.sorted.bam",
        // "/home/alnfedorov/projects/Z-DoTT/stories/nextflow/series/internal/B287138/results/star_salmon/2518965+MEF_ICP27-m15_2.markdup.sorted.bam",
        // "/home/alnfedorov/projects/Z-DoTT/stories/nextflow/series/internal/B287138/results/star_salmon/2518966+MEF_ICP27-n504_1.markdup.sorted.bam",
        // "/home/alnfedorov/projects/Z-DoTT/stories/nextflow/series/internal/B287138/results/star_salmon/2518967+MEF_ICP27-n504_2.markdup.sorted.bam",
    ] {
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

        countit.add_source(path.to_string(), source);
    }

    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    countit.run().unwrap();
}
