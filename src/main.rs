use image::{DynamicImage, GenericImageView, ImageBuffer, Luma, Pixel};
use lib::ocr;
use lib::ocr::Ocr;
use rand::rngs::ThreadRng;
use std::io::Cursor;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use template_matching::{MatchTemplateMethod, TemplateMatcher, find_extremes};
extern crate config;
use config::{Config, File};
use rand::{thread_rng, Rng};

#[derive(Debug, Default, serde_derive::Deserialize, PartialEq, Eq)]
struct AppConfig {
    ip_addr: Vec<String>,
    goods_name: String,
    goods_name_type: String,
    map_x: String,
    map_y: String,
}

struct App {
    matcher: TemplateMatcher,
    comment: Arc<Comment>,
    config: Arc<AppConfig>,
    paddle_ocr: ocr::paddle::PaddleOcr,
}
struct Comment {
    map_image: ImageBuffer<Luma<f32>, Vec<f32>>,
    map_map_image: ImageBuffer<Luma<f32>, Vec<f32>>,
    confirm_image: ImageBuffer<Luma<f32>, Vec<f32>>,
    confirm_image2: ImageBuffer<Luma<f32>, Vec<f32>>,
    backpack_image: ImageBuffer<Luma<f32>, Vec<f32>>,
    x_image: ImageBuffer<Luma<f32>, Vec<f32>>,
    rows: Vec<(u32, u32)>,
    goods: (u32, u32),
}
impl App {
    fn map_position(&mut self, v: &str) {
        //1144
        let mut flag = false;
        let x: u32 = self.config.map_x.parse().unwrap();
        let y: u32 = self.config.map_y.parse().unwrap();
        println!("尝试找到地图位置");
        while true {

            adb_Click(v, "1267", "61");

            thread::sleep(time::Duration::from_millis(3500));
            let screencap = screencap_device(v);
            let result = match_template(
                &screencap.to_luma32f(),
                &self.comment.map_map_image,
                &mut self.matcher,
            );
            if result.min_value < 13_f32 {
                println!("地图已经打开,查看是否到达指定位置");
                adb_Click(v,  &self.config.map_x,  &self.config.map_y);
                flag = true;
                thread::sleep(time::Duration::from_millis(2500));
                let rgb = screencap.get_pixel(x, y).to_rgb();
                println!("是否到地图指定位置{:?}", rgb);
                if rgb[0] > 240 && rgb[1] > 240 && rgb[2] < 10 {
                    println!("到地图指定位置{:?}", rgb);
                    break;
                }
            }

        }
    }
    fn backpack_open(&mut self, v: &str) -> Result<template_matching::Extremes, String> {
        let mut i = 0;
        while i < 10 {
            println!("点击背包");
            adb_Click(v, "1245", "255");
            let mut i2 = 0;
            while i2 < 3 {
                thread::sleep(time::Duration::from_millis(2500));
                let screencap = screencap_device(v).to_luma32f();
                let result =
                    match_template(&screencap, &self.comment.backpack_image, &mut self.matcher);
                if result.min_value <= 18_f32 {
                    println!("已经打开背包");
                    thread::sleep(time::Duration::from_millis(600));
                    return Ok(result);
                } else {
                }
                i2 += 1;
            }
            println!("未发现背包，再次尝试");
            i += 1;
        }
        Err("点击背包错误".to_string())
    }
    fn backpack_goods_discard(&mut self, vv: &str, result: template_matching::Extremes) {
        println!("ok,整理背包");
        thread::sleep(time::Duration::from_millis(300));
        adb_Click(
            vv,
            &(result.min_value_location.0 + 732_u32 - 792).to_string(),
            &(result.min_value_location.1 + 572_u32 - 563).to_string(),
        );
        thread::sleep(time::Duration::from_millis(500));

        let mut flag = false;
        let mut flag_trigger = 0;
        let mut discard_x = result.min_value_location.0+120;
        let mut discard_y = result.min_value_location.1;
        let mut rng_object = thread_rng();
        if discard_x>1200 {
            discard_x =100
        }

        for (k, v) in self.comment.rows.iter().enumerate() {
            let (x, y) = (k / 8 + 1, k % 8 + 1);
            let (mut click_x, mut click_y) = (0, 0);

            (click_x, click_y) = (
                result.min_value_location.0 - v.0,
                result.min_value_location.1 - v.1,
            );
            println!(
                "第{x}行第{y}个{},{},{},{}",
                click_x, click_y, self.comment.goods.0, self.comment.goods.1
            );
            adb_Click(vv, &click_x.to_string(), &click_y.to_string());
            if flag_trigger > 3 {
                println!("提前退出背包扫描已经有3次扫描不到");
                break;
            }
            let mut i = 0;
            while i < 6 {
                thread::sleep(time::Duration::from_millis(1000));
                let data_image = DynamicImage::ImageRgba8(
                    screencap_device(&vv)
                        .view(click_x, click_y, self.comment.goods.0, self.comment.goods.1)
                        .to_image(),
                );

                if let Ok(ocr_result) = self.paddle_ocr.ocr(&data_image) {
                    if ocr_result.len() >= 2 {
                        flag = false;
                        println!("<<<<<<<<<<<<<<<<<ocr 结果{:?}", ocr_result);
                        if ocr_result.len() >=1 {
                            if  self.config.goods_name == ocr_result[0] {
                                println!("===============找到物品 结果{:?} 开始丢弃", ocr_result);
                                adb_move(vv, &click_x.to_string(), &click_y.to_string(),
                                         &discard_x.to_string(),
                                         &discard_y.to_string(),
                                         &rng_object.gen_range(300..=2000).to_string(),
                                );

                                thread::sleep(time::Duration::from_millis(3500));
                                adb_input(vv);
                                let mut i2 = 0;
                                let mut flage2 = false;
                                while i2 < 12 {
                                    thread::sleep(time::Duration::from_millis(1800));
                                    let  temp = match_template(&screencap_device(&vv).to_luma32f(), &self.comment.confirm_image, &mut self.matcher);
                                    if temp.min_value<=13_f32 && temp.min_value_location.0 >749 {
                                        adb_Click(vv, &temp.min_value_location.0.to_string(), &(temp.min_value_location.1+5).to_string());
                                        println!("点击确认丢弃");
                                        thread::sleep(time::Duration::from_millis(1800));
                                    }
                                    let  temp = match_template(&screencap_device(&vv).to_luma32f(), &self.comment.confirm_image2, &mut self.matcher);
                                    if  temp.min_value<=13_f32 && temp.min_value_location.0  <749{
                                        adb_Click(vv, &temp.min_value_location.0.to_string(), &(temp.min_value_location.1+5).to_string());
                                        println!("点击确认丢弃2");
                                        flage2 = true;
                                    }
                                    i2+=1;
                                }
                                if flage2 {
                                    println!("ooooooo丢弃成功");

                                }else{
                                    println!("？？？？？丢弃失败");
                                }
                            }

                        }

                        break;
                    }
                }
                i += 1;
                if i == 6 {
                    flag = true;
                    println!(" ocr 识别不了");
                    //上次如果也未能找到就提前退出
                } else if flag && i == 4 {
                    flag_trigger += 1;
                    println!(" ocr 识别不了2");
                    break;
                }
            }

        }
    }

    fn x(&mut self, v: &str) {
        println!("游戏关闭x窗口");
        let mut screencap = screencap_device(v).to_luma32f();
        let result = match_template(&screencap, &self.comment.x_image, &mut self.matcher);
        if result.min_value < 13_f32 {
            adb_Click(
                v,
                &result.min_value_location.0.to_string(),
                &result.min_value_location.1.to_string(),
            );
            println!("找到游戏x窗口{:?},点击关闭", result.min_value_location);
            while match_template(&screencap, &self.comment.x_image, &mut self.matcher).min_value
                > 13_f32
            {
                thread::sleep(time::Duration::from_millis(500));
                screencap = screencap_device(v).to_luma32f();
            }
        } else {
            println!("未找到游戏内x窗口");
        }
    }
}
fn main() -> Result<(), ()> {
    //公共使用
    let comment_arc = Arc::new(Comment {
        map_image: image::open("./extra/map.bmp").unwrap().to_luma32f(),
        map_map_image: image::open("./extra/map_map2.bmp").unwrap().to_luma32f(),
        confirm_image: image::open("./extra/confirm.bmp").unwrap().to_luma32f(),
        confirm_image2: image::open("./extra/confirm2.bmp").unwrap().to_luma32f(),
        backpack_image: image::open("./extra/backpack.bmp").unwrap().to_luma32f(),
        x_image: image::open("./extra/x.bmp").unwrap().to_luma32f(),
        goods: (413 - 256_u32, 226 - 145_u32), //商品偏差 //商品偏差
        // 792, 563
        // 5*8
        rows: vec![
            (792 - 279, 563 - 192),
            (792 - 279 - 73, 563 - 192),
            (792 - 279 - 73 * 2, 563 - 192),
            (792 - 279 - 73 * 3, 563 - 192),
            (792 - 279 - 73 * 4, 563 - 192),
            (792 - 279 - 73 * 5, 563 - 192),
            (792 - 279 - 73 * 6, 563 - 192),
            (792 - 756, 563 - 192),
            //第二行
            (792 - 279, 563 - 272),
            (792 - 279 - 73, 563 - 272),
            (792 - 279 - 73 * 2, 563 - 272),
            (792 - 279 - 73 * 3, 563 - 272),
            (792 - 279 - 73 * 4, 563 - 272),
            (792 - 279 - 73 * 5, 563 - 272),
            (792 - 279 - 73 * 6, 563 - 272),
            (792 - 756, 563 - 272),
            //第3行
            (792 - 279, 563 - 345),
            (792 - 279 - 73, 563 - 345),
            (792 - 279 - 73 * 2, 563 - 345),
            (792 - 279 - 73 * 3, 563 - 345),
            (792 - 279 - 73 * 4, 563 - 345),
            (792 - 279 - 73 * 5, 563 - 345),
            (792 - 279 - 73 * 6, 563 - 345),
            (792 - 756, 563 - 345),
            //第4行
            (792 - 279, 563 - 419),
            (792 - 279 - 73, 563 - 419),
            (792 - 279 - 73 * 2, 563 - 419),
            (792 - 279 - 73 * 3, 563 - 419),
            (792 - 279 - 73 * 4, 563 - 419),
            (792 - 279 - 73 * 5, 563 - 419),
            (792 - 279 - 73 * 6, 563 - 419),
            (792 - 756, 563 - 419),
            //第5行
            (792 - 279, 563 - 485),
            (792 - 279 - 73, 563 - 485),
            (792 - 279 - 73 * 2, 563 - 485),
            (792 - 279 - 73 * 3, 563 - 485),
            (792 - 279 - 73 * 4, 563 - 485),
            (792 - 279 - 73 * 5, 563 - 485),
            (792 - 279 - 73 * 6, 563 - 485),
            (792 - 756, 563 - 485),
        ],
    });

    let config = Config::builder()
        .add_source(File::with_name("./config.toml"))
        .build()
        .unwrap();

    let config_object: AppConfig = config.try_deserialize().unwrap();

    println!("读到的配置 {:#?}", config_object);

    let bit = Command::new("cmd")
        .args(&["/C", "adb.exe", "devices"])
        .output()
        .expect("Failed to execute command");

    let device_message = String::from_utf8(bit.stdout).unwrap();

    let mut devices_arr = device_message
        .split("\r\n")
        .filter(|s| !s.is_empty() && s.find("\t").is_some())
        .map(|x| String::from(x.split("\t").next().unwrap()))
        .collect::<Vec<String>>();

    println!("查找到主机{:?}", devices_arr);
    let mut handles = vec![];
    if config_object.ip_addr.len() != 0 {
        devices_arr = config_object.ip_addr.clone();
        println!("主机补正威{:?}", devices_arr);
    }
    let config_object_arc = Arc::new(config_object);
    for ip in devices_arr.into_iter() {
        let data = Arc::clone(&comment_arc);
        let config_object_arc = Arc::clone(&config_object_arc);

        let handle = thread::spawn(move || {
            let mut object = App {

                matcher: TemplateMatcher::new(),
                comment: data,
                config: config_object_arc,
                paddle_ocr: ocr::paddle::PaddleOcr::new(),
            };
            println!("当前执行的是{:?}", &ip);

            object.map_position(&ip);
            if let Ok(result) = object.backpack_open(ip.as_str()) {
                object.backpack_goods_discard(ip.as_str(), result);
            } else {
                println!("{:?} 背包打开失败", ip);
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().expect("<UNK>");
    }

    // adb_Click(&devices_arr[0],"1245","255");
    // adb_input(&devices_arr[0]);
    // adb_move(
    //     &devices_arr[0],
    //     "198",
    //     "187",
    //     "298",
    //     "287",
    //     &rng.gen_range(300..=2000).to_string(),
    // );
    //1280*720 dpi 240
    Ok(())
}
fn screencap_device(device_ip: &str) -> DynamicImage {
    let bit = Command::new("cmd")
        .args(&[
            "/C",
            "adb.exe",
            "-s",
            device_ip,
            "exec-out",
            "screencap",
            "-p",
        ])
        .output()
        .expect("Failed to execute command");

    image::load(Cursor::new(bit.stdout), image::ImageFormat::Png).unwrap()
}
fn adb_Click(device_ip: &str, x: &str, y: &str) {
    let _ = Command::new("cmd")
        .args(&[
            "/C", "adb.exe", "-s", device_ip, "shell", "input", "tap", x, y,
        ])
        .output()
        .expect("Failed to execute command");
}
fn adb_move(device_ip: &str, x: &str, y: &str, to_x: &str, to_y: &str, ms: &str) {
    let _ = Command::new("cmd")
        .args(&[
            "/C", "adb.exe", "-s", device_ip, "shell", "input", "swipe", x, y, to_x, to_y, ms,
        ])
        .output()
        .expect("Failed to execute command");
}

fn adb_input(device_ip: &str) {
    let _ = Command::new("cmd")
        .args(&[
            "/C",
            "adb.exe",
            "-s",
            device_ip,
            "shell",
            "input",
            "text",
            "999999999",
        ])
        .output()
        .expect("Failed to execute command");
    thread::sleep(time::Duration::from_millis(500));
    // let _ = Command::new("cmd")
    //     .args(&[
    //         "/C",
    //         "adb.exe",
    //         "-s",
    //         device_ip,
    //         "shell",
    //         "input",
    //         "keyevent",
    //         "KEYCODE_ENTER",
    //     ])
    //     .output()
    //     .expect("Failed to execute command");
}

fn match_template(
    input_image: &ImageBuffer<Luma<f32>, Vec<f32>>,
    template_image: &ImageBuffer<Luma<f32>, Vec<f32>>,
    matcher: &mut TemplateMatcher,
) -> template_matching::Extremes {
    matcher.match_template(
        input_image,
        template_image,
        MatchTemplateMethod::SumOfSquaredDifferences,
    );
    let result = matcher.wait_for_result().unwrap();

    let extremes = find_extremes(&result);
    extremes
}
