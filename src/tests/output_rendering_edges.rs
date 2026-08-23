use crate::engine::property::{PropertyCaptureRow, PropertyResultKind};
use crate::engine::{CommandData, HumanRenderOptions};

use super::*;

#[test]
fn renders_property_state_changes() {
    let properties = CommandData::Property(vec![
        PropertyCaptureRow {
            time: "0ns".to_string(),
            sample_time: "0ns".to_string(),
            kind: PropertyResultKind::Assert,
        },
        PropertyCaptureRow {
            time: "1ns".to_string(),
            sample_time: "1ns".to_string(),
            kind: PropertyResultKind::Deassert,
        },
    ]);

    assert!(render_human(&properties, HumanRenderOptions::default()).contains("@1ns deassert"));
}
