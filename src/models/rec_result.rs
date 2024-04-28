/*
    输出结构
*/

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use super::{card::MyPoint, scan_json::{Coordinate, InputScan}};


#[derive(Debug, Serialize, Deserialize)]
pub struct OutputRec{
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
    pub page_size: PageSize
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page{
    pub has_page: bool,
    pub page_size: Option<PageSize>,
    pub image_source: Option<String>,
    pub image_rotated: Option<String>,
    pub image_rendering: Option<String>,
    pub assist_points: Option<HashMap<i32, MoveOperation>>,
    pub recognizes: Vec<Recognize>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoveOperation{
    pub move_x: i32,
    pub move_y: i32,
    pub center: MyPoint,
    pub angle: f32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Recognize{
    pub rec_id: String,
    pub rec_type: u8,
    pub rec_options: Vec<RecOption>
}


#[derive(Debug, Serialize, Deserialize)]
pub struct RecOption{
    pub value: Option<Value>,
    pub coordinate: Option<Coordinate>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageSize{
    pub w: i32,
    pub h: i32
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)] // Allows using multiple types for the enum variants
pub enum Value {
    String(String),
    Integer(i32),
    Float(f32),
}

impl OutputRec{
    pub fn new(input: &InputScan) -> Self {
        OutputRec{
            task_id: "".to_string(),
            code: 0,
            message: "succeeded".to_string(),
            pages: input.pages.iter().map(|page| {
                Page{
                    has_page: false,
                    page_size: None,
                    image_source: None,
                    image_rendering: None,
                    image_rotated: None,
                    assist_points: None,
                    recognizes: page.recognizes.iter().map(|rec| {
                        Recognize {
                            rec_id: rec.rec_id.clone(),
                            rec_type: rec.rec_type,
                            rec_options: rec.options.iter().map(|_|{
                                RecOption{
                                    value: None,
                                    coordinate: None
                                }
                            }).collect()
                        }
                    }).collect()
                }
            }).collect(),
            images: Vec::new(),
        }
    }
}