use super::acquisition::illumina;
use super::library::p5p7;
use super::{
    Acquisition, AcquisitionId, AcquisitionIdRef, Library, LibraryId, LibraryIdRef, Sample,
    SampleId, Source, SourceId,
};
use crate::lookup;
use crate::primitives::{OrderedMap, impl_checked_lookup, impl_direct_lookup, impl_variant_lookup};
use crate::{Lookup, Result, UntypedId};

/// The resolved, immutable provenance graph for a released description.
///
/// This initial graph models biological material and its immediate acquisition
/// contract: [`Source`] <- [`Sample`] <- [`Library`] <- [`Acquisition`].
/// Each collection owns its records, and each child owns the concrete parent ID
/// types it can consume.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Provenance {
    sources: OrderedMap<SourceId, Source>,
    samples: OrderedMap<SampleId, Sample>,
    libraries: OrderedMap<UntypedId, Library>,
    acquisitions: OrderedMap<UntypedId, Acquisition>,
}

impl Provenance {
    /// Constructs and validates a released provenance graph.
    pub fn new(
        sources: impl IntoIterator<Item = Source>,
        samples: impl IntoIterator<Item = Sample>,
        libraries: impl IntoIterator<Item = Library>,
        acquisitions: impl IntoIterator<Item = Acquisition>,
    ) -> Result<Self> {
        let provenance = Self {
            sources: OrderedMap::try_from_iter(
                "provenance.sources",
                sources
                    .into_iter()
                    .map(|source| (source.id().clone(), source)),
            )?,
            samples: OrderedMap::try_from_iter(
                "provenance.samples",
                samples
                    .into_iter()
                    .map(|sample| (sample.id().clone(), sample)),
            )?,
            libraries: OrderedMap::try_from_iter(
                "provenance.libraries",
                libraries
                    .into_iter()
                    .map(|library| (library.id().as_untyped().clone(), library)),
            )?,
            acquisitions: OrderedMap::try_from_iter(
                "provenance.acquisitions",
                acquisitions
                    .into_iter()
                    .map(|acquisition| (acquisition.id().as_untyped().clone(), acquisition)),
            )?,
        };
        provenance.validate()?;
        Ok(provenance)
    }

    /// Iterates over sources in ID order.
    pub fn sources(&self) -> impl ExactSizeIterator<Item = &Source> + '_ {
        self.sources.values()
    }

    /// Iterates over samples in ID order.
    pub fn samples(&self) -> impl ExactSizeIterator<Item = &Sample> + '_ {
        self.samples.values()
    }

    /// Iterates over libraries in ID order.
    pub fn libraries(&self) -> impl ExactSizeIterator<Item = &Library> + '_ {
        self.libraries.values()
    }

    /// Iterates over acquisitions in ID order.
    pub fn acquisitions(&self) -> impl ExactSizeIterator<Item = &Acquisition> + '_ {
        self.acquisitions.values()
    }

    /// Finds a source by local ID.
    pub fn source(&self, id: &UntypedId) -> Option<&Source> {
        self.sources.get(id)
    }

    /// Finds a sample by local ID.
    pub fn sample(&self, id: &UntypedId) -> Option<&Sample> {
        self.samples.get(id)
    }

    /// Finds a library by local ID.
    pub fn library(&self, id: &UntypedId) -> Option<&Library> {
        self.libraries.get(id)
    }

    /// Finds an acquisition by local ID.
    pub fn acquisition(&self, id: &UntypedId) -> Option<&Acquisition> {
        self.acquisitions.get(id)
    }

    /// Performs a lookup whose result is determined by its typed ID.
    pub fn get<'a, K: ?Sized>(
        &'a self,
        key: &K,
    ) -> lookup::OptResult<&'a <Self as Lookup<K>>::Target, <Self as Lookup<K>>::Err>
    where
        Self: Lookup<K>,
    {
        <Self as Lookup<K>>::lookup(self, key)
    }

    fn validate(&self) -> Result<()> {
        Sample::validate_against(self)?;
        Library::validate_against(self)?;
        Acquisition::validate_against(self)
    }
}

impl_direct_lookup!(Provenance, SourceId, Source, sources);
impl_direct_lookup!(Provenance, SampleId, Sample, samples);
impl_checked_lookup!(Provenance, LibraryId, Library, libraries);
impl_checked_lookup!(Provenance, LibraryIdRef<'_>, Library, libraries);
impl_checked_lookup!(Provenance, AcquisitionId, Acquisition, acquisitions);
impl_checked_lookup!(Provenance, AcquisitionIdRef<'_>, Acquisition, acquisitions);

impl_variant_lookup!(
    Provenance,
    p5p7::LibraryId,
    p5p7::Library,
    libraries,
    Library::P5P7
);
impl_variant_lookup!(
    Provenance,
    illumina::SingleEndSequencingId,
    illumina::SingleEndSequencing,
    acquisitions,
    Acquisition::IlluminaSingleEndSequencing
);
impl_variant_lookup!(
    Provenance,
    illumina::PairedEndSequencingId,
    illumina::PairedEndSequencing,
    acquisitions,
    Acquisition::IlluminaPairedEndSequencing
);
