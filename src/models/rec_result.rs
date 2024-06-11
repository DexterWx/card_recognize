/*
    输出结构
*/

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use super::{card::MyPoint, scan_json::{AreaAssistPoint, Coordinate, InputScan, InputSecond, Value}};


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
    pub code: u8
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page{
    pub has_page: bool,
    pub page_size: Option<PageSize>,
    pub image_source: Option<String>,
    pub image_rotated: Option<String>,
    pub image_rendering: Option<String>,
    pub area_assist_points: Option<Vec<AreaAssistPoint>>,
    pub assist_points_move_op: HashMap<i32, MoveOperation>,
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
    pub coordinate: Option<Coordinate>,
    pub _value: Option<Value>,
    pub invalue: Option<Value>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageSize{
    pub w: i32,
    pub h: i32
}


impl Value {
    pub fn to_float(&self) -> Option<f32> {
        match self {
            Value::String(_) => None,
            Value::Integer(_) => None, // Return None if it's an Integer
            Value::Float(f) => Some(*f),
        }
    }

    pub fn to_i32(&self) -> Option<i32> {
        match self {
            Value::String(_) => None,
            Value::Integer(i) => Some(*i),
            Value::Float(_) => None, // Return None if it's a Float
        }
    }

    pub fn to_string(&self) -> Option<String> {
        match self {
            Value::String(s) => Some(s.clone()),
            Value::Integer(_) => None, // Return None if it's an Integer
            Value::Float(_) => None, // Return None if it's a Float
        }
    }
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
                    area_assist_points: None,
                    assist_points_move_op: HashMap::new(),
                    recognizes: page.recognizes.iter().map(|rec| {
                        Recognize {
                            rec_id: rec.rec_id.clone(),
                            rec_type: rec.rec_type,
                            rec_options: rec.options.iter().map(|rec_value|{
                                RecOption{
                                    value: None,
                                    coordinate: None,
                                    _value: None,
                                    invalue: rec_value.value.clone()
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

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputRecSecond{
    pub task_id: String,
    pub pages: Vec<PageSecond>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageSecond{
    pub image_rendering: Option<String>,
    pub recognizes: Vec<Recognize>
}

impl OutputRecSecond{
    pub fn new(input: &InputSecond) -> Self {
        OutputRecSecond{
            task_id: "".to_string(),
            pages: input.pages.iter().map(|page| {
                PageSecond{
                    image_rendering: None,
                    recognizes: page.recognizes.iter().map(|rec| {
                        Recognize {
                            rec_id: rec.rec_id.clone(),
                            rec_type: rec.rec_type,
                            rec_options: rec.options.iter().map(|rec_value|{
                                RecOption{
                                    value: None,
                                    coordinate: None,
                                    _value: None,
                                    invalue: rec_value.value.clone()
                                }
                            }).collect()
                        }
                    }).collect()
                }
            }).collect()
        }
    }
}