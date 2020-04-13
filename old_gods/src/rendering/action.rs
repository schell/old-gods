use super::super::prelude::*;

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
pub fn draw_button<Ctx: HasRenderingContext> (
    context: &mut Ctx,
    btn: ActionButton,
    point: &V2,
    msg: &Option<String>,
) -> Result<AABB, String> {
    let mut btn_text = Ctx::fancy_text(&button_text(&btn).as_str());
    btn_text.color = button_color(&btn);
    context.draw_text(&btn_text, point)?;

    let dest_size = context.measure_text(&btn_text)?;
    let btn_rect = AABB {
        top_left: *point,
        extents: V2::new(dest_size.0, dest_size.1),
    };
    let text_rect = if let Some(text) = msg {
        let point = V2::new(point.x + dest_size.0, point.y);
        let text = Ctx::normal_text(&text.as_str());
        context.draw_text(&text, &point)?;
        let text_size = context.measure_text(&text)?;
        AABB {
            top_left: point,
            extents: V2::new(text_size.0, text_size.1),
        }
    } else {
        btn_rect
    };
    Ok(AABB::union(&btn_rect, &text_rect))
}


/// Draw an Action.
pub fn draw<Ctx: HasRenderingContext>(
    context: &mut Ctx,
    point: &V2,
    action: &Action,
) -> Result<(), String> {
    let msg: Option<String> = action.text.non_empty().map(|s| s.clone());
    draw_button::<Ctx>(
        context,
        ActionButton::A,
        &(*point - V2::new(7.0, 7.0)),
        &msg,
    )?;
    Ok(())
}
