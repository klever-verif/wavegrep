use crate::engine::scoped_signal_path;
use crate::error::WavepeekError;
use crate::waveform::{ResolvedSignal, SampledSignalState, Waveform};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BitRange {
    msb: u32,
    lsb: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProjectedSignal {
    pub source: ResolvedSignal,
    pub path: String,
    range: Option<BitRange>,
}

impl ProjectedSignal {
    #[cfg(test)]
    pub(crate) fn unprojected(source: ResolvedSignal) -> Self {
        Self {
            path: source.path.clone(),
            source,
            range: None,
        }
    }

    pub(crate) fn width(&self) -> u32 {
        self.range
            .map_or(self.source.width, |range| range.msb - range.lsb + 1)
    }

    pub(crate) fn project_sample(
        &self,
        sample: SampledSignalState,
    ) -> Result<SampledSignalState, WavepeekError> {
        let bits = match self.range {
            Some(_) => self.project_bits(sample.bits.as_deref())?,
            None => sample.bits,
        };

        Ok(SampledSignalState {
            path: self.path.clone(),
            width: self.width(),
            bits,
        })
    }

    pub(crate) fn project_bits(&self, bits: Option<&str>) -> Result<Option<String>, WavepeekError> {
        let Some(bits) = bits else {
            return Ok(None);
        };
        let Some(range) = self.range else {
            return Ok(Some(bits.to_string()));
        };
        let width = usize::try_from(self.source.width).map_err(|_| {
            WavepeekError::Internal("signal width exceeds platform limits".to_string())
        })?;
        let start = width - 1 - range.msb as usize;
        let end = width - range.lsb as usize;
        Ok(Some(
            bits.get(start..end)
                .ok_or_else(|| {
                    WavepeekError::Internal(format!(
                        "sampled width for signal '{}' does not match resolved width {}",
                        self.source.path, self.source.width
                    ))
                })?
                .to_string(),
        ))
    }
}

pub(crate) fn resolve_projected_signal(
    waveform: &Waveform,
    token: &str,
    scope: Option<&str>,
) -> Result<ProjectedSignal, WavepeekError> {
    let canonical_path = scoped_signal_path(token, scope);
    match waveform.resolve_signals(std::slice::from_ref(&canonical_path)) {
        Ok(mut resolved) => {
            let source = resolved.remove(0);
            return Ok(ProjectedSignal {
                path: source.path.clone(),
                source,
                range: None,
            });
        }
        Err(WavepeekError::SignalNotFound(_)) => {}
        Err(error) => return Err(error),
    }

    let Some((base_token, range)) = parse_trailing_range(token)? else {
        let source = waveform
            .resolve_signals_with_diagnostics(
                std::slice::from_ref(&canonical_path),
                std::slice::from_ref(&token.to_string()),
                scope,
            )?
            .remove(0);
        return Ok(ProjectedSignal {
            path: source.path.clone(),
            source,
            range: None,
        });
    };

    let base_path = scoped_signal_path(base_token, scope);
    let source = waveform
        .resolve_signals_with_diagnostics(
            std::slice::from_ref(&base_path),
            std::slice::from_ref(&base_token.to_string()),
            scope,
        )?
        .remove(0);
    if range.msb >= source.width {
        return Err(WavepeekError::Signal(format!(
            "projection '[{}:{}]' is outside signal '{}' width {}",
            range.msb, range.lsb, source.path, source.width
        )));
    }

    Ok(ProjectedSignal {
        path: format!("{}[{}:{}]", source.path, range.msb, range.lsb),
        source,
        range: Some(range),
    })
}

fn parse_trailing_range(token: &str) -> Result<Option<(&str, BitRange)>, WavepeekError> {
    let Some(open) = token.rfind('[') else {
        return Ok(None);
    };
    let contents = &token[open + 1..];
    if !contents.contains(':') {
        return Ok(None);
    }
    if !token.ends_with(']') {
        return Err(invalid_projection(token, "range must end with ']'"));
    }

    let base = &token[..open];
    if base.is_empty() {
        return Err(invalid_projection(
            token,
            "base signal path must not be empty",
        ));
    }
    if base.ends_with(']') {
        return Err(invalid_projection(
            token,
            "chained and multidimensional projections are not supported",
        ));
    }

    let range = &contents[..contents.len() - 1];
    let Some((msb, lsb)) = range.split_once(':') else {
        return Ok(None);
    };
    if msb.is_empty() || lsb.is_empty() || lsb.contains(':') {
        return Err(invalid_projection(
            token,
            "expected one static range '[msb:lsb]'",
        ));
    }
    let msb = parse_bound(token, msb)?;
    let lsb = parse_bound(token, lsb)?;
    if msb < lsb {
        return Err(invalid_projection(
            token,
            "msb must be greater than or equal to lsb",
        ));
    }

    Ok(Some((base, BitRange { msb, lsb })))
}

fn parse_bound(token: &str, bound: &str) -> Result<u32, WavepeekError> {
    if !bound.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid_projection(
            token,
            "bounds must be non-negative decimal integers",
        ));
    }
    bound
        .parse::<u32>()
        .map_err(|_| invalid_projection(token, "bounds exceed the supported integer range"))
}

fn invalid_projection(token: &str, detail: &str) -> WavepeekError {
    WavepeekError::Signal(format!("invalid flat projection '{token}': {detail}"))
}

#[cfg(test)]
mod tests {
    use super::{BitRange, parse_trailing_range};

    #[test]
    fn parses_only_one_static_trailing_range() {
        assert_eq!(parse_trailing_range("top.data").unwrap(), None);
        assert_eq!(parse_trailing_range("top.mem[0]").unwrap(), None);
        assert_eq!(
            parse_trailing_range("top.data[7:4]").unwrap(),
            Some(("top.data", BitRange { msb: 7, lsb: 4 }))
        );
        assert_eq!(
            parse_trailing_range("top.data[0:0]").unwrap(),
            Some(("top.data", BitRange { msb: 0, lsb: 0 }))
        );

        for invalid in [
            "top.data[4:7]",
            "top.data[-1:0]",
            "top.data[WIDTH:0]",
            "top.data[7:]",
            "top.data[7:4:1]",
            "top.data[7:4",
            "top.data[7:4][3:2]",
        ] {
            assert!(
                parse_trailing_range(invalid).is_err(),
                "{invalid} should fail"
            );
        }
    }
}
