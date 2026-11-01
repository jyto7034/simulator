use crate::authoring::control::{self, Resource};
use serde::Serialize;
use std::io::{self, Write};

/// Admit each encoded chunk before copying a typed payload. A single counter
/// spans substitutions and source-map entries; no encoded document is retained.
pub(super) fn payload(
    value: &impl Serialize,
    total: &mut usize,
    limit: usize,
) -> Result<(), String> {
    struct Counter<'a> {
        total: &'a mut usize,
        limit: usize,
    }
    impl Write for Counter<'_> {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let next = self
                .total
                .checked_add(bytes.len())
                .filter(|n| *n <= self.limit)
                .ok_or_else(|| io::Error::other("expanded payload byte limit exceeded"))?;
            control::charge(bytes.len() as u64, "composition_payload_copy")
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
            control::admit(
                Resource::MetadataBytes,
                *self.total,
                bytes.len(),
                "composition_payload_storage",
            )
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
            *self.total = next;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Counter { total, limit }, value).map_err(|e| e.to_string())
}
