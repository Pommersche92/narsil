use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};

/// A gauge that inverts the label colour character-by-character at the fill
/// boundary so the percentage text is always readable regardless of fill level.
pub struct SplitGauge<'a> {
    block: Option<Block<'a>>,
    ratio: f64,
    bar_color: Color,
    label: String,
}

impl<'a> SplitGauge<'a> {
    pub fn new(ratio: f64, bar_color: Color, label: impl Into<String>) -> Self {
        Self {
            block: None,
            ratio: ratio.clamp(0.0, 1.0),
            bar_color,
            label: label.into(),
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for SplitGauge<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = match self.block {
            Some(b) => {
                let inner = b.inner(area);
                b.render(area, buf);
                inner
            }
            None => area,
        };

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let width = inner.width as usize;
        let fill = ((self.ratio * width as f64).round() as usize).min(width);

        let label_chars: Vec<char> = self.label.chars().collect();
        let label_len = label_chars.len();
        let label_start = width.saturating_sub(label_len) / 2;
        let mut chars = vec![' '; width];
        for (i, &c) in label_chars.iter().enumerate() {
            let pos = label_start + i;
            if pos < width {
                chars[pos] = c;
            }
        }

        let y = inner.y + inner.height / 2;
        for col in 0..width {
            let x = inner.x + col as u16;
            let style = if col < fill {
                Style::default().fg(Color::Black).bg(self.bar_color)
            } else {
                Style::default().fg(self.bar_color)
            };
            let mut s = [0u8; 4];
            buf.set_string(x, y, chars[col].encode_utf8(&mut s), style);
        }
    }
}
