use std::error::Error;
use std::{thread, time};
use rand::Rng;
use crate::virtual_ddl::windows::{MouseAndKeyboardInstruct, UpOrDown};


pub struct DNFVirtual {
    key_mouse: MouseAndKeyboardInstruct,
}

impl DNFVirtual {
    pub fn new() -> Self {
        DNFVirtual {
            key_mouse: MouseAndKeyboardInstruct::new()
        }
    }
    pub fn on_mov(&self, x: f32, y: f32, to_x: f32, to_y: f32,movement_speed:f32) -> Result<(), Box<dyn Error>> {
        // ("up", 709), ("left", 710), ("down", 711), ("right", 712),
        let mut x_code = "right";
        let mut y_code = "down";
        let x_mov: u64 = (x - to_x).abs() as u64;
        let y_mov: u64 = (y - to_y).abs() as u64;
        if x - to_x > 0f32 {
            x_code = "left"
        }
        if y - to_y > 0f32 {
            y_code = "up"
        }
        println!("{:?}", (x, y, to_x, to_y));
        //快速移动
        self.key_mouse.on(x_code, UpOrDown::DownAndUp)?;
        self.key_mouse.on(x_code, UpOrDown::Up)?;
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(50..130)));
        self.key_mouse.on(y_code, UpOrDown::Down)?;
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(50..130)));

        //需要速度
        if x_mov > y_mov {
            let s = y_mov * 3; //需要计算实际速度
            if s > 0 {
                thread::sleep(time::Duration::from_millis(s));
            }
            self.key_mouse.on(y_code, UpOrDown::Up)?;
        } else {
            let s = x_mov * 2; //需要计算实际速度
            if s > 0 {
                thread::sleep(time::Duration::from_millis(s));
            }
            self.key_mouse.on(x_code, UpOrDown::Up)?;
        }

        if x_mov > y_mov {
            let s = (x_mov - y_mov) * 2; //需要计算实际速度
            if s > 0 {
                thread::sleep(time::Duration::from_millis(s));
            }

            self.key_mouse.on(x_code, UpOrDown::Up)?;
        } else {
            let s = (y_mov - x_mov) * 3; //需要计算实际速度
            if s > 0 {
                thread::sleep(time::Duration::from_millis(s));
            }
            self.key_mouse.on(y_code, UpOrDown::Up)?;
        }
        Ok(())
    }
    // 畅玩任务领取
    pub fn task_collection(&self) {
        println!("畅玩领取");
        let _ = self.on("f2");
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(1500..2500)));
        let _ = self.mouse_mov_click(356, 226);
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(500..2500)));
        let _ = self.mouse_mov_click(360, 290);
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(500..1500)));
        let _ = self.mouse_mov_click(361, 359);
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(500..1500)));
        let _ = self.mouse_mov_click(358, 439);
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(500..1500)));
        let _ = self.mouse_mov_click(359, 163);
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(500..1100)));
        let _ = self.mouse_mov_click(359, 160);
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(500..2500)));
        let _ = self.on("f2");
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(1500..2500)));
    }

    //点击跟换角色
    pub fn change_character(&self) {
        println!("更换角色");
        let _ = self.on("esc");
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(1500..2500)));
        let _ = self.mouse_mov_click(374, 489);
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(1000..2500)));
    }

    //选择角色
    pub fn select_character(&self) {
        println!("选择角色");
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(1000..2500)));
        let _ = self.key_mouse.on("left", UpOrDown::Down);
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(70..80)));
        let _ = self.key_mouse.on("left", UpOrDown::Up);
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(70..80)));
        let _ = self.key_mouse.on("space", UpOrDown::Down);
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(100..500)));
        let _ = self.key_mouse.on("space", UpOrDown::Up);
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(1000..3500)));
    }

    //快速左移动
    pub fn skip_left(&self) {
        let _ = self.key_mouse.on("left", UpOrDown::Down);
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(70..80)));
        let _ = self.key_mouse.on("left", UpOrDown::Up);
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(70..80)));
        let _ = self.key_mouse.on("left", UpOrDown::Down);
    }

    //快速右移动
    pub fn skip_right(&self) {
        let _ = self.key_mouse.on("right", UpOrDown::Down);
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(70..80)));
        let _ = self.key_mouse.on("right", UpOrDown::Up);
        thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(70..80)));
        let _ = self.key_mouse.on("right", UpOrDown::Down);
    }

    //-------------------------------- 基础方法 ------------------------------------------------

    //键盘按键
    pub fn on(&self, keys: &str) -> Result<(), Box<dyn Error>> {
        self.key_mouse.on(keys, UpOrDown::DownAndUp)?;
        Ok(())
    }

    //移动后点击 会进行偏移点击
    pub fn mouse_mov_click(&self, x: u32, y: u32) {
        self.key_mouse.mouse_mov_click(rand::thread_rng().gen_range(x..x + 6), rand::thread_rng().gen_range(y..y + 6));
    }
}