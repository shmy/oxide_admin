use ab_glyph::{FontRef, PxScale};
use image::{ImageBuffer, Rgba};
use std::{io::Cursor, sync::LazyLock};

use super::{CaptchaData, CaptchaTrait};

static FONT: LazyLock<FontRef> = LazyLock::new(|| {
    FontRef::try_from_slice(include_bytes!("OpenSans-Bold.ttf")).expect("Failed to load font")
});

static SCALE: LazyLock<PxScale> = LazyLock::new(|| PxScale::from(26.0));

pub struct MathCaptcha {
    max_result: u16,
    width: u16,
    height: u16,
}

impl Default for MathCaptcha {
    fn default() -> Self {
        Self {
            max_result: 99,
            width: 140,
            height: 60,
        }
    }
}

impl CaptchaTrait for MathCaptcha {
    fn generate(&self) -> anyhow::Result<CaptchaData> {
        let (question, answer) = self.rand_math();
        let bytes = self.draw_text_png(&question)?;
        Ok(CaptchaData {
            bytes,
            value: answer.to_string(),
        })
    }
}

impl MathCaptcha {
    pub fn new(max_result: u16, width: u16, height: u16) -> Self {
        Self {
            max_result,
            width,
            height,
        }
    }

    fn gen_math(&self) -> (u16, char, u16) {
        let operators = ['+', '-', '×', '÷'];
        let b = rand::random_range(0..=3);
        let a = rand::random_range(0..=self.max_result);
        let b = operators[b];
        let c = rand::random_range(0..=self.max_result);
        (a, b, c)
    }

    fn check_math(&self, a: u16, b: char, c: u16) -> Option<u16> {
        match b {
            '+' => {
                if a + c <= self.max_result {
                    Some(a + c)
                } else {
                    None
                }
            }
            '-' => {
                if a >= c && a - c <= self.max_result {
                    Some(a - c)
                } else {
                    None
                }
            }
            '×' => {
                if a * c <= self.max_result {
                    Some(a * c)
                } else {
                    None
                }
            }
            '÷' => {
                if c != 0 && a.is_multiple_of(c) && a / c <= self.max_result {
                    Some(a / c)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn rand_math(&self) -> (String, u16) {
        let mut r = self.gen_math();
        loop {
            if let Some(v) = self.check_math(r.0, r.1, r.2) {
                return (format!("{} {} {} = ?", r.0, r.1, r.2), v);
            }
            r = self.gen_math();
        }
    }

    fn rand_color(&self) -> Rgba<u8> {
        let r: u8 = rand::random_range(0..255);
        let g: u8 = rand::random_range(0..255);
        let b; // 未定义，可省略mut
        let a: u8 = rand::random_range(0..255);
        let sum_rg = u16::from(r) + u16::from(g);
        if sum_rg > 400 {
            b = 0;
        } else {
            let tmp_b = 400 - u16::from(r) - u16::from(g);
            if tmp_b > 255 {
                b = 255;
            } else {
                b = tmp_b as u8;
            }
        }
        Rgba([r, g, b, a])
    }

    fn rand_light_color(&self) -> Rgba<u8> {
        let r = rand::random_range(200..=255);
        let g = rand::random_range(200..=255);
        let b = rand::random_range(200..=255);
        Rgba([r, g, b, 255])
    }

    fn rand_dark_color(&self) -> Rgba<u8> {
        let base = rand::random_range(0..=60); // 越小越黑
        let r = base + rand::random_range(0..=40);
        let g = base + rand::random_range(0..=40);
        let b = base + rand::random_range(0..=40);
        Rgba([r.min(100), g.min(100), b.min(100), 255])
    }

    fn draw_text_png(&self, text: &str) -> anyhow::Result<Vec<u8>> {
        let padding = 10.0; // 左右各保留10像素
        let available_width = self.width as f32 - padding * 2.0;
        let font_width = available_width / text.len() as f32;
        let background_color = self.rand_dark_color();
        let mut img =
            ImageBuffer::from_pixel(self.width as u32, self.height as u32, background_color);
        // 绘制干扰点
        for _ in 0..100 {
            let size = rand::random_range(1..=2);
            let x = rand::random_range(0..=(self.width - size)) as u32;
            let y = rand::random_range(0..=(self.height - size)) as u32;

            let color = if rand::random() {
                self.rand_dark_color()
            } else {
                self.rand_color()
            };

            for dx in 0..size {
                for dy in 0..size {
                    img.put_pixel(x + dx as u32, y + dy as u32, color);
                }
            }
        }
        // 绘制验证码
        let font_size = 26;
        let char_width = font_width.min(self.width as f32); // 防止太宽
        let char_height = font_size as u32;

        for (i, c) in text.chars().enumerate() {
            let mut x = (padding + font_width * i as f32 + font_width / 8.0) as i32;
            let mut y = (self.height / 2 - font_size / 2)
                .saturating_sub(rand::random_range(0..=(self.height / 5)))
                as i32;

            // 安全边界控制
            x = x.clamp(0, self.width as i32 - char_width as i32);
            y = y.clamp(0, self.height as i32 - char_height as i32);

            let color = self.rand_light_color();
            imageproc::drawing::draw_text_mut(
                &mut img,
                color,
                x,
                y,
                *SCALE,
                &*FONT,
                &c.to_string(),
            );
        }
        let dynamic_image: ::image::DynamicImage = img.into();
        let mut buf = Cursor::new(Vec::new());

        dynamic_image.write_to(&mut buf, ::image::ImageFormat::Png)?;
        Ok(buf.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_succeed() {
        let captcha = MathCaptcha::default();
        let result = captcha.generate().unwrap();
        assert!(result.value.parse::<u16>().is_ok());
        assert!(result.bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    }
}
