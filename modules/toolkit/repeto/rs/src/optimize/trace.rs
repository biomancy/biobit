use std::collections::BTreeMap;

#[derive(Clone)]
pub struct TraceCell {
    pub rnaid: Option<usize>,
    pub included: Vec<(usize, usize)>,
}

pub struct Tracer {
    mat: Vec<BTreeMap<usize, TraceCell>>,
}

impl Tracer {
    pub fn new(starts: usize, _ends: usize) -> Self {
        Self {
            mat: vec![BTreeMap::new(); starts],
        }
    }

    pub fn reset(&mut self, starts: usize, _ends: usize) {
        self.mat.resize(starts, Default::default());
        for x in &mut self.mat {
            x.clear();
        }
    }

    pub fn update(&mut self, start: usize, end: usize, trace: TraceCell) {
        self.mat[start].insert(end, trace);
    }

    pub fn trace(&self, start: usize, end: usize) -> Vec<usize> {
        if !self.mat[start].contains_key(&end) {
            return vec![];
        }

        let mut rnaids = Vec::new();

        let mut queue = vec![(start, end)];
        while let Some((sind, eind)) = queue.pop() {
            let trace = &self.mat[sind][&eind];

            if trace.rnaid.is_some() {
                rnaids.push(trace.rnaid.unwrap());
            }
            queue.extend(trace.included.iter());
        }

        rnaids
    }
}
