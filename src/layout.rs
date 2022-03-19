#[derive(Debug, Copy, Clone)]
pub struct Layout {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
pub trait Layoutable {
    fn layout_in(self, layout: &mut Layout, margin: i32) -> Self;
    fn layout_top(self, layout: &mut Layout, size: i32) -> Self;
    fn layout_right(self, layout: &mut Layout, size: i32) -> Self;
}
impl<T> Layoutable for T
where
    T: fltk::prelude::WidgetExt,
    T: Sized,
{
    fn layout_in(mut self, layout: &mut Layout, margin: i32) -> T {
        layout.x += margin;
        layout.y += margin;
        layout.width -= 2 * margin;
        layout.height -= 2 * margin;
        self.set_pos(layout.x, layout.y);
        self.set_size(layout.width, layout.height);
        self
    }
    fn layout_top(mut self, layout: &mut Layout, gap: i32) -> T {
        self.set_pos(layout.x, layout.y);
        self.set_size(layout.width, gap);
        layout.y += gap;
        layout.height -= gap;
        self
    }
    fn layout_right(mut self, layout: &mut Layout, gap: i32) -> T {
        self.set_pos(layout.x, layout.y);
        self.set_size(gap, layout.height);
        layout.x += gap;
        layout.width -= gap;
        self
    }
}
impl Layout {
    pub fn new(width: i32, height: i32) -> Layout {
        Layout {
            x: 0,
            y: 0,
            width,
            height,
        }
    }
}
