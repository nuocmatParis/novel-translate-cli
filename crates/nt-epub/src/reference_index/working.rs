use super::{ReferenceDiagnostic, ReferenceIndexError, ReferenceLocation, ReferenceTarget};

/// Space for owned variable-length data and the two vectors returned to the
/// caller. File decoding and scanner scratch have a separate allowance.
pub(super) struct GraphSpace {
    remaining: usize,
}

impl GraphSpace {
    pub(super) fn new(bytes: usize) -> Self {
        Self { remaining: bytes }
    }

    pub(super) fn take(&mut self, bytes: usize) -> Result<(), ReferenceIndexError> {
        self.remaining = self
            .remaining
            .checked_sub(bytes)
            .ok_or(ReferenceIndexError::WorkingMemoryLimit)?;
        Ok(())
    }

    pub(super) fn array<T>(&mut self, count: usize) -> Result<(), ReferenceIndexError> {
        self.take(
            count
                .checked_mul(size_of::<T>())
                .ok_or(ReferenceIndexError::WorkingMemoryLimit)?,
        )
    }

    pub(super) fn positions(
        &mut self,
        override_positions: Option<Vec<usize>>,
        source: &[usize],
    ) -> Result<Vec<usize>, ReferenceIndexError> {
        if let Some(positions) = override_positions {
            self.array::<usize>(positions.capacity())?;
            return Ok(positions);
        }
        self.array::<usize>(source.len())?;
        let mut positions = Vec::new();
        positions
            .try_reserve_exact(source.len())
            .map_err(|_| ReferenceIndexError::WorkingMemoryLimit)?;
        self.array::<usize>(positions.capacity() - source.len())?;
        positions.extend_from_slice(source);
        Ok(positions)
    }

    /// Account for old and new backing storage until the allocation succeeds.
    pub(super) fn slot<T>(&mut self, output: &mut Vec<T>) -> Result<(), ReferenceIndexError> {
        if output.len() < output.capacity() {
            return Ok(());
        }
        let old = output.capacity();
        let next = old
            .saturating_mul(2)
            .max(1)
            .min(self.remaining / size_of::<T>());
        if next <= old {
            return Err(ReferenceIndexError::WorkingMemoryLimit);
        }
        self.array::<T>(next)?;
        output
            .try_reserve_exact(next - old)
            .map_err(|_| ReferenceIndexError::WorkingMemoryLimit)?;
        self.array::<T>(output.capacity() - next)?;
        self.remaining += old * size_of::<T>();
        Ok(())
    }

    pub(super) fn location(
        &mut self,
        location: &ReferenceLocation,
    ) -> Result<(), ReferenceIndexError> {
        self.array::<u32>(location.element_ordinals.capacity())?;
        if let Some(value) = &location.attribute_namespace {
            self.take(value.capacity())?;
        }
        if let Some(value) = &location.attribute_local_name {
            self.take(value.capacity())?;
        }
        Ok(())
    }

    pub(super) fn target(&mut self, target: &ReferenceTarget) -> Result<(), ReferenceIndexError> {
        match target {
            ReferenceTarget::Internal { path, fragment }
            | ReferenceTarget::Missing { path, fragment } => {
                self.take(path.capacity())?;
                if let Some(fragment) = fragment {
                    self.take(fragment.capacity())?;
                }
            }
            ReferenceTarget::External { scheme } => self.take(scheme.capacity())?,
            ReferenceTarget::Invalid { .. } | ReferenceTarget::Ambiguous => (),
        }
        Ok(())
    }

    pub(super) fn diagnostic(
        &mut self,
        source: &str,
        location: Option<&ReferenceLocation>,
        code: ReferenceIndexError,
    ) -> Result<ReferenceDiagnostic, ReferenceIndexError> {
        self.take(source.len())?;
        if let Some(location) = location {
            self.location(location)?;
        }
        Ok(ReferenceDiagnostic {
            source_path: source.to_owned(),
            location: location.cloned(),
            code,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_vectors_spend_the_same_allowance_before_copying() {
        let mut space = GraphSpace::new(4 * size_of::<usize>());
        assert_eq!(space.positions(None, &[1, 4, 7]).unwrap(), [1, 4, 7]);
        assert_eq!(
            space.positions(None, &[3, 8]),
            Err(ReferenceIndexError::WorkingMemoryLimit)
        );
        assert_eq!(space.positions(Some(vec![9]), &[1, 4, 7]).unwrap(), [9]);
        assert_eq!(
            space.array::<usize>(1),
            Err(ReferenceIndexError::WorkingMemoryLimit)
        );
        assert_eq!(
            space.array::<usize>(usize::MAX),
            Err(ReferenceIndexError::WorkingMemoryLimit)
        );
    }
}
