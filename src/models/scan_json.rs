/*
    排版标注信息结构，从scanjson洗出来的。
*/

use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct InputScan {
    pub pages: Vec<Page>,
    pub card_type: u8
}

impl InputScan{
    /// 过滤一些不需要的信息
    pub fn renew(input: Self) -> Self{
        let mut pages = Vec::new();
        for page in input.pages{
            pages.push(Page::renew(page));
        }
        Self { pages: pages, card_type: input.card_type }
    }
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
    pub recognizes: Vec<Recognition>,
    pub model_points_4: Option<[ModelPoint;4]>,
}

impl Page {
    pub fn renew(page:Self) -> Self {
        assert!(page.model_points.len() >= 4);

        let lt = page.model_points[0].clone();
        let rd = page.model_points[page.model_points.len() - 1].clone();
        let mut rt:ModelPoint;
        let mut ld:ModelPoint;
        match page.card_columns{
            1 => {
                rt = page.model_points[1].clone();
                ld = page.model_points[4].clone();
            }
            2 => {
                rt = page.model_points[2].clone();
                ld = page.model_points[6].clone();
            }
            3 => {
                rt = page.model_points[3].clone();
                ld = page.model_points[8].clone();
            }
            4 => {
                rt = page.model_points[4].clone();
                ld = page.model_points[10].clone();
            }
            _ => {
                panic!("Unhandled card_columns value: {}", page.card_columns);
            }
        }
        Self {
            card_columns:page.card_columns,
            model_size:page.model_size,
            model_points:page.model_points,
            page_number_points:page.page_number_points,
            recognizes:page.recognizes,
            model_points_4:Some([lt, rt, ld, rd]),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ModelSize {
    pub w: i32,
    pub h: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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