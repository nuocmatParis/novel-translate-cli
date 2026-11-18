//! Count live owned buffers without serializing or copying the snapshot.
use super::{CodecError, records::Budget};

pub(crate) trait Heap {
    fn heap(&self, budget: &mut Budget) -> Result<(), CodecError>;
}

impl Heap for String {
    fn heap(&self, budget: &mut Budget) -> Result<(), CodecError> {
        budget.charge(self.capacity(), 0)
    }
}
impl<T: Heap> Heap for Vec<T> {
    fn heap(&self, budget: &mut Budget) -> Result<(), CodecError> {
        budget.charge(
            self.capacity()
                .checked_mul(std::mem::size_of::<T>())
                .ok_or(CodecError::ResourceLimit)?,
            self.len(),
        )?;
        for value in self {
            value.heap(budget)?;
        }
        Ok(())
    }
}
impl<T: Heap> Heap for Option<T> {
    fn heap(&self, budget: &mut Budget) -> Result<(), CodecError> {
        if let Some(value) = self {
            value.heap(budget)?;
        }
        Ok(())
    }
}
impl<T: Heap> Heap for Box<T> {
    fn heap(&self, budget: &mut Budget) -> Result<(), CodecError> {
        budget.charge(std::mem::size_of::<T>(), 1)?;
        (**self).heap(budget)
    }
}
impl<A: Heap, B: Heap> Heap for (A, B) {
    fn heap(&self, budget: &mut Budget) -> Result<(), CodecError> {
        self.0.heap(budget)?;
        self.1.heap(budget)
    }
}
impl<T: Heap, const N: usize> Heap for [T; N] {
    fn heap(&self, budget: &mut Budget) -> Result<(), CodecError> {
        for value in self {
            value.heap(budget)?;
        }
        Ok(())
    }
}
macro_rules! scalar {
    ($($value:ty),* $(,)?) => { $(impl Heap for $value {
        fn heap(&self, _: &mut Budget) -> Result<(), CodecError> { Ok(()) }
    })* };
}
scalar!(u8, u16, u32, u64, usize, i64, bool, super::Digest);

// Exhaustive destructuring makes a new field or enum variant a compiler error
// until its retained allocation has been considered here.
macro_rules! heap_struct {
    ($name:ident {$($field:ident),* $(,)?}) => {
        impl $crate::import_codec::memory::Heap for $name {
            fn heap(&self, budget: &mut $crate::import_codec::records::Budget)
                -> Result<(), $crate::import_codec::CodecError> {
                let Self { $($field),* } = self;
                $($crate::import_codec::memory::Heap::heap($field, budget)?;)*
                Ok(())
            }
        }
    };
}
pub(super) use heap_struct;

macro_rules! heap_enum {
    ($name:ident {$($variant:ident $({$($field:ident),* $(,)?})? $(($inner:ident))?),* $(,)?}) => {
        impl $crate::import_codec::memory::Heap for $name {
            fn heap(&self, budget: &mut $crate::import_codec::records::Budget)
                -> Result<(), $crate::import_codec::CodecError> {
                let _ = budget;
                match self { $(Self::$variant $({$($field),*})? $(($inner))? => {
                    $($($crate::import_codec::memory::Heap::heap($field, budget)?;)*)?
                    $($crate::import_codec::memory::Heap::heap($inner, budget)?;)?
                    Ok(())
                }),* }
            }
        }
    };
}
pub(super) use heap_enum;

mod epub;

pub(crate) fn working<T: Heap>(value: &T, reserved: usize) -> Result<usize, CodecError> {
    let mut budget = Budget::new(reserved)?;
    budget.charge(std::mem::size_of::<T>(), 1)?;
    value.heap(&mut budget)?;
    Ok(budget.used_bytes())
}

impl super::VolumeRecord {
    /// Space occupied by this complete snapshot, including spare capacities.
    pub fn retained_working_bytes(&self) -> Result<usize, CodecError> {
        working(self, 0)
    }

    pub(crate) fn reimport_history_bytes(&self) -> Result<usize, CodecError> {
        use super::ScopeRecord;
        let mut budget = Budget::new(working(&self.mapping.confirmed_boundaries, 0)?)?;
        let content = self
            .scope
            .records
            .iter()
            .filter_map(|record| match record {
                ScopeRecord::Content(content) => Some(&content.input),
                _ => None,
            })
            .chain(self.scope.notes.iter().map(|note| &note.content.input));
        for item in content.filter(|item| !item.history.is_empty()) {
            // The saved decisions move into the history map. Their keys and
            // reimported classification inputs can coexist until scope staging.
            let entry = working(&item.history, working(&item.observation.location, 512)?)?;
            budget.charge(entry.checked_mul(3).ok_or(CodecError::ResourceLimit)?, 0)?;
        }
        Ok(budget.used_bytes())
    }

    pub(crate) fn scope_working_bytes(&self, reserved: usize) -> Result<usize, CodecError> {
        let live = working(self, reserved)?;
        let restored = working(&self.scope, 0)?
            .checked_mul(8)
            .ok_or(CodecError::ResourceLimit)?;
        let mut budget = Budget::new(live)?;
        budget.charge(restored, 0)?;
        self.reserve_scope_lookup(&mut budget)?;
        Ok(budget.used_bytes())
    }

    /// Rebuilding observations holds compact source views and restored
    /// mapping. The original saved scope is still encoded during this pass.
    pub(crate) fn scope_rebuild_bytes(&self, reserved: usize) -> Result<usize, CodecError> {
        let mut budget = Budget::new(working(self, reserved)?)?;
        for bytes in [working(&self.mapping, 0)?, working(&self.source_index, 0)?] {
            budget.charge(bytes.checked_mul(2).ok_or(CodecError::ResourceLimit)?, 0)?;
        }
        self.reserve_scope_lookup(&mut budget)?;
        Ok(budget.used_bytes())
    }

    fn reserve_scope_lookup(&self, budget: &mut Budget) -> Result<(), CodecError> {
        let blocks = self
            .source_index
            .documents
            .iter()
            .try_fold(0usize, |sum, doc| {
                sum.checked_add(doc.blocks.len())
                    .ok_or(CodecError::ResourceLimit)
            })?;
        budget.charge(
            blocks.checked_mul(2048).ok_or(CodecError::ResourceLimit)?,
            0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_count_includes_unused_capacity_and_accepts_only_fitting_reservations() {
        let mut text = String::with_capacity(8192);
        text.push('x');
        let mut values = Vec::with_capacity(16);
        values.push(text);
        let expected = std::mem::size_of_val(&values)
            + values.capacity() * std::mem::size_of::<String>()
            + values[0].capacity();
        assert_eq!(working(&values, 0).unwrap(), expected);
        let limit = 256 * 1024 * 1024;
        assert_eq!(working(&values, limit - expected).unwrap(), limit);
        assert_eq!(
            working(&values, limit - expected + 1),
            Err(CodecError::ResourceLimit)
        );
    }
}
