//! Render diagnostics (D1): overflow detection from frame inspection.

use typst::layout::{Frame, FrameItem};

/// The greatest y-extent reached by any item in the frame, in points,
/// recursing through groups.
fn max_y_extent(frame: &Frame, offset_y: f64) -> f64 {
    let mut max = offset_y;
    for (position, item) in frame.items() {
        let y = offset_y + position.y.to_pt();
        let item_bottom = match item {
            FrameItem::Group(group) => max_y_extent(&group.frame, y),
            FrameItem::Text(run) => y + run.size.to_pt(),
            FrameItem::Shape(shape, _) => y + shape_size_y(shape),
            FrameItem::Image(_, size, _) => y + size.y.to_pt(),
            _ => y,
        };
        if item_bottom > max {
            max = item_bottom;
        }
    }
    max
}

/// The pages whose content extends beyond the page's bottom edge — content
/// a viewer will clip — as 1-based page numbers.
pub fn shape_size_y(shape: &typst::visualize::Shape) -> f64 {
    use typst::visualize::Geometry;
    match &shape.geometry {
        Geometry::Line(to) => to.y.to_pt(),
        Geometry::Rect(size) => size.y.to_pt(),
        Geometry::Curve(_) => 0.0,
    }
}

pub fn overflow_pages(pages: &[typst_layout::Page]) -> Vec<usize> {
    pages
        .iter()
        .enumerate()
        .filter(|(index, page)| {
            let extent = max_y_extent(&page.frame, 0.0);
            let _ = index;
            extent > page.frame.size().y.to_pt() + 0.5
        })
        .map(|(index, _)| index + 1)
        .collect()
}
