//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md

mod data_type;
mod features;
mod structure_detect;
mod smart_detect;

pub use data_type::DataType;
pub use features::CompressionFeatures;
pub use structure_detect::{
    StructureResult,
    compute_structure,
    compute_structure_at_depth,
    find_optimal_depth,
    compare_depths,
};
pub use smart_detect::{
    SmartDetectionResult,
    smart_detect,
    would_bloat,
};
