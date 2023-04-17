use core::panic;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::Path;
use std::{env, vec};

use png::ColorType;

// cargo run -- l layer_images/0_53x53.png layer_files/0_53x53.layer 0 -1 0 -2
// cargo run -- n nav_images/0_53x53.png nav_files/0_53x53.nav 0 1 -1 -2

// cargo run -- l layer_images/crowd_flow.png layer_files/crowd_flow.layer 0 -1 -2
// cargo run -- n nav_images/crowd_flow.png nav_files/crowd_flow.nav 0 1 -1 -2
fn main() {
    let args: Vec<String> = env::args().collect();

    let method = args.get(1).unwrap();
    let source = Path::new(args.get(2).unwrap());
    let dest = Path::new(args.get(3).unwrap());
    dbg!(&args);

    if !source.exists() {
        panic!(
            "The source file {:?} doesn't exist",
            source.to_str().unwrap()
        );
    }

    let mut new_vals: Vec<f32> = vec![];
    let mut cur_val = 0;
    for i in &args[4..] {
        let parsed_val = i.parse::<f32>();

        match parsed_val {
            Ok(val) => {
                new_vals.push(val);
                cur_val += 1;
            }
            Err(err) => panic!("Argument at position {} is incorrect, {}", cur_val, err),
        }
    }

    let image = get_image_from_path(source);
    image.save(new_vals.clone(), dest, method);
    // image.save(new_vals, dest, method);
    // image.save(new_vals.clone(), Path::new("dumping_ground/0.nav"), method);
    // image.save(new_vals.clone(), Path::new("dumping_ground/2.nav"), method);
    // image.save(new_vals.clone(), Path::new("dumping_ground/4.nav"), method);
    // image.save(new_vals.clone(), Path::new("dumping_ground/6.nav"), method);
    // image.save(new_vals.clone(), Path::new("dumping_ground/8.nav"), method);
    // image.save(new_vals.clone(), Path::new("dumping_ground/10.nav"), method);
    // image.save(new_vals.clone(), Path::new("dumping_ground/12.nav"), method);
    // image.save(new_vals.clone(), Path::new("dumping_ground/14.nav"), method);
    // image.save(new_vals.clone(), Path::new("dumping_ground/16.nav"), method);
    // image.save(new_vals.clone(), Path::new("dumping_ground/18.nav"), method);

    // image.save(new_vals.clone(), Path::new("dumping_ground/1.nav"), method);
    // image.save(new_vals.clone(), Path::new("dumping_ground/3.nav"), method);
    // image.save(new_vals.clone(), Path::new("dumping_ground/5.nav"), method);
    // image.save(new_vals.clone(), Path::new("dumping_ground/7.nav"), method);
    // image.save(new_vals.clone(), Path::new("dumping_ground/9.nav"), method);
    // image.save(new_vals.clone(), Path::new("dumping_ground/11.nav"), method);
    // image.save(new_vals.clone(), Path::new("dumping_ground/13.nav"), method);
    // image.save(new_vals.clone(), Path::new("dumping_ground/15.nav"), method);
    // image.save(new_vals.clone(), Path::new("dumping_ground/17.nav"), method);
    // image.save(new_vals.clone(), Path::new("dumping_ground/19.nav"), method);
}

fn get_image_from_path(path: &Path) -> Image {
    let decoder = png::Decoder::new(File::open(path).unwrap());
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    println!("{:?}", info);

    let bytes = &buf[..info.buffer_size()];
    return get_image(bytes, info.width, info.height, info.color_type);
}

fn get_image(bytes: &[u8], w: u32, h: u32, color_type: ColorType) -> Image {
    let mut i = 0;
    let mut image: Vec<Vec<u32>> = vec![];
    let mut counts: HashMap<u32, u32> = HashMap::new();

    for _y in 0..h {
        let mut row: Vec<u32> = vec![];
        for _x in 0..w {
            //convert pixels to u32
            let val = match color_type {
                ColorType::Rgba => as_u32([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]),
                ColorType::Rgb => as_u32([bytes[i], bytes[i + 1], bytes[i + 2], 0]),
                ColorType::Grayscale => as_u32([bytes[i], 0, 0, 0]),
                ColorType::Indexed => todo!(),
                ColorType::GrayscaleAlpha => todo!(),
            };

            if counts.contains_key(&val) {
                if let Some(count) = counts.get_mut(&val) {
                    *count += 1;
                }
            } else {
                counts.insert(val, 1);
            }
            row.push(val);

            i += match color_type {
                ColorType::Rgba => 4,
                ColorType::Rgb => 3,
                ColorType::Grayscale => 1,
                ColorType::Indexed => todo!(),
                ColorType::GrayscaleAlpha => todo!(),
            };
        }
        image.push(row);
    }
    return Image::new(image, counts);
}
fn as_u32(arr: [u8; 4]) -> u32 {
    return ((arr[0] as u32) << 24)
        + ((arr[1] as u32) << 16)
        + ((arr[2] as u32) << 8)
        + (arr[3] as u32);
}

#[derive(Debug)]
pub struct Image {
    image: Vec<Vec<u32>>,
    counts: HashMap<u32, u32>,
}
impl Image {
    pub fn new(image: Vec<Vec<u32>>, counts: HashMap<u32, u32>) -> Self {
        return Self { image, counts };
    }

    fn save(&self, new_vals: Vec<f32>, dest: &Path, method: &str) {
        let data = match method {
            "l" => format!("{{\"data\":{:?}}}", self.to_layer(new_vals)),
            "n" => format!("{{\"graph\":{:?}}}", self.to_nav(new_vals)),
            &_ => todo!(),
        };

        fs::write(dest, data).expect("Unable to write file");
    }

    fn to_layer(&self, new_vals: Vec<f32>) -> Vec<Vec<f32>> {
        let mut layer: Vec<Vec<f32>> = vec![];

        //order counts by size
        let mut ordered_count: Vec<(u32, u32)> = self.counts.clone().into_iter().collect();
        println!("Counts: {:?}", ordered_count);
        ordered_count.sort_by(|x, y| y.1.cmp(&x.1));
        assert!(
            new_vals.len() == ordered_count.len(),
            "Need {} new values",
            ordered_count.len()
        );
        println!("Counts: {:?}", ordered_count);

        //create mapping
        let mut mapping: HashMap<u32, f32> = HashMap::new();
        for i in 0..new_vals.len() {
            mapping.insert(ordered_count[i].0, new_vals[i]);
        }
        println!("Using mapping: {:?}", mapping);

        //apply mapping
        for row in &self.image {
            let mut mapped_row = vec![];
            for val in row {
                mapped_row.push(mapping[val]);
            }
            layer.push(mapped_row);
        }

        return layer;
    }

    fn to_nav(&self, new_vals: Vec<f32>) -> Vec<Vec<i32>> {
        let mut layer: Vec<Vec<i32>> = vec![];

        //order counts by size
        let mut ordered_count: Vec<(u32, u32)> = self.counts.clone().into_iter().collect();
        println!("Counts: {:?}", ordered_count);
        ordered_count.sort_by(|x, y| y.1.cmp(&x.1));
        assert!(
            new_vals.len() == ordered_count.len(),
            "Need {} new values",
            ordered_count.len()
        );
        println!("Counts: {:?}", ordered_count);

        //create mapping
        let mut mapping: HashMap<u32, i32> = HashMap::new();
        for i in 0..new_vals.len() {
            mapping.insert(ordered_count[i].0, new_vals[i] as i32);
        }
        println!("Using mapping: {:?}", mapping);

        //apply mapping
        for row in &self.image {
            let mut mapped_row = vec![];
            for val in row {
                mapped_row.push(mapping[val]);
            }
            layer.push(mapped_row);
        }

        return layer;
    }
}
