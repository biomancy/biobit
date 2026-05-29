use std::cell::RefCell;
use std::collections::HashMap;
use std::io;

use crate::core::mismatches::{Batch, MismatchesVec};
use crate::core::runner::Runner;
use rayon::prelude::*;

use std::sync::Mutex;

use thread_local::ThreadLocal;

pub struct ThreadCache<Builder, Type>
where
    Builder: Fn() -> RefCell<Type>,
    Type: Send,
{
    mutex: Mutex<Builder>,
    thrlocal: ThreadLocal<RefCell<Type>>,
}

impl<Builder, Type> ThreadCache<Builder, Type>
where
    Builder: Fn() -> RefCell<Type>,
    Type: Send,
{
    pub fn new(builder: Builder) -> Self {
        Self {
            mutex: Mutex::new(builder),
            thrlocal: Default::default(),
        }
    }

    pub fn get(&self) -> &RefCell<Type> {
        self.thrlocal.get_or(|| self.mutex.lock().unwrap()())
    }

    pub fn dissolve(self) -> <ThreadLocal<RefCell<Type>> as IntoIterator>::IntoIter {
        self.thrlocal.into_iter()
    }
}

pub struct Args;
pub fn run(_args: Args) {
    // // Strander & Hooks don't require any further processing
    // let mut strander = args.stranding;
    // let hooks: HookEngine<SiteMismatchesVec> = HookEngine::new();
    //
    // // Mismatchs builder. Always with prefilter since there are no site-level stats right now
    // let builder = SiteMismatchesBuilder::new(args.maxwsize, args.refnucpred, args.retain, Some(args.prefilter));
    //
    // // Initialize basic counter
    // let counter = BaseNucCounter::new(args.maxwsize, args.readfilter);
    // let counter = IntervalNucCounter::new(counter);
    //
    // // Remove all stranding algorithm -> they are not required
    // strander.clear();
    // // Compose strander + pileuper
    // let deductor = crate::core::stranding::deduce::DeduceStrandByDesign::new(todo!());
    // let pileuper = HTSPileupEngine::new(args.bamfiles, StrandedNucCounter::new(counter, deductor));
    //
    // // Launch the processing
    // let runner = REATRunner::new(builder, strander, pileuper, hooks);
    // _run(args.workload, runner, factory(), &mut args.saveto).unwrap();
}

pub fn _run<RunnerT, Mismatches, Workload, W: io::Write>(
    workload: Vec<Workload>,
    runner: RunnerT,
) -> eyre::Result<HashMap<String, Vec<Mismatches>>>
where
    Mismatches: Send + MismatchesVec,
    Workload: Sized + Send,
    RunnerT: for<'runner> Runner<'runner, Mismatches, Workload = Workload> + Clone + Send,
{
    let ctxstore = ThreadCache::new(move || RefCell::new(runner.clone()));
    let edits: Vec<Batch<Mismatches>> = workload
        .into_par_iter()
        // .into_iter()
        .filter_map(|w| {
            let result = ctxstore.get().borrow_mut().run(w);
            result
        })
        .collect();

    // Group by contigs
    let mut percontig = HashMap::with_capacity(120);
    for batch in edits {
        for item in [batch.items, batch.retained] {
            for mm in [item.forward, item.dual, item.reverse] {
                if mm.is_empty() {
                    continue;
                }
                if !percontig.contains_key(mm.contig()) {
                    percontig.insert(mm.contig().to_owned(), vec![]);
                }
                percontig.get_mut(mm.contig()).unwrap().push(mm);
            }
        }
    }
    Ok(percontig)
}
