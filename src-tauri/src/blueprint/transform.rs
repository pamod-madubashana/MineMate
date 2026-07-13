use super::types::Blueprint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlueprintTransform {
    None,
    Rotate90,
    Rotate180,
    Rotate270,
    FlipX,
    FlipZ,
}

impl BlueprintTransform {
    pub fn apply(&self, blueprint: &Blueprint) -> Blueprint {
        match self {
            Self::None => blueprint.clone(),
            Self::Rotate90 => rotate_blueprint(blueprint, 90),
            Self::Rotate180 => rotate_blueprint(blueprint, 180),
            Self::Rotate270 => rotate_blueprint(blueprint, 270),
            Self::FlipX => flip_blueprint(blueprint, true),
            Self::FlipZ => flip_blueprint(blueprint, false),
        }
    }
}

fn rotate_blueprint(blueprint: &Blueprint, degrees: u32) -> Blueprint {
    let mut result = blueprint.clone();
    match degrees {
        90 => {
            result.width = blueprint.length;
            result.length = blueprint.width;
            result.blocks = Vec::new();
            for y in 0..blueprint.height as usize {
                let mut new_layer = Vec::new();
                for z in 0..blueprint.width as usize {
                    let mut new_row = Vec::new();
                    for x in (0..blueprint.length as usize).rev() {
                        let block = blueprint.blocks.get(y).and_then(|l| l.get(x)).and_then(|r| r.get(z)).cloned().flatten();
                        new_row.push(block);
                    }
                    new_layer.push(new_row);
                }
                result.blocks.push(new_layer);
            }
        }
        180 => {
            result.blocks = Vec::new();
            for y in 0..blueprint.height as usize {
                let mut new_layer = Vec::new();
                for z in (0..blueprint.length as usize).rev() {
                    let mut new_row = Vec::new();
                    for x in (0..blueprint.width as usize).rev() {
                        let block = blueprint.blocks.get(y).and_then(|l| l.get(z)).and_then(|r| r.get(x)).cloned().flatten();
                        new_row.push(block);
                    }
                    new_layer.push(new_row);
                }
                result.blocks.push(new_layer);
            }
        }
        270 => {
            result.width = blueprint.length;
            result.length = blueprint.width;
            result.blocks = Vec::new();
            for y in 0..blueprint.height as usize {
                let mut new_layer = Vec::new();
                for z in (0..blueprint.width as usize).rev() {
                    let mut new_row = Vec::new();
                    for x in 0..blueprint.length as usize {
                        let block = blueprint.blocks.get(y).and_then(|l| l.get(x)).and_then(|r| r.get(z)).cloned().flatten();
                        new_row.push(block);
                    }
                    new_layer.push(new_row);
                }
                result.blocks.push(new_layer);
            }
        }
        _ => {}
    }
    result
}

fn flip_blueprint(blueprint: &Blueprint, flip_x: bool) -> Blueprint {
    let mut result = blueprint.clone();
    result.blocks = Vec::new();
    for y in 0..blueprint.height as usize {
        let mut new_layer = Vec::new();
        for z in 0..blueprint.length as usize {
            let mut new_row = Vec::new();
            for x in 0..blueprint.width as usize {
                let (src_x, src_z) = if flip_x {
                    (blueprint.width as usize - 1 - x, z)
                } else {
                    (x, blueprint.length as usize - 1 - z)
                };
                let block = blueprint.blocks.get(y).and_then(|l| l.get(src_z)).and_then(|r| r.get(src_x)).cloned().flatten();
                new_row.push(block);
            }
            new_layer.push(new_row);
        }
        result.blocks.push(new_layer);
    }
    result
}
