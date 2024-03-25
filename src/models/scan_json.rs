/*
    排版标注信息结构，从scanjson洗出来的。
*/

use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Input1 {
    pub pages: Vec<Page>,
    pub card_type: u8
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input2 {
    pub task_id: String,
    pub images: Vec<String>,
    pub calling_type: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub card_columns: u8,
    pub model_size: ModelSize,
    pub model_points: Vec<LocationPoint>,
    pub page_number_points: Vec<PageNumberPoint>,
    pub recognizes: Vec<Recognition>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelSize {
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationPoint {
    pub point_type: u32,
    pub coordinate: Coordinate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Coordinate {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageNumberPoint {
    pub fill_rate: f32,
    pub coordinate: Coordinate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Recognition {
    pub rec_id: String,
    pub rec_type: u8,
    pub options: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub value: Option<Value>, // Assuming value can be a string for all types
    pub coordinate: Coordinate,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)] // Allows using multiple types for the enum variants
pub enum Value {
    String(String),
    Integer(i32),
    Float(f32),
}