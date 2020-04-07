use old_gods::prelude::*;
use web_sys::CanvasRenderingContext2d;

use super::HtmlResources;

fn button_color(btn: &ActionButton) -> Color {
  match btn {
    ActionButton::A => Color::rgb(50, 229, 56),
    ActionButton::B => Color::rgb(202, 16, 16),
    ActionButton::X => Color::rgb(16, 124, 202),
    ActionButton::Y => Color::rgb(197, 164, 23),
  }
}


fn button_text(btn: &ActionButton) -> String {
  match btn {
    ActionButton::A => "A",
    ActionButton::B => "B",
    ActionButton::X => "X",
    ActionButton::Y => "Y",
  }
  .to_string()
}

/// Draw an action button at a point with an optional message to the right.
pub fn draw_button(
  context: &mut CanvasRenderingContext2d,
  _resources: &mut HtmlResources,
  btn: ActionButton,
  point: &V2,
  msg: &Option<String>,
) -> AABB {
  let mut btn_text = super::fancy_text(&button_text(&btn).as_str());
  btn_text.color = button_color(&btn);
  super::draw_text(&btn_text, point, context);

  let dest_size = super::measure_text(&btn_text, context);
  let btn_rect = AABB {
    top_left: *point,
    extents: V2::new(dest_size.0, dest_size.1),
  };
  let text_rect = if let Some(text) = msg {
    let point = V2::new(point.x + dest_size.0, point.y);
    let text = super::normal_text(&text.as_str());
    super::draw_text(&text, &point, context);
    let text_size = super::measure_text(&text, context);
    AABB {
      top_left: point,
      extents: V2::new(text_size.0, text_size.1),
    }
  } else {
    btn_rect
  };
  AABB::union(&btn_rect, &text_rect)
}


/// Draw an Action.
pub fn draw(
  canvas: &mut CanvasRenderingContext2d,
  resources: &mut HtmlResources,
  point: &V2,
  action: &Action,
) {
  let msg: Option<String> = action.text.non_empty().map(|s| s.clone());
  draw_button(
    canvas,
    resources,
    ActionButton::A,
    &(*point - V2::new(7.0, 7.0)),
    &msg,
  );
}
