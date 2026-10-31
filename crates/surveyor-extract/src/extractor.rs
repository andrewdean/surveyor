//! The extractor trait and the registry that runs a set of extractors.

use itertools::{Either, Itertools};
use surveyor_model::{ExtractorId, FactSet, SourceUnit};

use crate::ExtractError;

/// Produces facts from a source unit.
///
/// Implementations are stateless or internally synchronized so one instance can run
/// over many files in parallel.
pub trait Extractor: Send + Sync {
    /// Stable identifier recorded on every fact set this extractor produces.
    fn id(&self) -> ExtractorId;

    /// True when this extractor knows how to read `unit`.
    fn accepts(&self, unit: &SourceUnit) -> bool;

    /// Extracts facts from `unit`.
    ///
    /// # Errors
    /// Returns an error when the unit cannot be parsed at all; partial parses succeed
    /// with the facts that could be recovered.
    fn extract(&self, unit: &SourceUnit) -> Result<FactSet, ExtractError>;
}

/// An ordered set of extractors run over each unit.
#[derive(Default)]
pub struct Extractors {
    items: Vec<Box<dyn Extractor>>,
}

impl std::fmt::Debug for Extractors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Extractors")
            .field("items", &self.ids().collect::<Vec<_>>())
            .finish()
    }
}

/// What one unit produced: the fact sets that succeeded and the failures, side by side.
#[derive(Debug, Default)]
pub struct Extracted {
    /// Fact sets from the extractors that succeeded.
    pub facts: Vec<FactSet>,
    /// Failures from the extractors that did not; never hides the successes.
    pub errors: Vec<ExtractError>,
}

impl Extractors {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an extractor and returns the registry.
    #[must_use]
    pub fn with(mut self, extractor: impl Extractor + 'static) -> Self {
        self.items.push(Box::new(extractor));
        self
    }

    /// Adds a boxed extractor, for adapters that build theirs dynamically.
    pub fn push(&mut self, extractor: Box<dyn Extractor>) {
        self.items.push(extractor);
    }

    /// Number of registered extractors.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// True when no extractor is registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Identifiers of the registered extractors, in order.
    pub fn ids(&self) -> impl Iterator<Item = ExtractorId> + '_ {
        self.items.iter().map(|e| e.id())
    }

    /// Runs every accepting extractor over `unit`.
    #[must_use]
    pub fn run(&self, unit: &SourceUnit) -> Extracted {
        let (facts, errors) = self
            .items
            .iter()
            .filter(|e| e.accepts(unit))
            .map(|e| e.extract(unit))
            .partition_map(|r| match r {
                Ok(set) => Either::Left(set),
                Err(err) => Either::Right(err),
            });
        Extracted { facts, errors }
    }
}
