use ratatui::{
  prelude::*,
  widgets::{Block, WidgetRef},
};

#[derive(Debug, Clone)]
pub struct Scrollable<'a> {
  child_buffer: Buffer,
  block: Option<Block<'a>>,
  state: ScrollableState,
}

#[derive(Debug, Clone, Default)]
pub struct ScrollableState {
  x_offset: u16,
  y_offset: u16,
}

impl<'a> Scrollable<'a> {
  pub fn new(child_widget: Box<dyn WidgetRef>, max_height: u16, max_width: u16) -> Self {
    let mut buf = Buffer::empty(Rect::new(0, 0, max_width, max_height));
    child_widget.render_ref(buf.area, &mut buf);
    let child_buffer = clamp(buf);
    let state = ScrollableState::default();
    Self { child_buffer, state, block: None }
  }

  pub fn block(mut self, block: Block<'a>) -> Self {
    self.block = Some(block);
    self
  }

  pub fn log(&self) {
    let buf_height = self.child_buffer.area.height;
    let buf_width = self.child_buffer.area.width;
    for n in 0..buf_height {
      let mut line: String = String::from("");
      let cells =
        self.child_buffer.content.to_vec()[((n * buf_width) as usize)..(((n + 1) * buf_width) as usize)].to_vec();
      for cell in cells.iter() {
        line += cell.clone().symbol();
      }
      log::info!("{}", line.as_str());
    }
  }

  pub fn debug_log(&self) {
    let buf_height = self.child_buffer.area.height;
    let buf_width = self.child_buffer.area.width;
    for n in 0..buf_height {
      let mut line: String = String::from("");
      let cells =
        self.child_buffer.content.to_vec()[((n * buf_width) as usize)..(((n + 1) * buf_width) as usize)].to_vec();
      for cell in cells.iter() {
        line += cell.clone().symbol();
      }
      log::info!(
        "rendering line {}/{}, length {}, last symbol {}, last symbol is blank {}:",
        n,
        buf_height,
        line.len(),
        cells[cells.len() - 1].symbol(),
        cells[cells.len() - 1].symbol() == " "
      );

      log::info!("{}", line.as_str());
    }
  }
}

impl<'a> StatefulWidget for Scrollable<'a> {
  type State = ScrollableState;

  fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
    self.block.render_ref(area, buf);
    let render_area = self.block.inner_if_some(area);
    if render_area.is_empty() {
      return;
    }
    log::info!("render area: {}", render_area);
    let area = render_area.intersection(buf.area);
    log::info!("intersection area: {}", area);
    let max_x = Ord::min(area.x.saturating_add(area.width), buf.area.right());
    let max_y = Ord::min(area.y.saturating_add(area.height), buf.area.bottom());
    let content_height = self.child_buffer.area.height;
    let content_width = self.child_buffer.area.width;
    for i in area.y..max_y {
      let content_i = i - area.y;
      let row = self.child_buffer.content.to_vec()[(((content_i + state.y_offset) * content_width) as usize)
        ..(((content_i + state.y_offset + 1) * content_width) as usize)]
        .to_vec();
      for j in area.x..max_x {
        let content_j = j - area.x;
        let cell = &row[(content_j + state.x_offset) as usize];
        buf.get_mut(j, i).set_symbol(cell.symbol()).set_fg(cell.fg).set_bg(cell.bg).set_skip(cell.skip);
      }
    }
  }
}

fn clamp(buf: Buffer) -> Buffer {
  let height = buf.area.height;
  let width = buf.area.width;
  let mut used_height: u16 = 0;
  let mut used_width: u16 = 0;
  for i in (0..height).rev() {
    let row = buf.content.to_vec()[((i * width) as usize)..(((i + 1) * width) as usize)].to_vec();
    for j in (0..width).rev() {
      let cell = row[j as usize].clone();
      if cell.symbol() != " " {
        used_height = std::cmp::max(used_height, i + 1);
        used_width = std::cmp::max(used_width, j + 1);
      }
    }
  }
  let mut content: Vec<ratatui::buffer::Cell> = Vec::new();
  for i in 0..used_height {
    let row = buf.content.to_vec()[((i * width) as usize)..(((i + 1) * width) as usize)].to_vec();
    for j in 0..used_width {
      content.push(row[j as usize].clone().to_owned());
    }
  }
  Buffer { area: Rect::new(0, 0, used_width, used_height), content }
}
