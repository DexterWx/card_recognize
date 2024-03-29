use crate::models::scan_json;

pub struct Engine {
    scan_data: scan_json::InputScan
    // todo
    // vx_model: torch::onnx,
    // number_model: torch::onnx,
}

impl Engine {
    pub fn new(scan_data: scan_json::InputScan) -> Self {
        Engine {
            scan_data
        }
    }
    pub fn get_scan_data(&self) -> &scan_json::InputScan {
        &self.scan_data
    }
}