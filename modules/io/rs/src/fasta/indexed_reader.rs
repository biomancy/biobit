use biobit_core_rs::loc::{Interval, IntervalOp};
use biobit_core_rs::num::PrimInt;
use derive_getters::{Dissolve, Getters};
use eyre::{Context, Result, ensure, eyre};
use impl_tools::autoimpl;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, Read, Seek};
use std::path::Path;
use substratum_compress::Decoder;

/// An indexed FASTA reader that can fetch sequences by reference sequence ID and interval.
#[autoimpl(for<T: trait + ?Sized> Box<T>)]
pub trait IndexedReaderMutOp {
    /// Fetch the sequence for the given reference sequence ID and interval.
    fn fetch(&mut self, seqid: &str, interval: Interval<u64>, buffer: &mut Vec<u8>) -> Result<()>;

    /// Fetch the full reference sequence with the given ID.
    fn fetch_full_seq(&mut self, seqid: &str, buffer: &mut Vec<u8>) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq, Dissolve, Getters)]
pub struct IndexedReader<R> {
    readers: Vec<R>, // FASTA file readers

    reader_idx: Vec<usize>, // IDs of the reference sequences as they appear in FASTA files
    lengths: Vec<u64>,      // Total length of each reference sequence, in bases
    offsets: Vec<u64>,      // Offset in the FASTA file of the first base of each reference sequence
    bases_per_line: Vec<u64>, // Number of bases per line for each reference sequence
    bytes_per_line: Vec<u64>, // Number of bytes per line for each reference sequence (including line ending character[s])

    index: HashMap<String, usize>, // Map from reference sequence ID to its index in the vectors above
}

trait IndexedRead: Read + Seek + Send + Sync + 'static {}
impl<T: Read + Seek + Send + Sync + 'static> IndexedRead for T {}

impl IndexedReader<()> {
    pub fn from_path(
        fasta: impl AsRef<Path>,
        compression: Decoder,
    ) -> Result<Box<dyn IndexedReaderMutOp + Send + Sync + 'static>> {
        Self::from_paths(&[(fasta, compression)])
    }

    pub fn from_paths(
        indexed: &[(impl AsRef<Path>, Decoder)],
    ) -> Result<Box<dyn IndexedReaderMutOp + Send + Sync + 'static>> {
        let mut parsed: Vec<(Box<dyn IndexedRead>, _)> = Vec::with_capacity(indexed.len());

        for (fasta, compression) in indexed.iter() {
            let mut path = fasta.as_ref().to_owned();
            let file = File::open(&path)?;

            let fname = path
                .file_name()
                .and_then(|x| x.to_str())
                .unwrap_or_default()
                .to_string();
            path.set_file_name(format!("{fname}.fai"));
            ensure!(path.exists(), "fai index does not exist: {:?}", path);
            let fai = std::io::BufReader::new(File::open(&path)?);

            match compression {
                Decoder::Identity(_) => {
                    parsed.push((Box::new(file), fai));
                }
                Decoder::Bgzf(_) => {
                    path.set_file_name(format!("{fname}.gzi"));
                    ensure!(path.exists(), "gzi index does not exist: {:?}", path);
                    let gzi = noodles::bgzf::gzi::fs::read(&path)?;

                    let reader = Box::new(noodles::bgzf::io::indexed_reader::IndexedReader::new(
                        file, gzi,
                    ));
                    parsed.push((reader, fai))
                }
                _ => {
                    return Err(eyre!(
                        "Unsupported compression {:?} for an Indexed FASTA file: {}",
                        compression,
                        path.display()
                    ));
                }
            };
        }

        Ok(Box::new(IndexedReader::new(parsed)?))
    }
}

impl<R: Read + Seek> IndexedReader<R> {
    /// Create a new indexed reader by parsing given FASTA/Index pairs.
    pub fn new<I: BufRead>(mut indexed: Vec<(R, I)>) -> Result<IndexedReader<R>> {
        let mut readers = Vec::with_capacity(indexed.len());

        let mut seqids = HashMap::new();
        let mut reader_idx = Vec::new();
        let mut lengths = Vec::new();
        let mut offsets = Vec::new();
        let mut bases_per_line = Vec::new();
        let mut bytes_per_line = Vec::new();

        let mut buffer = String::new();
        for (reader, mut index) in indexed.drain(..) {
            let ridx = readers.len();
            readers.push(reader);

            while index.read_line(&mut buffer)? > 0 {
                let err = || eyre!("Invalid FASTA index line: {}", buffer);
                let mut parts = buffer.split('\t');

                let id = parts.next().ok_or_else(err)?;
                ensure!(
                    !id.is_empty(),
                    "Reference sequence ID cannot be empty, line: {}",
                    buffer
                );
                match seqids.entry(id.to_string()) {
                    std::collections::hash_map::Entry::Vacant(e) => {
                        e.insert(lengths.len());
                    }
                    std::collections::hash_map::Entry::Occupied(_) => {
                        return Err(eyre!(
                            "Duplicate reference sequence ID in the FASTA index: {}",
                            id
                        ));
                    }
                }

                let length = parts
                    .next()
                    .ok_or_else(err)?
                    .parse::<u64>()
                    .wrap_err_with(err)?;
                ensure!(
                    length > 0,
                    "Length of the reference sequence must be greater than zero, line: {}",
                    buffer
                );
                lengths.push(length);

                let offset = parts
                    .next()
                    .ok_or_else(err)?
                    .parse::<u64>()
                    .wrap_err_with(err)?;
                ensure!(
                    offset > 0,
                    "Offset of the reference sequence must be greater than zero, line: {}",
                    buffer
                );
                offsets.push(offset);

                let _bases_per_line = parts
                    .next()
                    .ok_or_else(err)?
                    .parse::<u64>()
                    .wrap_err_with(err)?;
                ensure!(
                    _bases_per_line > 0,
                    "Bases per line must be greater than zero, line: {}",
                    buffer
                );
                bases_per_line.push(_bases_per_line);

                let _bytes_per_line = parts
                    .next()
                    .ok_or_else(err)?
                    .trim_end_matches(&['\r', '\n'] as &[char])
                    .parse::<u64>()
                    .wrap_err_with(err)?;
                ensure!(
                    _bytes_per_line > _bases_per_line,
                    "Bytes per line must be greater than bases per line, line: {}",
                    buffer
                );
                bytes_per_line.push(_bytes_per_line);

                ensure!(
                    parts.next().is_none(),
                    "Extra fields in the FASTA index, line: {}",
                    buffer
                );

                reader_idx.push(ridx);

                buffer.clear();
            }
        }

        Ok(Self {
            readers,
            reader_idx,
            lengths,
            offsets,
            bases_per_line,
            bytes_per_line,
            index: seqids,
        })
    }

    #[inline(always)]
    fn sanitize<Iv, Idx>(&mut self, seqid: &str, interval: &Iv) -> Result<(usize, u64, u64)>
    where
        Iv: IntervalOp<Idx = Idx>,
        Idx: PrimInt,
    {
        // Find the index of the reference sequence
        let index = self
            .index
            .get(seqid)
            .ok_or_else(|| eyre!("Reference sequence ID not found in the index: {}", seqid))?;

        // Get required coordinates in sequence space
        let start = interval
            .start()
            .to_u64()
            .ok_or_else(|| eyre!("Invalid start coordinate: {:?}", interval.start()))?;
        let end = interval
            .end()
            .to_u64()
            .ok_or_else(|| eyre!("Invalid end coordinate: {:?}", interval.end()))?;

        // Validate the coordinates
        let length = self.lengths[*index];
        ensure!(
            start < length,
            "Start coordinate for {} is out of bounds: {} >= {}",
            seqid,
            start,
            length
        );
        ensure!(
            end <= length,
            "End coordinate for {} is out of bounds: {} > {}",
            seqid,
            end,
            length
        );

        Ok((*index, start, end))
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn fetch(
        reader: &mut R,
        offset: u64,
        start: u64,
        end: u64,
        bytes_per_line: u64,
        bases_per_line: u64,
        endline_bytes: u64,
        buffer: &mut Vec<u8>,
    ) -> Result<()> {
        // Calculate start and end lines in the FASTA file
        let start_line = start / bases_per_line;
        let end_line = end / bases_per_line;

        // Prepare the buffer
        buffer.clear();

        let length = (end - start) as usize;
        if buffer.capacity() < length {
            buffer.try_reserve(length - buffer.capacity())?;
        }

        // Seek to the start of the sequence
        let start_byte = offset + start_line * bytes_per_line + start % bases_per_line;
        reader.seek(std::io::SeekFrom::Start(start_byte))?;

        // Special case – the sequence is contained in a single line
        if start_line == end_line {
            reader.by_ref().take(length as u64).read_to_end(buffer)?;
            return Ok(());
        }

        // Read the first line, which might be incomplete
        let mut sink = std::io::sink();
        {
            let to_read = bases_per_line * (start_line + 1) - start;
            reader.take(to_read).read_to_end(buffer)?;
            std::io::copy(&mut reader.take(endline_bytes), &mut sink)?;
        }

        // Read the middle lines
        for _ in start_line + 1..end_line {
            reader.by_ref().take(bases_per_line).read_to_end(buffer)?;
            std::io::copy(&mut reader.take(endline_bytes), &mut sink)?;
        }

        // Read the last line, which might be incomplete
        reader
            .take(end - end_line * bases_per_line)
            .read_to_end(buffer)?;

        Ok(())
    }

    /// Fetch the sequence for the given reference sequence ID and interval.
    pub fn fetch_interval<Iv, Idx>(
        &mut self,
        seqid: &str,
        interval: &Iv,
        buffer: &mut Vec<u8>,
    ) -> Result<()>
    where
        Iv: IntervalOp<Idx = Idx>,
        Idx: PrimInt,
    {
        let (index, start, end) = self.sanitize(seqid, interval)?;

        let ridx = self.reader_idx[index];
        let reader = &mut self.readers[ridx];
        let offset = self.offsets[index];
        let bases_per_line = self.bases_per_line[index];
        let bytes_per_line = self.bytes_per_line[index];
        let endline_bytes = bytes_per_line - bases_per_line;

        Self::fetch(
            reader,
            offset,
            start,
            end,
            bytes_per_line,
            bases_per_line,
            endline_bytes,
            buffer,
        )?;

        Ok(())
    }

    /// Fetch the full sequence for the given reference sequence ID.
    pub fn fetch_full_seq(&mut self, seqid: &str, buffer: &mut Vec<u8>) -> Result<()> {
        let index = self
            .index
            .get(seqid)
            .ok_or_else(|| eyre!("Reference sequence ID not found in the index: {}", seqid))?;

        let interval = Interval::new(0, self.lengths[*index])?;
        self.fetch_interval(seqid, &interval, buffer)
    }
}

impl<R: Read + Seek> IndexedReaderMutOp for IndexedReader<R> {
    fn fetch(&mut self, seqid: &str, interval: Interval<u64>, buffer: &mut Vec<u8>) -> Result<()> {
        Self::fetch_interval(self, seqid, &interval, buffer)
    }

    fn fetch_full_seq(&mut self, seqid: &str, buffer: &mut Vec<u8>) -> Result<()> {
        Self::fetch_full_seq(self, seqid, buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_indexed_fa() -> Result<()> {
        fn test(mut reader: impl IndexedReaderMutOp) -> Result<()> {
            // Check that we can fetch full sequences
            let sequences = vec![
                (
                    "sp|O95786|RIGI_HUMAN",
                    "MTTEQRRSLQAFQDYIRKTLDPTYILSYMAPWFREEEVQYIQAEKNNKGPMEAATLFLKF\
                     LLELQEEGWFRGFLDALDHAGYSGLYEAIESWDFKKIEKLEEYRLLLKRLQPEFKTRIIP\
                     TDIISDLSECLINQECEEILQICSTKGMMAGAEKLVECLLRSDKENWPKTLKLALEKERN\
                     KFSELWIVEKGIKDVETEDLEDKMETSDIQIFYQEDPECQNLSENSCPPSEVSDTNLYSP\
                     FKPRNYQLELALPAMKGKNTIICAPTGCGKTFVSLLICEHHLKKFPQGQKGKVVFFANQI\
                     PVYEQQKSVFSKYFERHGYRVTGISGATAENVPVEQIVENNDIIILTPQILVNNLKKGTI\
                     PSLSIFTLMIFDECHNTSKQHPYNMIMFNYLDQKLGGSSGPLPQVIGLTASVGVGDAKNT\
                     DEALDYICKLCASLDASVIATVKHNLEELEQVVYKPQKFFRKVESRISDKFKYIIAQLMR\
                     DTESLAKRICKDLENLSQIQNREFGTQKYEQWIVTVQKACMVFQMPDKDEESRICKALFL\
                     YTSHLRKYNDALIISEHARMKDALDYLKDFFSNVRAAGFDEIEQDLTQRFEEKLQELESV\
                     SRDPSNENPKLEDLCFILQEEYHLNPETITILFVKTRALVDALKNWIEGNPKLSFLKPGI\
                     LTGRGKTNQNTGMTLPAQKCILDAFKASGDHNILIATSVADEGIDIAQCNLVILYEYVGN\
                     VIKMIQTRGRGRARGSKCFLLTSNAGVIEKEQINMYKEKMMNDSILRLQTWDEAVFREKI\
                     LHIQTHEKFIRDSQEKPKPVPDKENKKLLCRKCKALACYTADVRVIEECHYTVLGDAFKE\
                     CFVSRPHPKPKQFSSFEKRAKIFCARQNCSHDWGIHVKYKTFEIPVIKIESFVVEDIATG\
                     VQTLYSKWKDFHFEKIPFDPAEMSK",
                ),
                (
                    "sp|P55265|DSRAD_HUMAN",
                    "MNPRQGYSLSGYYTHPFQGYEHRQLRYQQPGPGSSPSSFLLKQIEFLKGQLPEAPVIGKQ\
                     TPSLPPSLPGLRPRFPVLLASSTRGRQVDIRGVPRGVHLRSQGLQRGFQHPSPRGRSLPQ\
                     RGVDCLSSHFQELSIYQDQEQRILKFLEELGEGKATTAHDLSGKLGTPKKEINRVLYSLA\
                     KKGKLQKEAGTPPLWKIAVSTQAWNQHSGVVRPDGHSQGAPNSDPSLEPEDRNSTSVSED\
                     LLEPFIAVSAQAWNQHSGVVRPDSHSQGSPNSDPGLEPEDSNSTSALEDPLEFLDMAEIK\
                     EKICDYLFNVSDSSALNLAKNIGLTKARDINAVLIDMERQGDVYRQGTTPPIWHLTDKKR\
                     ERMQIKRNTNSVPETAPAAIPETKRNAEFLTCNIPTSNASNNMVTTEKVENGQEPVIKLE\
                     NRQEARPEPARLKPPVHYNGPSKAGYVDFENGQWATDDIPDDLNSIRAAPGEFRAIMEMP\
                     SFYSHGLPRCSPYKKLTECQLKNPISGLLEYAQFASQTCEFNMIEQSGPPHEPRFKFQVV\
                     INGREFPPAEAGSKKVAKQDAAMKAMTILLEEAKAKDSGKSEESSHYSTEKESEKTAESQ\
                     TPTPSATSFFSGKSPVTTLLECMHKLGNSCEFRLLSKEGPAHEPKFQYCVAVGAQTFPSV\
                     SAPSKKVAKQMAAEEAMKALHGEATNSMASDNQPEGMISESLDNLESMMPNKVRKIGELV\
                     RYLNTNPVGGLLEYARSHGFAAEFKLVDQSGPPHEPKFVYQAKVGGRWFPAVCAHSKKQG\
                     KQEAADAALRVLIGENEKAERMGFTEVTPVTGASLRRTMLLLSRSPEAQPKTLPLTGSTF\
                     HDQIAMLSHRCFNTLTNSFQPSLLGRKILAAIIMKKDSEDMGVVVSLGTGNRCVKGDSLS\
                     LKGETVNDCHAEIISRRGFIRFLYSELMKYNSQTAKDSIFEPAKGGEKLQIKKTVSFHLY\
                     ISTAPCGDGALFDKSCSDRAMESTESRHYPVFENPKQGKLRTKVENGEGTIPVESSDIVP\
                     TWDGIRLGERLRTMSCSDKILRWNVLGLQGALLTHFLQPIYLKSVTLGYLFSQGHLTRAI\
                     CCRVTRDGSAFEDGLRHPFIVNHPKVGRVSIYDSKRQSGKTKETSVNWCLADGYDLEILD\
                     GTRGTVDGPRNELSRVSKKNIFLLFKKLCSFRYRRDLLRLSYGEAKKAARDYETAKNYFK\
                     KGLKDMGYGNWISKPQEEKNFYLCPV",
                ),
                (
                    "sp|Q7Z434|MAVS_HUMAN",
                    "MPFAEDKTYKYICRNFSNFCNVDVVEILPYLPCLTARDQDRLRATCTLSGNRDTLWHLFN\
                     TLQRRPGWVEYFIAALRGCELVDLADEVASVYQSYQPRTSDRPPDPLEPPSLPAERPGPP\
                     TPAAAHSIPYNSCREKEPSYPMPVQETQAPESPGENSEQALQTLSPRAIPRNPDGGPLES\
                     SSDLAALSPLTSSGHQEQDTELGSTHTAGATSSLTPSRGPVSPSVSFQPLARSTPRASRL\
                     PGPTGSVVSTGTSFSSSSPGLASAGAAEGKQGAESDQAEPIICSSGAEAPANSLPSKVPT\
                     TLMPVNTVALKVPANPASVSTVPSKLPTSSKPPGAVPSNALTNPAPSKLPINSTRAGMVP\
                     SKVPTSMVLTKVSASTVPTDGSSRNEETPAAPTPAGATGGSSAWLDSSSENRGLGSELSK\
                     PGVLASQVDSPFSGCFEDLAISASTSLGMGPCHGPEENEYKSEGTFGIHVAENPSIQLLE\
                     GNPGPPADPDGGPRPQADRKFQEREVPCHRPSPGALWLQVAVTGVLVVTLLVVLYRRRLH",
                ),
                (
                    "sp|Q8NB16|MLKL_HUMAN",
                    "MENLKHIITLGQVIHKRCEEMKYCKKQCRRLGHRVLGLIKPLEMLQDQGKRSVPSEKLTT\
                     AMNRFKAALEEANGEIEKFSNRSNICRFLTASQDKILFKDVNRKLSDVWKELSLLLQVEQ\
                     RMPVSPISQGASWAQEDQQDADEDRRAFQMLRRDNEKIEASLRRLEINMKEIKETLRQYL\
                     PPKCMQEIPQEQIKEIKKEQLSGSPWILLRENEVSTLYKGEYHRAPVAIKVFKKLQAGSI\
                     AIVRQTFNKEIKTMKKFESPNILRIFGICIDETVTPPQFSIVMEYCELGTLRELLDREKD\
                     LTLGKRMVLVLGAARGLYRLHHSEAPELHGKIRSSNFLVTQGYQVKLAGFELRKTQTSMS\
                     LGTTREKTDRVKSTAYLSPQELEDVFYQYDVKSEIYSFGIVLWEIATGDIPFQGCNSEKI\
                     RKLVAVKRQQEPLGEDCPSELREIIDECRAHDPSVRPSVDEILKKLSTFSK",
                ),
                (
                    "sp|Q96C10|DHX58_HUMAN",
                    "MELRSYQWEVIMPALEGKNIIIWLPTGAGKTRAAAYVAKRHLETVDGAKVVVLVNRVHLV\
                     TQHGEEFRRMLDGRWTVTTLSGDMGPRAGFGHLARCHDLLICTAELLQMALTSPEEEEHV\
                     ELTVFSLIVVDECHHTHKDTVYNVIMSQYLELKLQRAQPLPQVLGLTASPGTGGASKLDG\
                     AINHVLQLCANLDTWCIMSPQNCCPQLQEHSQQPCKQYNLCHRRSQDPFGDLLKKLMDQI\
                     HDHLEMPELSRKFGTQMYEQQVVKLSEAAALAGLQEQRVYALHLRRYNDALLIHDTVRAV\
                     DALAALQDFYHREHVTKTQILCAERRLLALFDDRKNELAHLATHGPENPKLEMLEKILQR\
                     QFSSSNSPRGIIFTRTRQSAHSLLLWLQQQQGLQTVDIRAQLLIGAGNSSQSTHMTQRDQ\
                     QEVIQKFQDGTLNLLVATSVAEEGLDIPHCNVVVRYGLLTNEISMVQARGRARADQSVYA\
                     FVATEGSRELKRELINEALETLMEQAVAAVQKMDQAEYQAKIRDLQQAALTKRAAQAAQR\
                     ENQRQQFPVEHVQLLCINCMVAVGHGSDLRKVEGTHHVNVNPNFSNYYNVSRDPVVINKV\
                     FKDWKPGGVISCRNCGEVWGLQMIYKSVKLPVLKVRSMLLETPQGRIQAKKWSRVPFSVP\
                     DFDFLQHCAENLSDLSLD",
                ),
                (
                    "sp|Q9BYX4|IFIH1_HUMAN",
                    "MSNGYSTDENFRYLISCFRARVKMYIQVEPVLDYLTFLPAEVKEQIQRTVATSGNMQAVE\
                     LLLSTLEKGVWHLGWTREFVEALRRTGSPLAARYMNPELTDLPSPSFENAHDEYLQLLNL\
                     LQPTLVDKLLVRDVLDKCMEEELLTIEDRNRIAAAENNGNESGVRELLKRIVQKENWFSA\
                     FLNVLRQTGNNELVQELTGSDCSESNAEIENLSQVDGPQVEEQLLSTTVQPNLEKEVWGM\
                     ENNSSESSFADSSVVSESDTSLAEGSVSCLDESLGHNSNMGSDSGTMGSDSDEENVAARA\
                     SPEPELQLRPYQMEVAQPALEGKNIIICLPTGSGKTRVAVYIAKDHLDKKKKASEPGKVI\
                     VLVNKVLLVEQLFRKEFQPFLKKWYRVIGLSGDTQLKISFPEVVKSCDIIISTAQILENS\
                     LLNLENGEDAGVQLSDFSLIIIDECHHTNKEAVYNNIMRHYLMQKLKNNRLKKENKPVIP\
                     LPQILGLTASPGVGGATKQAKAEEHILKLCANLDAFTIKTVKENLDQLKNQIQEPCKKFA\
                     IADATREDPFKEKLLEIMTRIQTYCQMSPMSDFGTQPYEQWAIQMEKKAAKEGNRKERVC\
                     AEHLRKYNEALQINDTIRMIDAYTHLETFYNEEKDKKFAVIEDDSDEGGDDEYCDGDEDE\
                     DDLKKPLKLDETDRFLMTLFFENNKMLKRLAENPEYENEKLTKLRNTIMEQYTRTEESAR\
                     GIIFTKTRQSAYALSQWITENEKFAEVGVKAHHLIGAGHSSEFKPMTQNEQKEVISKFRT\
                     GKINLLIATTVAEEGLDIKECNIVIRYGLVTNEIAMVQARGRARADESTYVLVAHSGSGV\
                     IEHETVNDFREKMMYKAIHCVQNMKPEEYAHKILELQMQSIMEKKMKTKRNIAKHYKNNP\
                     SLITFLCKNCSVLACSGEDIHVIEKMHHVNMTPEFKELYIVRENKALQKKCADYQINGEI\
                     ICKCGQAWGTMMVHKGLDLPCLKIRNFVVVFKNNSTKKQYKKWVELPITFPNLDYSECCL\
                     FSDED",
                ),
                (
                    "sp|Q9H171|ZBP1_HUMAN",
                    "MAQAPADPGREGHLEQRILQVLTEAGSPVKLAQLVKECQAPKRELNQVLYRMKKELKVSL\
                     TSPATWCLGGTDPEGEGPAELALSSPAERPQQHAATIPETPGPQFSQQREEDIYRFLKDN\
                     GPQRALVIAQALGMRTAKDVNRDLYRMKSRHLLDMDEQSKAWTIYRPEDSGRRAKSASII\
                     YQHNPINMICQNGPNSWISIANSEAIQIGHGNIITRQTVSREDGSAGPRHLPSMAPGDSS\
                     TWGTLVDPWGPQDIHMEQSILRRVQLGHSNEMRLHGVPSEGPAHIPPGSPPVSATAAGPE\
                     ASFEARIPSPGTHPEGEAAQRIHMKSCFLEDATIGNSNKMSISPGVAGPGGVAGSGEGEP\
                     GEDAGRRPADTQSRSHFPRDIGQPITPSHSKLTPKLETMTLGNRSHKAAEGSHYVDEASH\
                     EGSWWGGGI",
                ),
                (
                    "sp|Q9Y572|RIPK3_HUMAN",
                    "MSCVKLWPSGAPAPLVSIEELENQELVGKGGFGTVFRAQHRKWGYDVAVKIVNSKAISRE\
                     VKAMASLDNEFVLRLEGVIEKVNWDQDPKPALVTKFMENGSLSGLLQSQCPRPWPLLCRL\
                     LKEVVLGMFYLHDQNPVLLHRDLKPSNVLLDPELHVKLADFGLSTFQGGSQSGTGSGEPG\
                     GTLGYLAPELFVNVNRKASTASDVYSFGILMWAVLAGREVELPTEPSLVYEAVCNRQNRP\
                     SLAELPQAGPETPGLEGLKELMQLCWSSEPKDRPSFQECLPKTDEVFQMVENNMNAAVST\
                     VKDFLSQLRSSNRRFSIPESGQGGTEMDGFRRTIENQHSRNDVMVSEWLNKLNLEEPPSS\
                     VPKKCPSLTKRSRAQEEQVPQAWTAGTSSDSMAQPPQTPETSTFRNQMPSPTSTGTPSPG\
                     PRGNQGAERQGMNWSCRTPEPNPVTGRPLVNIYNCSGVQVGDNNYLTMQQTTALPTWGLA\
                     PSGKGRGLQHPPPVGSQEGPKDPEAWSRPQGWYNHSGK",
                ),
            ];

            let buffer = &mut Vec::new();
            for (id, seq) in sequences.iter() {
                buffer.clear();
                reader.fetch_full_seq(id, buffer)?;

                let fetched = String::from_utf8(buffer.clone())?;
                assert_eq!(fetched, *seq, "ID: {}", id);
            }
            let sequences: HashMap<_, _> = sequences.into_iter().collect();

            // Query individual intervals from the sequence
            for (seqid, intervals) in [
                (
                    "sp|Q7Z434|MAVS_HUMAN",
                    vec![
                        0..24,
                        0..100,
                        35..120,
                        59..60,
                        60..61,
                        61..62,
                        119..120,
                        120..121,
                        121..122,
                    ],
                ),
                (
                    "sp|Q9Y572|RIPK3_HUMAN",
                    vec![1..375, 60..180, 1..517, 180..518],
                ),
            ] {
                for interval in intervals {
                    buffer.clear();
                    reader.fetch(seqid, Interval::try_from(interval.clone())?, buffer)?;
                    let fetched = String::from_utf8(buffer.clone())?;

                    let expected =
                        &sequences[seqid][interval.start as usize..interval.end as usize];
                    assert_eq!(fetched, expected, "ID: {}, Interval: {:?}", seqid, interval);
                }
            }

            // Query invalid intervals
            for (seqid, intervals) in [
                ("sp|Q7Z434|MAVS_HUMAN", vec![0..1000, 1000..10000, 987..988]),
                ("sp|Q9Y572|RIPK3_HUMAN", vec![1..1000]),
            ] {
                for interval in intervals {
                    let interval = Interval::try_from(interval.clone())?;
                    let result = reader.fetch(seqid, interval, buffer);
                    assert!(result.is_err(), "ID: {}, Interval: {:?}", seqid, interval);
                }
            }

            Ok(())
        }

        for path in ["indexed.fa", "indexed.fa.bgz"] {
            let path = PathBuf::from(env!("BIOBIT_RESOURCES"))
                .join("fasta")
                .join(path);
            test(IndexedReader::from_path(
                &path,
                Decoder::from_path(&path, crate::fasta::EXTENSIONS).unwrap(),
            )?)?
        }

        Ok(())
    }

    #[test]
    fn test_multiple_indexed_fa() -> Result<()> {
        let mut indexed = Vec::new();
        for path in ["indexed.fa", "CHM13v2.M-21-22.fa.bgz"] {
            let path = PathBuf::from(env!("BIOBIT_RESOURCES"))
                .join("fasta")
                .join(path);
            indexed.push((
                path.clone(),
                Decoder::from_path(&path, crate::fasta::EXTENSIONS).unwrap(),
            ));
        }

        let mut reader = IndexedReader::from_paths(&indexed)?;
        for (seqid, interval, expected) in [
            ("sp|O95786|RIGI_HUMAN", 0..24, "MTTEQRRSLQAFQDYIRKTLDPTY"),
            ("sp|Q9Y572|RIPK3_HUMAN", 510..518, "GWYNHSGK"),
            ("chr21", 23997387..23997416, "tcctcctgctgctgctgcgcTACCTGGTG"),
            ("chrM", 2830..2849, "CAAAGGCCCCAACGTTGTA"),
        ] {
            let mut buffer = Vec::new();
            reader.fetch(seqid, Interval::try_from(interval.clone())?, &mut buffer)?;
            let fetched = String::from_utf8(buffer.clone())?;
            assert_eq!(fetched, expected, "ID: {}, Interval: {:?}", seqid, interval);
        }
        Ok(())
    }
}
