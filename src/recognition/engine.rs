use crate::models::scan_json;

pub struct Engine {
    scan_data: Vec<scan_json::Page>,
    card_type: u8,
    // todo
    // vx_model: torch::onnx,
    // number_model: torch::onnx,
}

impl Engine {
    pub fn new(scan_data: Vec<scan_json::Page>, card_type: u8) -> Self {
        Engine {
            scan_data,
            card_type,
        }
    }
}