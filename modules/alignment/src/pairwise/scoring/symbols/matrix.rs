// pub struct Matrix<T> {
//     alphsize: usize,
//     scores: Vec<T>,
//     subtypes: Vec<EquivType>,
// }
//
// impl<T: Into<Score> + Copy> Matrix<T> {
//     pub fn new(alphsize: usize, matched: T, mismatched: T) -> Self {
//         let mut scores = Vec::new();
//         scores.resize(alphsize * alphsize, mismatched);
//         let mut subtypes = Vec::new();
//         subtypes.resize(alphsize * alphsize, EquivType::Mismatch);
//
//         for i in 0..alphsize {
//             scores[i * alphsize + i] = matched;
//             subtypes[i * alphsize + i] = EquivType::Match;
//         }
//
//         Self {
//             alphsize,
//             scores,
//             subtypes,
//         }
//     }
//
//     pub fn set(&mut self, a: Symbol, b: Symbol, weight: T, subtype: EquivType) -> &mut Self {
//         let ind = (a as usize) * self.alphsize + (b as usize);
//         self.scores[ind] = weight;
//         self.subtypes[ind] = subtype;
//         self
//     }
// }
//
// impl<T: Into<Score> + Copy> EquivClassifier for Matrix<T> {
//     fn classify(&self, a: Symbol, b: Symbol) -> EquivType {
//         let ind = (a as usize) * self.alphsize + (b as usize);
//         self.subtypes[ind]
//     }
// }
//
// impl<T: Into<Score> + Copy> PosInvariantScorer for Matrix<T> {
//     #[inline(always)]
//     fn score(&self, a: Symbol, b: Symbol) -> Score {
//         let ind = (a as usize) * self.alphsize + (b as usize);
//         self.scores[ind].into()
//     }
// }