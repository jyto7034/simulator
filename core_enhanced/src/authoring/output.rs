//! Mandatory output encoding. Optional trace must never use this writer.

use super::{control, AuthoringError, ExecutionPhase, RequestFailure};
use serde::Serialize;
use std::io::Write;

struct Writer {
    bytes: Vec<u8>,
    receipt_remaining: Option<usize>,
    receipt_limit_failed: bool,
}
impl Writer {
    fn new(receipt_remaining: Option<usize>) -> Self {
        Self {
            bytes: Vec::new(),
            receipt_remaining,
            receipt_limit_failed: false,
        }
    }
    fn error(&self, error: impl std::fmt::Display) -> AuthoringError {
        if self.receipt_limit_failed {
            receipt_error(
                "authoring_receipt_limit",
                "Mandatory receipt bytes exceed the session ceiling.",
            )
        } else {
            encoding_error(error.to_string())
        }
    }
}
impl std::io::Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self
            .receipt_remaining
            .is_some_and(|remaining| bytes.len() > remaining - self.bytes.len())
        {
            self.receipt_limit_failed = true;
            return Err(std::io::Error::other("mandatory receipt byte ceiling"));
        }
        control::admit(
            control::Resource::ResultBytes,
            self.bytes.len(),
            bytes.len(),
            "mandatory_result",
        )
        .and_then(|()| control::charge(bytes.len() as u64, "mandatory_result_encode"))
        .map_err(|error| std::io::Error::other(format!("{error:?}")))?;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(super) fn encode_result(value: &impl Serialize) -> Result<String, AuthoringError> {
    let mut writer = Writer::new(None);
    serde_json::to_writer(&mut writer, value).map_err(|error| encoding_error(error.to_string()))?;
    String::from_utf8(writer.bytes).map_err(|error| encoding_error(error.to_string()))
}

#[derive(Debug)]
pub(super) struct PreparedReceipt {
    pub document: String,
    pub total_bytes: usize,
}

/// Captures cumulative scope usage through this encoding. Intermediate receipts
/// may precede more work in the same preparation scope. Its final receipt must
/// follow all charged preparation; only moves/final poll follow that receipt,
/// whose header then equals the scope's returned usage.
pub(super) fn encode_receipt(
    payload: &impl Serialize,
    committed_bytes: usize,
    max_total_bytes: usize,
) -> Result<PreparedReceipt, AuthoringError> {
    if !control::is_active() {
        return Err(receipt_error(
            "authoring_scope_required",
            "Receipt preparation requires an active authoring attempt.",
        ));
    }
    let _phase = control::phase(ExecutionPhase::Receipt);
    if max_total_bytes == 0
        || max_total_bytes > 64 * 1024 * 1024
        || committed_bytes > max_total_bytes
    {
        return Err(receipt_error(
            "authoring_receipt_limits_invalid",
            "Trusted receipt ceiling or committed byte count is invalid.",
        ));
    }
    let prefix = usage_prefix(&super::ExecutionUsage::default())?;
    let prefix_len = prefix.len();
    let mut writer = Writer::new(Some(max_total_bytes - committed_bytes));
    writer
        .write_all(prefix.as_bytes())
        .map_err(|error| writer.error(error))?;
    serde_json::to_writer(&mut writer, payload).map_err(|error| writer.error(error))?;
    writer
        .write_all(b"}")
        .map_err(|error| writer.error(error))?;
    control::charge((prefix_len as u64).saturating_mul(2), "receipt_usage_fill")
        .map_err(|error| encoding_error(format!("{error:?}")))?;
    let usage = control::usage_snapshot().ok_or_else(|| {
        receipt_error(
            "authoring_scope_required",
            "Receipt preparation lost its authoring scope.",
        )
    })?;
    let prefix = usage_prefix(&usage)?;
    if prefix.len() != prefix_len {
        return Err(receipt_error(
            "authoring_receipt_header_invalid",
            "Usage header changed its reserved length.",
        ));
    }
    writer.bytes[..prefix_len].copy_from_slice(prefix.as_bytes());
    let total_bytes = committed_bytes + writer.bytes.len();
    let document =
        String::from_utf8(writer.bytes).map_err(|error| encoding_error(error.to_string()))?;
    Ok(PreparedReceipt {
        document,
        total_bytes,
    })
}

fn usage_prefix(usage: &super::ExecutionUsage) -> Result<String, AuthoringError> {
    use std::fmt::Write as _;
    let value = serde_json::to_value(usage).map_err(|error| encoding_error(error.to_string()))?;
    let fields = value.as_object().ok_or_else(|| {
        receipt_error(
            "authoring_receipt_header_invalid",
            "Usage must be a fixed object of unsigned counters.",
        )
    })?;
    let mut prefix = String::from("{\"usage\":{");
    for (index, (key, value)) in fields.iter().enumerate() {
        let number = value.as_u64().ok_or_else(|| {
            receipt_error(
                "authoring_receipt_header_invalid",
                "Usage fields must be unsigned counters.",
            )
        })?;
        if index != 0 {
            prefix.push(',');
        }
        let key = serde_json::to_string(key).map_err(|error| encoding_error(error.to_string()))?;
        write!(&mut prefix, "{key}:{number:<20}").expect("writing to String is infallible");
    }
    prefix.push_str("},\"payload\":");
    Ok(prefix)
}

fn receipt_error(code: &'static str, reason: &'static str) -> AuthoringError {
    AuthoringError::Request {
        diagnostic: RequestFailure {
            code,
            phase: ExecutionPhase::Receipt,
            field_path: "receipt",
            reason: reason.into(),
        },
    }
}

fn encoding_error(reason: String) -> AuthoringError {
    AuthoringError::Request {
        diagnostic: RequestFailure {
            code: "authoring_result_encoding",
            phase: ExecutionPhase::Receipt,
            field_path: "result",
            reason,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authoring::{CancellationToken, ExecutionLimits, ExecutionUsage};
    use serde_json::json;

    fn run(
        payload: &impl Serialize,
        prior: usize,
        cap: usize,
        limits: &ExecutionLimits,
    ) -> Result<(PreparedReceipt, ExecutionUsage), AuthoringError> {
        let (result, usage) = control::run(
            limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || encode_receipt(payload, prior, cap),
        )
        .map_err(|diagnostic| AuthoringError::Execution { diagnostic })?;
        result.map(|receipt| (receipt, usage))
    }
    fn code<T: std::fmt::Debug>(result: Result<T, AuthoringError>) -> &'static str {
        match result.unwrap_err() {
            AuthoringError::Execution { diagnostic } => diagnostic.code,
            AuthoringError::Request { diagnostic } => diagnostic.code,
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn receipt_counters_include_encoding_and_match_final_scope_usage() {
        let payload = json!({"step":1,"result":{"value":"a\"b\\c\n"}});
        let (receipt, usage) = run(&payload, 19, 8192, &ExecutionLimits::default()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&receipt.document).unwrap();
        assert_eq!(parsed["payload"], payload);
        assert_eq!(parsed["usage"], serde_json::to_value(&usage).unwrap());
        assert_eq!(usage.result_bytes_peak, receipt.document.len());
        assert_eq!(receipt.total_bytes, 19 + receipt.document.len());
        assert!(usage.work > receipt.document.len() as u64);
        let (repeat, repeated_usage) =
            run(&payload, 19, 8192, &ExecutionLimits::default()).unwrap();
        assert_eq!(repeat.document, receipt.document);
        assert_eq!(repeated_usage, usage);
    }

    #[test]
    fn usage_header_length_is_fixed_even_at_maximum_counter_values() {
        let max = ExecutionUsage {
            work: u64::MAX,
            pending_events_peak: usize::MAX,
            bucket_events_peak: usize::MAX,
            effect_commands_peak: usize::MAX,
            geometry_cells_peak: usize::MAX,
            retained_entities_peak: usize::MAX,
            definitions_peak: usize::MAX,
            metadata_bytes_peak: usize::MAX,
            result_bytes_peak: usize::MAX,
            clone_slots_peak: usize::MAX,
        };
        let zero = usage_prefix(&ExecutionUsage::default()).unwrap();
        let maximum = usage_prefix(&max).unwrap();
        assert_eq!(zero.len(), maximum.len());
        let value: serde_json::Value = serde_json::from_str(&(maximum + "null}")).unwrap();
        assert_eq!(value["usage"], serde_json::to_value(max).unwrap());
    }

    #[test]
    fn receipt_preserves_a_larger_earlier_result_peak_in_the_same_attempt() {
        let ((earlier, receipt), usage) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                let earlier = encode_result(&"x".repeat(4096)).unwrap();
                let receipt = encode_receipt(&json!(null), 0, 8192).unwrap();
                (earlier, receipt)
            },
        )
        .unwrap();
        assert!(earlier.len() > receipt.document.len());
        assert_eq!(usage.result_bytes_peak, earlier.len());
        let parsed: serde_json::Value = serde_json::from_str(&receipt.document).unwrap();
        assert_eq!(parsed["usage"], serde_json::to_value(usage).unwrap());
    }

    #[test]
    fn receipt_bytes_and_work_are_inclusive_and_failed_append_keeps_prior_bytes() {
        let payload = json!({"accepted":true});
        let (first, usage) = run(&payload, 0, 8192, &ExecutionLimits::default()).unwrap();
        let size = first.document.len();
        let exact = ExecutionLimits {
            max_result_bytes: size,
            max_work: usage.work,
            ..Default::default()
        };
        let (second, _) = run(&payload, size, 2 * size, &exact).unwrap();
        assert_eq!(second.document, first.document);
        assert_eq!(second.total_bytes, 2 * size);
        assert_eq!(
            code(run(&payload, size, 2 * size - 1, &exact)),
            "authoring_receipt_limit"
        );
        assert_eq!(
            code(run(
                &payload,
                0,
                8192,
                &ExecutionLimits {
                    max_result_bytes: size - 1,
                    ..exact.clone()
                }
            )),
            "authoring_result_limit"
        );
        assert_eq!(
            code(run(
                &payload,
                0,
                8192,
                &ExecutionLimits {
                    max_work: usage.work - 1,
                    ..exact.clone()
                }
            )),
            "authoring_work_limit"
        );
        let (retry, _) = run(&payload, size, 2 * size, &exact).unwrap();
        assert_eq!(retry.document, second.document);
        assert_eq!(first.total_bytes, size);
        for cap in [0, 64 * 1024 * 1024 + 1] {
            assert_eq!(
                code(run(&payload, 0, cap, &exact)),
                "authoring_receipt_limits_invalid"
            );
        }
        assert_eq!(
            code(run(&payload, 2, 1, &exact)),
            "authoring_receipt_limits_invalid"
        );
    }

    #[test]
    fn cancellation_after_receipt_and_payload_encoding_error_never_return_a_prepared_receipt() {
        let token = CancellationToken::default();
        let result = control::run(
            &ExecutionLimits::default(),
            &token,
            ExecutionPhase::TrialExecution,
            || {
                let result = encode_receipt(&json!({"ready":true}), 0, 8192);
                assert!(result.is_ok());
                token.cancel();
                result
            },
        );
        assert_eq!(result.unwrap_err().code, "authoring_cancelled");
        struct Fails;
        impl Serialize for Fails {
            fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
                Err(serde::ser::Error::custom("actual payload rejection"))
            }
        }
        assert_eq!(
            code(run(&Fails, 0, 8192, &ExecutionLimits::default())),
            "authoring_result_encoding"
        );
        assert_eq!(
            code(encode_receipt(&json!(null), 0, 8192)),
            "authoring_scope_required"
        );
    }
}
