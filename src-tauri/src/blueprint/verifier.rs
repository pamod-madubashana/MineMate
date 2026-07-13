use super::types::Blueprint;
use super::materials::estimate_materials;

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub block_count: u32,
    pub layer_count: u32,
}

impl VerificationResult {
    pub fn success(block_count: u32, layer_count: u32) -> Self {
        Self { valid: true, errors: Vec::new(), warnings: Vec::new(), block_count, layer_count }
    }

    pub fn failure(errors: Vec<String>) -> Self {
        Self { valid: false, errors, warnings: Vec::new(), block_count: 0, layer_count: 0 }
    }
}

pub fn verify_blueprint(blueprint: &Blueprint) -> VerificationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if blueprint.width == 0 || blueprint.height == 0 || blueprint.length == 0 {
        errors.push("Blueprint has zero dimensions".to_string());
        return VerificationResult::failure(errors);
    }

    if blueprint.blocks.len() != blueprint.height as usize {
        errors.push(format!("Layer count mismatch: expected {}, got {}", blueprint.height, blueprint.blocks.len()));
    }

    for (y, layer) in blueprint.blocks.iter().enumerate() {
        if layer.len() != blueprint.length as usize {
            errors.push(format!("Layer {}: row count mismatch: expected {}, got {}", y, blueprint.length, layer.len()));
        }
    }

    let materials = estimate_materials(blueprint);
    let block_count = materials.required.total();

    if block_count == 0 {
        warnings.push("Blueprint contains no blocks".to_string());
    }

    if errors.is_empty() {
        VerificationResult::success(block_count, blueprint.height)
    } else {
        VerificationResult::failure(errors)
    }
}
