//! How an edge was derived, how sure Surveyor is, and what text supports it.
//!
//! Three rules hold everywhere: an [`Derivation::Inferred`] edge never carries
//! [`Confidence::CERTAIN`]; every non-inferred edge carries at least one
//! [`Evidence`]; and every query can filter on [`DerivationKind`], so an agent can ask
//! for deterministic edges only.

use std::fmt::{self, Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::{
    CommitId, ContentHash, GrammarId, ModelError, ModelId, ProbeId, RepoId, ResolverId, RuleId,
    SourceLocation,
};

/// Milliseconds since the Unix epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(i64);

impl Timestamp {
    /// The current wall-clock time.
    #[must_use]
    pub fn now() -> Self {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX));
        Self(millis)
    }

    /// Wraps a millisecond count.
    #[must_use]
    pub const fn from_unix_millis(millis: i64) -> Self {
        Self(millis)
    }

    /// Milliseconds since the Unix epoch.
    #[must_use]
    pub const fn as_unix_millis(self) -> i64 {
        self.0
    }
}

/// A closed interval of time during which a runtime observation was made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start of the window.
    pub start: Timestamp,
    /// End of the window.
    pub end: Timestamp,
}

/// Where a declared relationship was declared.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum DeclaredSource {
    /// A package manifest, such as `Cargo.toml` or `package.json`.
    Manifest,
    /// A deployment or runtime configuration file.
    Config,
    /// A commit trailer, such as `Design: RFD-abc`.
    Trailer,
    /// An in-code citation marker.
    Citation,
    /// A documentation front matter block.
    FrontMatter,
    /// A repository profile (`surveyor.toml`).
    Profile,
    /// Another declared source, named.
    Other(String),
}

/// The method that produced an edge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Derivation {
    /// A Tree-sitter query matched; names were matched without semantic resolution.
    Syntactic { grammar: GrammarId, rule: RuleId },
    /// A precise resolver (SCIP, sqlparser, a semantic enricher) resolved the target.
    Resolved { resolver: ResolverId },
    /// A manifest, configuration, trailer or citation stated the relationship.
    Declared { source: DeclaredSource },
    /// Runtime telemetry observed the relationship inside a time window.
    Observed { probe: ProbeId, window: TimeRange },
    /// A model proposed the relationship; never certain.
    Inferred {
        model: ModelId,
        prompt_hash: ContentHash,
        rationale: String,
    },
}

impl Derivation {
    /// The derivation's category, for filtering.
    #[must_use]
    pub fn kind(&self) -> DerivationKind {
        match self {
            Self::Syntactic { .. } => DerivationKind::Syntactic,
            Self::Resolved { .. } => DerivationKind::Resolved,
            Self::Declared { .. } => DerivationKind::Declared,
            Self::Observed { .. } => DerivationKind::Observed,
            Self::Inferred { .. } => DerivationKind::Inferred,
        }
    }

    /// True for derivations that reproduce exactly from the same input.
    #[must_use]
    pub fn is_deterministic(&self) -> bool {
        self.kind().is_deterministic()
    }
}

/// The category of a [`Derivation`], without its parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DerivationKind {
    Syntactic,
    Resolved,
    Declared,
    Observed,
    Inferred,
}

impl DerivationKind {
    /// The lowercase name used in filters and storage.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Syntactic => "syntactic",
            Self::Resolved => "resolved",
            Self::Declared => "declared",
            Self::Observed => "observed",
            Self::Inferred => "inferred",
        }
    }

    /// True for syntactic, resolved and declared derivations.
    #[must_use]
    pub fn is_deterministic(self) -> bool {
        matches!(self, Self::Syntactic | Self::Resolved | Self::Declared)
    }
}

impl Display for DerivationKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for DerivationKind {
    type Err = ModelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "syntactic" => Ok(Self::Syntactic),
            "resolved" => Ok(Self::Resolved),
            "declared" => Ok(Self::Declared),
            "observed" => Ok(Self::Observed),
            "inferred" => Ok(Self::Inferred),
            other => Err(ModelError::invalid("DerivationKind", other)),
        }
    }
}

/// A probability-like score in `[0, 1]`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Confidence(f32);

impl Confidence {
    /// Full confidence; reserved for deterministic derivations.
    pub const CERTAIN: Self = Self(1.0);

    /// Creates a confidence, clamping into `[0, 1]` and mapping NaN to zero.
    #[must_use]
    pub fn new(value: f32) -> Self {
        if value.is_nan() {
            Self(0.0)
        } else {
            Self(value.clamp(0.0, 1.0))
        }
    }

    /// The underlying score.
    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }
}

impl Display for Confidence {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

/// A source span that supports an edge.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Evidence {
    /// Repository the span lives in.
    pub repo: RepoId,
    /// File and span of the supporting text.
    pub location: SourceLocation,
    /// Commit the span was read at.
    pub commit: CommitId,
    /// Hash of the span's text, so a moved span is still recognized.
    pub content_hash: ContentHash,
}

impl Evidence {
    /// A `file:lines` citation for prompts and logs.
    #[must_use]
    pub fn citation(&self) -> String {
        self.location.citation()
    }
}

/// Derivation, confidence and evidence for one edge.
///
/// Construct through [`Provenance::syntactic`], [`Provenance::resolved`],
/// [`Provenance::declared`], [`Provenance::observed`] or [`Provenance::inferred`]: the
/// first four take their first piece of evidence as an argument, so a deterministic
/// edge without evidence cannot be built, and the last caps confidence below certainty.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    /// How the edge was derived.
    pub derivation: Derivation,
    /// How sure Surveyor is; deterministic derivations are usually certain.
    pub confidence: Confidence,
    /// Source spans supporting the edge; at least one unless inferred.
    pub evidence: Vec<Evidence>,
    /// When the edge was produced.
    pub observed_at: Timestamp,
}

impl Provenance {
    fn with_evidence(derivation: Derivation, evidence: Evidence) -> Self {
        Self {
            derivation,
            confidence: Confidence::CERTAIN,
            evidence: vec![evidence],
            observed_at: Timestamp::now(),
        }
    }

    /// Certain provenance from a Tree-sitter rule match, supported by `evidence`.
    #[must_use]
    pub fn syntactic(grammar: GrammarId, rule: RuleId, evidence: Evidence) -> Self {
        Self::with_evidence(Derivation::Syntactic { grammar, rule }, evidence)
    }

    /// Certain provenance from a precise resolver, supported by `evidence`.
    #[must_use]
    pub fn resolved(resolver: ResolverId, evidence: Evidence) -> Self {
        Self::with_evidence(Derivation::Resolved { resolver }, evidence)
    }

    /// Certain provenance from a declaration in a manifest or config.
    #[must_use]
    pub fn declared(source: DeclaredSource, evidence: Evidence) -> Self {
        Self::with_evidence(Derivation::Declared { source }, evidence)
    }

    /// Provenance from a runtime probe, supported by `evidence`.
    #[must_use]
    pub fn observed(probe: ProbeId, window: TimeRange, evidence: Evidence) -> Self {
        Self::with_evidence(Derivation::Observed { probe, window }, evidence)
    }

    /// Provenance from a model's proposal; the confidence is capped below certainty.
    #[must_use]
    pub fn inferred(
        model: ModelId,
        prompt_hash: ContentHash,
        rationale: String,
        confidence: Confidence,
    ) -> Self {
        Self {
            derivation: Derivation::Inferred {
                model,
                prompt_hash,
                rationale,
            },
            confidence: cap_inferred(confidence),
            evidence: Vec::new(),
            observed_at: Timestamp::now(),
        }
    }

    /// Sets the confidence and returns the provenance; inferred edges stay below certainty.
    #[must_use]
    pub fn with_confidence(self, confidence: Confidence) -> Self {
        let confidence = match self.derivation {
            Derivation::Inferred { .. } => cap_inferred(confidence),
            _ => confidence,
        };
        Self { confidence, ..self }
    }

    /// Adds supporting evidence and returns the provenance.
    #[must_use]
    pub fn with_more_evidence(mut self, evidence: impl IntoIterator<Item = Evidence>) -> Self {
        self.evidence.extend(evidence);
        self
    }

    /// True when the derivation reproduces exactly from the same input.
    #[must_use]
    pub fn is_deterministic(&self) -> bool {
        self.derivation.is_deterministic()
    }
}

/// The highest confidence an inferred edge may carry.
///
/// Chosen so that a filter on `confidence >= 1.0` returns deterministic edges only,
/// while an inferred edge the model is sure about still ranks above weaker ones.
const INFERRED_CEILING: f32 = 0.99;

fn cap_inferred(confidence: Confidence) -> Confidence {
    if confidence >= Confidence::CERTAIN {
        Confidence::new(INFERRED_CEILING)
    } else {
        confidence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inferred_is_never_certain() {
        let p = Provenance::inferred(
            ModelId::new("m"),
            ContentHash::of(b"p"),
            "looks similar".into(),
            Confidence::CERTAIN,
        );
        assert!(p.confidence < Confidence::CERTAIN);
        assert!(!p.is_deterministic());
        assert_eq!(p.derivation.kind(), DerivationKind::Inferred);
    }

    #[test]
    fn confidence_clamps() {
        assert!((Confidence::new(2.0).get() - 1.0).abs() < f32::EPSILON);
        assert!(Confidence::new(-1.0).get().abs() < f32::EPSILON);
        assert!(Confidence::new(f32::NAN).get().abs() < f32::EPSILON);
        assert_eq!(
            "resolved".parse::<DerivationKind>().ok(),
            Some(DerivationKind::Resolved)
        );
    }

    #[test]
    fn derivation_serializes_with_tag() {
        let d = Derivation::Resolved {
            resolver: ResolverId::new("sqlparser"),
        };
        let json = serde_json::to_string(&d).expect("serializes");
        assert!(json.contains("\"kind\":\"resolved\""));
    }
}
