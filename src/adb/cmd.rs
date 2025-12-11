use image::DynamicImage;
use std::io::Cursor;
use std::process::Command;

pub struct AdbCmd {
    pub device_ip: String,
}

impl AdbCmd {
    pub fn screencap_device(&self) -> DynamicImage {
        let bit = Command::new("cmd")
            .args(&[
                "/C",
                "adb.exe",
                "-s",
                self.device_ip.as_str(),
                "exec-out",
                "screencap",
                "-p",
            ])
            .output()
            .expect("Failed to execute command");

        image::load(Cursor::new(bit.stdout), image::ImageFormat::Png).unwrap()
    }

    pub fn adb_click(&self, x: &str, y: &str) {
        let _ = Command::new("cmd")
            .args(&[
                "/C",
                "adb.exe",
                "-s",
                self.device_ip.as_str(),
                "shell",
                "input",
                "tap",
                x,
                y,
            ])
            .output()
            .expect("Failed to execute command");
    }

    pub fn adb_move(&self, x: &str, y: &str, to_x: &str, to_y: &str, ms: &str) {
        let _ = Command::new("cmd")
            .args(&[
                "/C",
                "adb.exe",
                "-s",
                self.device_ip.as_str(),
                "shell",
                "input",
                "swipe",
                x,
                y,
                to_x,
                to_y,
                ms,
            ])
            .output()
            .expect("Failed to execute command");
    }
}
