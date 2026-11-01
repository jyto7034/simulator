//! Bounded host-only OS samples. VmHWM is a process lifetime peak, not a window.
use serde::Serialize;
use std::io::Read;
const STATUS_BYTE_CAP: usize = 64 * 1024;
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RssSample {
    Available {
        resident_bytes: u64,
        lifetime_peak_bytes: u64,
    },
    Missing {
        reason: &'static str,
    },
    Unsupported {
        platform: &'static str,
    },
}
pub fn process_rss() -> RssSample {
    #[cfg(target_os = "linux")]
    {
        let Ok(file) = std::fs::File::open("/proc/self/status") else {
            return RssSample::Missing {
                reason: "proc status open failed",
            };
        };
        let mut bytes = Vec::new();
        if file
            .take(STATUS_BYTE_CAP as u64 + 1)
            .read_to_end(&mut bytes)
            .is_err()
        {
            return RssSample::Missing {
                reason: "proc status read failed",
            };
        }
        if bytes.len() > STATUS_BYTE_CAP {
            return RssSample::Missing {
                reason: "proc status exceeded 64KiB",
            };
        }
        parse_status(&bytes)
    }
    #[cfg(not(target_os = "linux"))]
    {
        RssSample::Unsupported {
            platform: std::env::consts::OS,
        }
    }
}
fn parse_status(bytes: &[u8]) -> RssSample {
    if bytes.len() > STATUS_BYTE_CAP {
        return RssSample::Missing {
            reason: "proc status exceeded 64KiB",
        };
    }
    let Ok(text) = std::str::from_utf8(bytes) else {
        return RssSample::Missing {
            reason: "proc status was not UTF8",
        };
    };
    let field = |name: &str| -> Option<u64> {
        let mut values = text.lines().filter_map(|line| line.strip_prefix(name));
        let line = values.next()?;
        if values.next().is_some() {
            return None;
        }
        let mut parts = line.split_whitespace();
        let bytes = parts.next()?.parse::<u64>().ok()?.checked_mul(1024)?;
        if parts.next()? != "kB" || parts.next().is_some() {
            return None;
        }
        Some(bytes)
    };
    match (field("VmRSS:"), field("VmHWM:")) {
        (Some(resident_bytes), Some(lifetime_peak_bytes)) => RssSample::Available {
            resident_bytes,
            lifetime_peak_bytes,
        },
        _ => RssSample::Missing {
            reason: "proc RSS/HWM missing, malformed, duplicated or overflowed",
        },
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cmrp_rss_units_and_missing_are_explicit_and_bounded() {
        assert!(matches!(
            parse_status(b"VmRSS: 12 kB\nVmHWM: 34 kB\n"),
            RssSample::Available {
                resident_bytes: 12288,
                lifetime_peak_bytes: 34816
            }
        ));
        for bytes in [
            b"".as_slice(),
            b"VmRSS: 12 MB\nVmHWM: 34 kB\n",
            b"VmRSS: 18446744073709551615 kB\nVmHWM: 34 kB\n",
            b"VmRSS: 12 kB\nVmRSS: 14 kB\nVmHWM: 34 kB\n",
        ] {
            assert!(matches!(parse_status(bytes), RssSample::Missing { .. }));
        }
        assert!(matches!(
            parse_status(&vec![0; STATUS_BYTE_CAP + 1]),
            RssSample::Missing { .. }
        ));
        assert!(matches!(
            process_rss(),
            RssSample::Available { .. } | RssSample::Missing { .. } | RssSample::Unsupported { .. }
        ));
    }
}
