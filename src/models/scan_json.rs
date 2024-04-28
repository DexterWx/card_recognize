/*
    排版标注信息结构，从scanjson洗出来的。
*/

use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct InputScan {
    pub pages: Vec<Page>,
    pub card_type: u8
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputImage {
    pub task_id: String,
    pub images: Vec<String>,
    pub calling_type: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub card_columns: u8, 
    pub model_size: ModelSize,
    pub model_points: Vec<ModelPoint>,
    pub page_number_points: Vec<PageNumberPoint>,
    pub assist_points: Option<Vec<AssistPoint>>,
    pub recognizes: Vec<Recognition>,
    pub model_points_4: Option<[ModelPoint;4]>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct AssistPoint {
    pub right: Coordinate,
    pub left: Coordinate
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ModelSize {
    pub w: i32,
    pub h: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ModelPoint {
    pub point_type: u8,
    pub coordinate: Coordinate,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PageNumberPoint {
    pub fill_rate: f32,
    pub coordinate: Coordinate,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Recognition {
    pub rec_id: String,
    pub rec_type: u8,
    pub options: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub value: Option<Value>, // Assuming value can be a string for all types
    pub coordinate: Coordinate,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)] // Allows using multiple types for the enum variants
pub enum Value {
    String(String),
    Integer(i32),
    Float(f32),
}


impl InputScan{
    /// 处理输入数据
    pub fn renew(&self) -> Self{
        let mut pages = Vec::new();
        for page in &self.pages{
            pages.push(page.renew());
        }
        Self { pages: pages, card_type: self.card_type }
    }
}

impl Page {
    pub fn renew(&self) -> Self {
        assert!(self.model_points.len() >= 4);

        let lt = self.model_points[0].clone();
        let rd = self.model_points[self.model_points.len() - 1].clone();
        let rt:ModelPoint;
        let ld:ModelPoint;
        match self.card_columns{
            1 => {
                rt = self.model_points[1].clone();
                ld = self.model_points[4].clone();
            }
            2 => {
                rt = self.model_points[2].clone();
                ld = self.model_points[6].clone();
            }
            3 => {
                rt = self.model_points[3].clone();
                ld = self.model_points[8].clone();
            }
            4 => {
                rt = self.model_points[4].clone();
                ld = self.model_points[10].clone();
            }
            _ => {
                panic!("Unhandled card_columns value: {}", self.card_columns);
            }
        }
        Self {
            card_columns:self.card_columns,
            model_size:self.model_size,
            model_points:self.model_points.clone(),
            page_number_points:self.page_number_points.clone(),
            assist_points:self.assist_points.clone(),
            recognizes:self.recognizes.clone(),
            model_points_4:Some([lt, rt, ld, rd]),
        }
    }
}