/*
    输出结构
*/

use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Output{
    pub task_id: String,
    pub code: u8,
    pub message: String,
    pub pages: Vec<Page>,
    pub images: Vec<ImageStatus>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageStatus{
    pub image_source: String,
    pub code: u8,
    pub w: u32,
    pub h: u32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page{
    pub has_page: bool,
    pub image_source: Option<String>,
    pub image_rotated: Option<String>,
    pub image_rendering: Option<String>,
    pub recognizes: Option<Vec<Recognize>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Recognize{
    pub rec_id: String,
    pub rec_type: u8,
    pub value: RecValue
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecValue{
    pub single_item: Option<String>,
    pub multi_item: Option<Vec<String>>,
    pub numbers: Option<f32>,
    pub coordinates: Option<Coordinates>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Coordinates{
    pub x: u32,
    pub y: u32,
    pub h: u32,
    pub w: u32
}