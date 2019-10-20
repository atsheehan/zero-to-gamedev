use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

pub struct Renderer {
    canvas: WindowCanvas,
}

impl Renderer {
    pub fn new(canvas: WindowCanvas) -> Self {
        Self { canvas }
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(Color::RGB(75, 75, 75));
        self.canvas.clear();
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.canvas.set_draw_color(color);
        self.canvas.fill_rect(rect).unwrap();
    }
}
