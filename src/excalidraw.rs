use crate::state::{GraphData};
use serde_json::{json, Value};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn seed_from<T: Hash>(t: &T) -> u32 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    (s.finish() & 0xFFFF_FFFF) as u32
}

fn builtin_emoji(name: &str) -> &str {
    match name.to_lowercase().as_str() {
        "salesperson" | "sales" => "🧑‍💼",
        "email" => "📨",
        "database" | "db" => "🗄️",
        "model" => "🧠",
        "search" => "🔎",
        _ => "📌",
    }
}

pub fn graphdata_to_excalidraw_scene_with_opts(g: &GraphData, allow_images: bool, _assets_dir: &str) -> Value {
    let mut scene = graphdata_to_excalidraw_scene(g);
    if !allow_images { return scene; }

    // append decorations as text icons for now
    let decorations_opt = g.decorations.as_ref();
    if decorations_opt.is_none() { return scene; }

    let mut extra_elements: Vec<Value> = Vec::new();
    for d in decorations_opt.unwrap() {
        let mut cx = d.at_x.unwrap_or(0.0) as f64;
        let mut cy = d.at_y.unwrap_or(0.0) as f64;
        if let Some(tid) = &d.target {
            if let Some(n) = g.nodes.iter().find(|n| &n.id == tid) {
                cx = n.x as f64; cy = n.y as f64;
            }
        }
        if let Some(off) = &d.offset { cx += off.dx as f64; cy += off.dy as f64; }
        let label = if let Some(b) = &d.builtin { builtin_emoji(b).to_string() } else { d.text.clone().unwrap_or_else(|| "".to_string()) };
        if label.is_empty() { continue; }
        let size = d.size.as_ref().map(|s| (s.w as f64, s.h as f64)).unwrap_or((20.0, 20.0));
        let seed = seed_from(&(label.clone(), cx as i64, cy as i64));
        let el = json!({
            "type": "text",
            "version": 1,
            "versionNonce": (seed as i64),
            "isDeleted": false,
            "id": format!("decor-{}-{}", label, seed),
            "seed": seed,
            "fillStyle": "solid",
            "strokeWidth": 1,
            "strokeStyle": "solid",
            "roughness": 0,
            "opacity": 100,
            "angle": 0,
            "x": cx - size.0/2.0,
            "y": cy - size.1/2.0,
            "strokeColor": "#111827",
            "backgroundColor": "transparent",
            "width": size.0,
            "height": size.1,
            "boundElements": [],
            "updated": 0,
            "text": label,
            "fontSize": 16,
            "fontFamily": 1,
            "textAlign": "center",
            "verticalAlign": "middle",
            "baseline": 16
        });
        extra_elements.push(el);
    }

    if let Some(arr) = scene.get_mut("elements").and_then(|v| v.as_array_mut()) {
        arr.extend(extra_elements);
    }
    scene
}

fn node_size(label: &str) -> (f64, f64) {
    // Wider boxes for longer labels to reduce overlap
    let w = (label.len() as f64 * 10.0 + 30.0).max(100.0);
    let h = 48.0;
    (w, h)
}

fn color_or(default_hex: &str, c: &str) -> String {
    if c.trim().is_empty() { default_hex.to_string() } else { c.to_string() }
}

pub fn graphdata_to_excalidraw_scene(g: &GraphData) -> Value {
    // Build in layers: arrows (bottom), rectangles (middle), labels (top)
    let mut arrows: Vec<Value> = Vec::new();
    let mut rects: Vec<Value> = Vec::new();
    let mut texts: Vec<Value> = Vec::new();

    for n in &g.nodes {
        let seed = seed_from(&n.id);
        let (w, h) = node_size(&n.label);
        let stroke = "#111827";
        let bg = color_or("#F3F4F6", &n.style.color);
        let rect = json!({
            "type": "rectangle",
            "version": 1,
            "versionNonce": (seed as i64),
            "isDeleted": false,
            "id": format!("node-{}", n.id),
            "seed": seed,
            "fillStyle": "solid",
            "strokeWidth": 2,
            "strokeStyle": "solid",
            "roughness": 1,
            "opacity": 100,
            "angle": 0,
            "x": n.x as f64 - w/2.0,
            "y": n.y as f64 - h/2.0,
            "strokeColor": stroke,
            "backgroundColor": bg,
            "width": w,
            "height": h,
            "boundElements": [],
            "updated": 0,
            "roundness": {"type": 3}
        });
        rects.push(rect);

        let text_seed = seed_from(&(n.id.clone(), "text"));
        // Center text: position text box so its center aligns with node center
        let text_w = (n.label.len() as f64 * 9.0).min(w - 16.0).max(24.0);
        let text_h = 24.0;
        let text_x = n.x as f64 - text_w/2.0;
        let text_y = n.y as f64 - text_h/2.0;
        let text = json!({
            "type": "text",
            "version": 1,
            "versionNonce": (text_seed as i64),
            "isDeleted": false,
            "id": format!("node-label-{}", n.id),
            "seed": text_seed,
            "fillStyle": "solid",
            "strokeWidth": 1,
            "strokeStyle": "solid",
            "roughness": 0,
            "opacity": 100,
            "angle": 0,
            "x": text_x,
            "y": text_y,
            "strokeColor": stroke,
            "backgroundColor": "transparent",
            "width": text_w,
            "height": text_h,
            "boundElements": [],
            "updated": 0,
            "text": n.label,
            "fontSize": 16,
            "fontFamily": 1,
            "textAlign": "center",
            "verticalAlign": "middle",
            "baseline": 18
        });
        texts.push(text);
    }

    for e in &g.edges {
        let seed = seed_from(&e.id);
        let src = g.nodes.iter().find(|n| n.id == e.source);
        let tgt = g.nodes.iter().find(|n| n.id == e.target);
        if let (Some(s), Some(t)) = (src, tgt) {
            let (sw, sh) = node_size(&s.label);
            let (tw, th) = node_size(&t.label);
            let sx = s.x as f64;
            let sy = s.y as f64;
            let tx = t.x as f64;
            let ty = t.y as f64;

            // Compute intersection points with source and target rectangles to avoid overlapping nodes
            let (start_x, start_y) = line_rect_border_intersection(sx, sy, sw, sh, tx, ty);
            let (end_x, end_y) = line_rect_border_intersection(tx, ty, tw, th, sx, sy);

            let dx = end_x - start_x;
            let dy = end_y - start_y;
            // Orthogonal routing: L-shape via (dx, 0) then (dx, dy)
            let arrow_seed = seed_from(&(e.id.clone(), "arrow"));
            let points = vec![vec![0.0, 0.0], vec![dx, 0.0], vec![dx, dy]];
            let arrow = json!({
                "type": "arrow",
                "version": 1,
                "versionNonce": (arrow_seed as i64),
                "isDeleted": false,
                "id": format!("edge-{}", e.id),
                "seed": arrow_seed,
                "fillStyle": "solid",
                "strokeWidth": 2,
                "strokeStyle": "solid",
                "roughness": 1,
                "opacity": 100,
                "angle": 0,
                "x": start_x,
                "y": start_y,
                "strokeColor": "#111827",
                "backgroundColor": "transparent",
                "width": dx.abs(),
                "height": dy.abs(),
                "boundElements": [],
                "updated": 0,
                "lastCommittedPoint": Value::Null,
                "points": points,
                "startArrowhead": Value::Null,
                "endArrowhead": "arrow"
            });
            arrows.push(arrow);

            if !e.label.is_empty() {
                // Segment-aware label: choose longest orthogonal segment
                let (seg_mid_x, seg_mid_y, nx, ny) = if dx.abs() >= dy.abs() {
                    // horizontal segment midpoint at (start_x + dx/2, start_y)
                    (start_x + dx/2.0, start_y, 0.0, 1.0)
                } else {
                    // vertical segment midpoint at (end_x, start_y + dy/2)
                    (end_x, start_y + dy/2.0, -1.0, 0.0)
                };
                let off = 12.0;
                let lx = seg_mid_x + nx * off;
                let ly = seg_mid_y + ny * off;
                let lw = (e.label.len() as f64 * 9.0 + 8.0).max(24.0);
                let lh = 20.0;
                let lseed = seed_from(&(e.id.clone(), "label"));
                let label = json!({
                    "type": "text",
                    "version": 1,
                    "versionNonce": (lseed as i64),
                    "isDeleted": false,
                    "id": format!("edge-label-{}", e.id),
                    "seed": lseed,
                    "fillStyle": "solid",
                    "strokeWidth": 1,
                    "strokeStyle": "solid",
                    "roughness": 0,
                    "opacity": 100,
                    "angle": 0,
                    "x": lx - lw/2.0,
                    "y": ly - lh/2.0,
                    "strokeColor": "#111827",
                    "backgroundColor": "transparent",
                    "width": lw,
                    "height": lh,
                    "boundElements": [],
                    "updated": 0,
                    "text": e.label,
                    "fontSize": 14,
                    "fontFamily": 1,
                    "textAlign": "center",
                    "verticalAlign": "middle",
                    "baseline": 16
                });
                texts.push(label);
            }
        }
    }

    // Compose layers: order matters (arrows under nodes, labels on top)
    let mut elements: Vec<Value> = Vec::new();
    // Containers behind everything
    if let Some(conts) = g.containers.as_ref() {
        let padding = 40.0;
        for c in conts {
            // compute bbox from children
            let mut minx = f64::INFINITY; let mut miny = f64::INFINITY; let mut maxx = f64::NEG_INFINITY; let mut maxy = f64::NEG_INFINITY;
            for id in &c.children {
                if let Some(n) = g.nodes.iter().find(|n| &n.id == id) {
                    let (w,h) = node_size(&n.label);
                    minx = minx.min(n.x as f64 - w/2.0);
                    miny = miny.min(n.y as f64 - h/2.0);
                    maxx = maxx.max(n.x as f64 + w/2.0);
                    maxy = maxy.max(n.y as f64 + h/2.0);
                }
            }
            if !minx.is_finite() { continue; }
            let x = minx - padding; let y = miny - padding; let w = (maxx-minx) + 2.0*padding; let h = (maxy-miny) + 2.0*padding;
            let seed = seed_from(&(c.id.clone(), "container"));
            let bg = c.style.as_ref().and_then(|s| s.bg.clone()).unwrap_or("#FFFFFF".to_string());
            let rect = json!({
                "type": "rectangle",
                "version": 1,
                "versionNonce": (seed as i64),
                "isDeleted": false,
                "id": format!("container-{}", c.id),
                "seed": seed,
                "fillStyle": "solid",
                "strokeWidth": 2,
                "strokeStyle": "solid",
                "roughness": 0,
                "opacity": 100,
                "angle": 0,
                "x": x,
                "y": y,
                "strokeColor": "#D1D5DB",
                "backgroundColor": bg,
                "width": w,
                "height": h,
                "boundElements": [],
                "updated": 0,
                "roundness": {"type": 3}
            });
            elements.push(rect);
            // header chip/tag
            let tag = c.style.as_ref().and_then(|s| s.label_tag.clone()).unwrap_or(c.label.clone());
            let tag_w = (tag.len() as f64 * 7.5 + 24.0).max(48.0);
            let tag_h = 20.0;
            let tseed = seed_from(&(c.id.clone(), "container-tag"));
            let label = json!({
                "type": "text",
                "version": 1,
                "versionNonce": (tseed as i64),
                "isDeleted": false,
                "id": format!("container-tag-{}", c.id),
                "seed": tseed,
                "fillStyle": "solid",
                "strokeWidth": 1,
                "strokeStyle": "solid",
                "roughness": 0,
                "opacity": 100,
                "angle": 0,
                "x": x + 12.0,
                "y": y + 8.0,
                "strokeColor": "#6B7280",
                "backgroundColor": "transparent",
                "width": tag_w,
                "height": tag_h,
                "boundElements": [],
                "updated": 0,
                "text": tag,
                "fontSize": 14,
                "fontFamily": 1,
                "textAlign": "left",
                "verticalAlign": "top",
                "baseline": 16
            });
            elements.push(label);
        }
    }

    // Compose layers: arrows first, then rectangles, then texts on top
    elements.extend(arrows);
    elements.extend(rects);
    elements.extend(texts);

    // app background color from global_style or default
    let app_bg = g
        .global_style
        .as_ref()
        .map(|gs| gs.background.clone())
        .unwrap_or("#FFFFFF".to_string());

    json!({
        "type": "excalidraw",
        "version": 2,
        "source": "graphflow",
        "elements": elements,
        "appState": {
            "viewBackgroundColor": app_bg,
            "gridSize": 0
        },
        "files": {}
    })
}

// Compute intersection point between a rectangle border and a ray from (cx,cy) toward (tx,ty)
fn line_rect_border_intersection(cx: f64, cy: f64, w: f64, h: f64, tx: f64, ty: f64) -> (f64, f64) {
    let dx = tx - cx;
    let dy = ty - cy;
    // Avoid division by zero
    let mut t_candidates: Vec<f64> = Vec::new();
    if dx != 0.0 { t_candidates.push((w / 2.0) / dx.abs()); }
    if dy != 0.0 { t_candidates.push((h / 2.0) / dy.abs()); }
    let t = if let Some(min_t) = t_candidates.into_iter().fold(None, |acc: Option<f64>, v| {
        Some(match acc { Some(a) => if v < a { v } else { a }, None => v })
    }) { min_t } else { 0.0 };
    // Slight inset to keep arrow off the stroke
    let inset = 2.0;
    let norm = (dx.hypot(dy)).max(1.0);
    let sx = cx + dx * t;
    let sy = cy + dy * t;
    // Pull back by inset along the direction
    let ux = dx / norm;
    let uy = dy / norm;
    (sx - ux * inset, sy - uy * inset)
}
