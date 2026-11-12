use nt_domain::workspace::{GenerationCommitError, GenerationCommitResult, GenerationWriteRequest};

/// Commit boundary for one Series generation. Implementations own storage and
/// worker lifecycle; callers provide the complete immutable write set.
pub trait GenerationRepository: Send + Sync {
    fn commit_generation(
        &self,
        request: GenerationWriteRequest,
    ) -> Result<GenerationCommitResult, GenerationCommitError>;
}

pub fn commit_generation(
    repository: &impl GenerationRepository,
    request: GenerationWriteRequest,
) -> Result<GenerationCommitResult, GenerationCommitError> {
    repository.commit_generation(request)
}
