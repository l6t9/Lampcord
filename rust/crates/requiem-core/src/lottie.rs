use serde_json::Value;

pub const EMPTY_ANIMATION: &str =
    r#"{"v":"5.7.0","fr":60,"ip":0,"op":2,"w":24,"h":24,"layers":[]}"#;

pub fn asset_json(name: &str) -> &'static str {
    requiem_asset_store::get()
        .and_then(|store| store.lottie(name))
        .unwrap_or(EMPTY_ANIMATION)
}

pub fn fix_mask_gaps(json: &str) -> serde_json::Result<String> {
    let mut value: Value = serde_json::from_str(json)?;
    fix_mask_gaps_in_place(&mut value);
    serde_json::to_string(&value)
}

pub fn fix_leading_subtract_masks(json: &str) -> serde_json::Result<String> {
    let mut value: Value = serde_json::from_str(json)?;
    let mut lists: Vec<&mut Value> = Vec::new();
    let (layers, assets) = match &mut value {
        Value::Object(map) => {
            let mut layers = None;
            let mut assets = None;
            for (key, v) in map.iter_mut() {
                match key.as_str() {
                    "layers" => layers = Some(v),
                    "assets" => assets = Some(v),
                    _ => {}
                }
            }
            (layers, assets)
        }
        _ => (None, None),
    };
    lists.extend(layers);
    if let Some(Value::Array(assets)) = assets {
        lists.extend(assets.iter_mut().filter_map(|a| a.get_mut("layers")));
    }
    for list in lists {
        for layer in list.as_array_mut().into_iter().flatten() {
            let Some(masks) = layer
                .get_mut("masksProperties")
                .and_then(Value::as_array_mut)
            else {
                continue;
            };
            let leading_subtract = masks.first().is_some_and(|m| {
                m["mode"] == "s" && m.get("inv").and_then(Value::as_bool) != Some(true)
            });
            if leading_subtract {
                const BIG: f64 = 100_000.0;
                masks.insert(
                    0,
                    serde_json::json!({
                        "nm": "full", "mode": "a", "inv": false,
                        "pt": { "a": 0, "k": { "c": true, "i": [[0, 0], [0, 0], [0, 0], [0, 0]], "o": [[0, 0], [0, 0], [0, 0], [0, 0]],
                            "v": [[-BIG, -BIG], [BIG, -BIG], [BIG, BIG], [-BIG, BIG]] } },
                        "o": { "a": 0, "k": 100 }, "x": { "a": 0, "k": 0 },
                    }),
                );
            }
        }
    }
    serde_json::to_string(&value)
}

const MASK_GAP_THRESHOLD_FRAMES: f64 = 30.0;

const MASK_COLLAPSE_DURATION_FRAMES: f64 = 8.0;

fn fix_mask_gaps_in_place(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(Value::Array(masks)) = map.get_mut("masksProperties") {
                masks.retain(|m| m.get("mode").and_then(Value::as_str) == Some("s"));
                for mask in masks {
                    collapse_one_mask_gap(mask);
                }
            }
            for v in map.values_mut() {
                fix_mask_gaps_in_place(v);
            }
        }
        Value::Array(items) => {
            for v in items {
                fix_mask_gaps_in_place(v);
            }
        }
        _ => {}
    }
}

fn collapse_one_mask_gap(mask: &mut Value) {
    let Some(pt_keyframes) = mask
        .get("pt")
        .and_then(|pt| pt.get("k"))
        .and_then(Value::as_array)
        .cloned()
    else {
        return;
    };
    let times: Vec<f64> = pt_keyframes
        .iter()
        .filter_map(|kf| kf.get("t").and_then(Value::as_f64))
        .collect();
    if times.len() < 2 {
        return;
    }

    let mut largest_gap = (0.0_f64, 0usize, times[0], times[0]);
    for (idx, pair) in times.windows(2).enumerate() {
        let gap = pair[1] - pair[0];
        if gap > largest_gap.0 {
            largest_gap = (gap, idx, pair[0], pair[1]);
        }
    }
    let (gap_size, gap_start_idx, gap_start, gap_end) = largest_gap;
    if gap_size < MASK_GAP_THRESHOLD_FRAMES {
        return;
    }

    let Some(vertex_count) = pt_keyframes[gap_start_idx]
        .get("s")
        .and_then(Value::as_array)
        .and_then(|s| s.first())
        .and_then(|shape| shape.get("v"))
        .and_then(Value::as_array)
        .map(Vec::len)
    else {
        return;
    };

    let Some(first_point) = pt_keyframes[gap_start_idx]
        .get("s")
        .and_then(|s| s.get(0))
        .and_then(|shape| shape.get("v"))
        .and_then(|v| v.get(0))
        .cloned()
    else {
        return;
    };

    let v_points: Vec<Value> = (0..vertex_count).map(|_| first_point.clone()).collect();
    let zero_tangents: Vec<Value> = (0..vertex_count)
        .map(|_| serde_json::json!([0, 0]))
        .collect();
    let degenerate_shape = serde_json::json!({
        "i": zero_tangents, "o": zero_tangents, "v": v_points, "c": true,
    });

    let collapse_end = gap_start + MASK_COLLAPSE_DURATION_FRAMES;
    let expand_start = gap_end - MASK_COLLAPSE_DURATION_FRAMES;
    let ease = serde_json::json!({"x": [0.333], "y": [0]});
    let ease_in = serde_json::json!({"x": [0.667], "y": [1]});
    let new_keyframes = serde_json::json!([
        {"t": collapse_end, "s": [degenerate_shape], "i": ease, "o": ease_in},
        {"t": expand_start, "s": [degenerate_shape], "i": ease, "o": ease_in},
    ]);

    if let (Value::Object(map), Value::Array(new_kfs)) = (mask, new_keyframes) {
        if let Some(Value::Array(kfs)) = map.get_mut("pt").and_then(|pt| pt.get_mut("k")) {
            for (offset, kf) in new_kfs.into_iter().enumerate() {
                kfs.insert(gap_start_idx + 1 + offset, kf);
            }
        }
    }
}

pub fn recolor_fills_and_strokes(json: &str, r: f64, g: f64, b: f64) -> serde_json::Result<String> {
    let mut value: Value = serde_json::from_str(json)?;
    recolor_in_place(&mut value, r, g, b);
    serde_json::to_string(&value)
}

fn recolor_in_place(value: &mut Value, r: f64, g: f64, b: f64) {
    match value {
        Value::Object(map) => {
            let is_fill_or_stroke =
                matches!(map.get("ty"), Some(Value::String(t)) if t == "fl" || t == "st");
            if is_fill_or_stroke {
                if let Some(color_prop) = map.get_mut("c") {
                    recolor_color_property(color_prop, r, g, b);
                }
            }
            for v in map.values_mut() {
                recolor_in_place(v, r, g, b);
            }
        }
        Value::Array(items) => {
            for v in items {
                recolor_in_place(v, r, g, b);
            }
        }
        _ => {}
    }
}

fn recolor_color_property(color_prop: &mut Value, r: f64, g: f64, b: f64) {
    let Some(Value::Array(k)) = color_prop.get_mut("k") else {
        return;
    };
    match k.first() {
        Some(Value::Number(_)) => set_rgb(k, r, g, b),

        Some(Value::Object(_)) => {
            for kf in k {
                if let Some(Value::Array(s)) = kf.get_mut("s") {
                    set_rgb(s, r, g, b);
                }
                if let Some(Value::Array(e)) = kf.get_mut("e") {
                    set_rgb(e, r, g, b);
                }
            }
        }
        _ => {}
    }
}

fn set_rgb(rgba: &mut [Value], r: f64, g: f64, b: f64) {
    if rgba.len() >= 3 {
        rgba[0] = serde_json::json!(r);
        rgba[1] = serde_json::json!(g);
        rgba[2] = serde_json::json!(b);
    }
}

#[cfg(test)]
mod tests {
    use skia_safe::AlphaType;
    use skia_safe::Color;
    use skia_safe::ColorType;
    use skia_safe::ImageInfo;
    use skia_safe::Rect;
    use skia_safe::skottie;
    use skia_safe::surfaces;
    use std::env;
    use std::fs;
    use super::*;

    #[test]
    fn leading_subtract_masks_get_a_full_add_mask_in_front() {
        let raw = r#"{"layers":[{"ty":4,"masksProperties":[{"mode":"s","pt":{"a":0,"k":{}}}]},{"ty":4,"masksProperties":[{"mode":"a","pt":{"a":0,"k":{}}}]}]}"#;
        let fixed: Value = serde_json::from_str(&fix_leading_subtract_masks(raw).unwrap()).unwrap();
        let first = fixed["layers"][0]["masksProperties"].as_array().unwrap();
        assert_eq!(first.len(), 2);
        assert_eq!(first[0]["mode"], "a");
        assert_eq!(first[1]["mode"], "s");
        assert_eq!(
            fixed["layers"][1]["masksProperties"]
                .as_array()
                .unwrap()
                .len(),
            1,
            "an additive first mask is left alone"
        );
    }

    fn pristine(name: &str) -> Option<String> {
        let dir = env::var("LOTTIE_PRISTINE_DIR").ok()?;
        fs::read_to_string(format!("{dir}/{name}.json")).ok()
    }

    #[test]
    fn fixes_mask_gaps_on_the_real_mic_icon() {
        let Some(raw) = pristine("mic") else { return };
        let raw = raw.as_str();
        let cleaned = fix_mask_gaps(raw).expect("valid JSON in, valid JSON out");
        let after: Value = serde_json::from_str(&cleaned).unwrap();

        let comp0 = after["assets"][0]["layers"].as_array().unwrap();
        let mprecomp = comp0
            .iter()
            .find(|l| l["nm"] == "Microphone_precomp")
            .unwrap();
        let masks = mprecomp["masksProperties"].as_array().unwrap();
        assert_eq!(
            masks.len(),
            1,
            "the additive mask should be dropped, only the subtract mask kept"
        );
        assert_eq!(masks[0]["mode"], "s");

        assert_eq!(masks[0]["o"]["a"], serde_json::json!(0));
        assert_eq!(masks[0]["o"]["k"], serde_json::json!(100));

        let keyframes = masks[0]["pt"]["k"].as_array().unwrap();

        let gap_start_idx = keyframes
            .iter()
            .position(|k| k["t"].as_f64() == Some(109.0))
            .unwrap();
        assert_eq!(keyframes[gap_start_idx + 1]["t"].as_f64(), Some(117.0));
        assert_eq!(keyframes[gap_start_idx + 2]["t"].as_f64(), Some(230.0));
        let collapsed = &keyframes[gap_start_idx + 1]["s"][0];
        let v = collapsed["v"].as_array().unwrap();
        assert!(
            v.windows(2).all(|w| w[0] == w[1]),
            "all vertices should collapse to the same point"
        );
        for tangent in collapsed["i"]
            .as_array()
            .unwrap()
            .iter()
            .chain(collapsed["o"].as_array().unwrap())
        {
            assert_eq!(
                *tangent,
                serde_json::json!([0, 0]),
                "a degenerate point has no tangent"
            );
        }
    }

    #[test]
    fn leaves_layers_without_masks_untouched() {
        let raw = r#"{"layers":[{"ty":4,"ind":1,"nm":"shape"}]}"#;
        let cleaned = fix_mask_gaps(raw).unwrap();
        let after: Value = serde_json::from_str(&cleaned).unwrap();
        assert_eq!(after["layers"][0]["nm"], "shape");
    }

    #[test]
    fn leaves_small_keyframe_gaps_alone() {
        let raw = r#"{"masksProperties":[{"mode":"s","pt":{"a":1,"k":[
            {"t":0,"s":[{"i":[[0,0]],"o":[[0,0]],"v":[[1,2]],"c":true}]},
            {"t":10,"s":[{"i":[[0,0]],"o":[[0,0]],"v":[[3,4]],"c":true}]},
            {"t":20,"s":[{"i":[[0,0]],"o":[[0,0]],"v":[[5,6]],"c":true}]}
        ]},"o":{"a":0,"k":100}}]}"#;
        let cleaned = fix_mask_gaps(raw).unwrap();
        let after: Value = serde_json::from_str(&cleaned).unwrap();
        assert_eq!(
            after["masksProperties"][0]["pt"]["k"]
                .as_array()
                .unwrap()
                .len(),
            3,
            "no gap large enough to fix, keyframes untouched"
        );
    }

    #[test]
    fn drops_additive_masks_unconditionally() {
        let raw = r#"{"masksProperties":[{"mode":"a","pt":{"a":0,"k":{}},"o":{"a":0,"k":100}},{"mode":"s","pt":{"a":0,"k":{}},"o":{"a":0,"k":100}}]}"#;
        let cleaned = fix_mask_gaps(raw).unwrap();
        let after: Value = serde_json::from_str(&cleaned).unwrap();
        let masks = after["masksProperties"].as_array().unwrap();
        assert_eq!(masks.len(), 1);
        assert_eq!(masks[0]["mode"], "s");
    }

    #[test]
    fn recolors_static_fill_and_stroke() {
        let raw = r#"{"it":[
            {"ty":"fl","c":{"a":0,"k":[0.1,0.2,0.3,1]}},
            {"ty":"st","c":{"a":0,"k":[0.4,0.5,0.6,1]}}
        ]}"#;
        let cleaned = recolor_fills_and_strokes(raw, 0.9, 0.8, 0.7).unwrap();
        let after: Value = serde_json::from_str(&cleaned).unwrap();
        assert_eq!(
            after["it"][0]["c"]["k"],
            serde_json::json!([0.9, 0.8, 0.7, 1])
        );
        assert_eq!(
            after["it"][1]["c"]["k"],
            serde_json::json!([0.9, 0.8, 0.7, 1])
        );
    }

    #[test]
    fn recolors_keyframed_fill() {
        let raw = r#"{"ty":"fl","c":{"a":1,"k":[
            {"t":0,"s":[0.1,0.2,0.3,1],"e":[0.2,0.3,0.4,1]},
            {"t":10,"s":[0.2,0.3,0.4,1]}
        ]}}"#;
        let cleaned = recolor_fills_and_strokes(raw, 0.9, 0.8, 0.7).unwrap();
        let after: Value = serde_json::from_str(&cleaned).unwrap();
        let kfs = after["c"]["k"].as_array().unwrap();
        assert_eq!(kfs[0]["s"], serde_json::json!([0.9, 0.8, 0.7, 1]));
        assert_eq!(kfs[0]["e"], serde_json::json!([0.9, 0.8, 0.7, 1]));
        assert_eq!(kfs[1]["s"], serde_json::json!([0.9, 0.8, 0.7, 1]));
    }

    #[test]
    fn recolors_real_mic_icon_fills() {
        let Some(raw) = pristine("mic") else { return };
        let raw = raw.as_str();
        let cleaned = recolor_fills_and_strokes(raw, 0.671, 0.675, 0.698).unwrap();
        assert!(
            !cleaned.contains("0.345098039216"),
            "the original blurple fill should be gone"
        );
        assert!(
            cleaned.contains("0.671"),
            "the new gray fill should be present"
        );
    }

    #[test]
    fn ignores_non_color_properties() {
        let raw = r#"{"ty":"fl","o":{"a":0,"k":100}}"#;
        let cleaned = recolor_fills_and_strokes(raw, 0.9, 0.8, 0.7).unwrap();
        let after: Value = serde_json::from_str(&cleaned).unwrap();
        assert_eq!(after["o"]["k"], serde_json::json!(100));
    }

    fn render(json: &str, frame: f64, size: i32) -> Vec<u8> {
        let animation = skottie::Animation::from_str(json).expect("valid lottie");
        let mut surface = surfaces::raster_n32_premul((size, size)).unwrap();
        surface.canvas().clear(Color::TRANSPARENT);
        animation.seek_frame(frame);
        animation.render(surface.canvas(), Rect::from_wh(size as f32, size as f32));
        let info = ImageInfo::new((size, size), ColorType::RGBA8888, AlphaType::Premul, None);
        let mut pixels = vec![0u8; (size * size * 4) as usize];
        assert!(surface.read_pixels(&info, &mut pixels, (size * 4) as usize, (0, 0)));
        pixels
    }

    #[test]
    fn pristine_animations_render_content_at_every_frame() {
        for name in ["mic", "deafen", "settings", "gif_button"] {
            let Some(raw) = pristine(name) else { return };
            let json = fix_mask_gaps(&raw).unwrap();
            let doc: Value = serde_json::from_str(&json).unwrap();
            let (ip, op) = (doc["ip"].as_f64().unwrap(), doc["op"].as_f64().unwrap());
            for i in 0..8 {
                let frame = ip + (op - ip - 1.0) * i as f64 / 7.0;
                let pixels = render(&json, frame, 96);
                let opaque = pixels
                    .iter()
                    .skip(3)
                    .step_by(4)
                    .filter(|a| **a > 128)
                    .count();
                assert!(
                    opaque > 50,
                    "{name} frame {frame:.0} drew only {opaque} opaque pixels"
                );
            }
        }
    }
}
