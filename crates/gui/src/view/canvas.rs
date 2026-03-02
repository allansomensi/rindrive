use crate::message::Message;
use crate::view::components::badge::{
    COLOR_INVALID, COLOR_PENDING, COLOR_READING, COLOR_VALID, COLOR_WRITING,
};
use iced::widget::canvas::Cache;
use iced::widget::{canvas, container};
use iced::{Element, Length, Point, Rectangle, Size, Theme, mouse};

pub struct BlockMapProgram<'a> {
    pub blocks: &'a [u8],
    pub cache: &'a Cache,
}

impl<'a> canvas::Program<Message> for BlockMapProgram<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let total = self.blocks.len();

            if total == 0 {
                return;
            }

            let screen_ratio = bounds.width / bounds.height;
            let rows = (total as f32 / screen_ratio).sqrt().ceil();
            let mut cols = (total as f32 / rows).ceil();

            while (rows * cols) < total as f32 {
                cols += 1.0;
            }

            let w_space = bounds.width / cols;
            let h_space = bounds.height / rows;
            let box_size = w_space.min(h_space);

            let spacing = if box_size < 3.0 { 0.0 } else { 1.0 };
            let draw_size = (box_size - spacing).max(1.0);

            let grid_width = cols * box_size;
            let grid_height = rows * box_size;
            let start_x = (bounds.width - grid_width) / 2.0;
            let start_y = (bounds.height - grid_height) / 2.0;

            for (i, &status) in self.blocks.iter().enumerate() {
                let col = i as f32 % cols;
                let row = (i as f32 / cols).floor();

                let x = start_x + (col * box_size);
                let y = start_y + (row * box_size);

                if y > bounds.height {
                    break;
                }

                let color = match status {
                    1 => COLOR_VALID,
                    2 => COLOR_INVALID,
                    3 => COLOR_READING,
                    4 => COLOR_WRITING,
                    _ => COLOR_PENDING,
                };

                frame.fill_rectangle(Point::new(x, y), Size::new(draw_size, draw_size), color);
            }
        });

        vec![geometry]
    }
}

pub fn view<'a>(blocks: &'a [u8], cache: &'a Cache, border_radius: f32) -> Element<'a, Message> {
    container(
        canvas(BlockMapProgram { blocks, cache })
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(10)
    .style(move |theme: &Theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(palette.background.weak.color.into()),
            border: iced::Border {
                color: palette.background.strong.color,
                width: 1.0,
                radius: border_radius.into(),
            },
            ..Default::default()
        }
    })
    .into()
}
