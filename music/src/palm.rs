use std::path::Path;

use opencv::{core, dnn, imgproc, prelude::*};

const INPUT_SIZE: i32 = 192;
const SCORE_THRESHOLD: f32 = 0.50; // lower so a second angled hand still passes
const NMS_THRESHOLD: f32 = 0.50;  // was 0.30 — too aggressive, suppressed 2nd hand
const TOP_K: i32 = 0;

pub struct PalmDetection {
    pub bounds: core::Rect,
    pub palm_landmarks: Vec<core::Point2f>,
    pub score: f32,
}

pub struct PalmDetector {
    net: dnn::Net,
    anchors: Vec<[f32; 2]>,
}

impl PalmDetector {
    pub fn new(model_path: &Path) -> opencv::Result<Self> {
        let model_path = model_path.to_string_lossy().into_owned();
        let mut net = dnn::read_net_from_onnx(&model_path)?;
        net.set_preferable_backend(dnn::DNN_BACKEND_OPENCV)?;
        net.set_preferable_target(dnn::DNN_TARGET_CPU)?;

        Ok(Self {
            net,
            anchors: generate_anchors(),
        })
    }

    pub fn detect(&mut self, frame: &core::Mat) -> opencv::Result<Vec<PalmDetection>> {
        let frame_size = frame.size()?;
        if frame_size.width <= 0 || frame_size.height <= 0 {
            return Ok(Vec::new());
        }

        let (input_blob, pad_bias, scale) = self.preprocess(frame)?;
        self.net
            .set_input(&input_blob, "", 1.0, core::Scalar::default())?;

        let out_names = self.net.get_unconnected_out_layers_names()?;
        let mut outputs = core::Vector::<core::Mat>::new();
        self.net.forward(&mut outputs, &out_names)?;

        let (box_output, score_output) = classify_outputs(&outputs)?;
        self.postprocess(&box_output, &score_output, frame_size, pad_bias, scale)
    }

    fn preprocess(&self, frame: &core::Mat) -> opencv::Result<(core::Mat, [f32; 2], f32)> {
        let size = frame.size()?;
        let scale =
            (INPUT_SIZE as f32 / size.width as f32).min(INPUT_SIZE as f32 / size.height as f32);
        let resized_width = ((size.width as f32) * scale) as i32;
        let resized_height = ((size.height as f32) * scale) as i32;

        let mut resized = core::Mat::default();
        imgproc::resize(
            frame,
            &mut resized,
            core::Size::new(resized_width, resized_height),
            0.0,
            0.0,
            imgproc::INTER_LINEAR,
        )?;

        let pad_w = INPUT_SIZE - resized_width;
        let pad_h = INPUT_SIZE - resized_height;
        let left = pad_w / 2;
        let top = pad_h / 2;
        let right = pad_w - left;
        let bottom = pad_h - top;

        let mut padded = core::Mat::default();
        core::copy_make_border(
            &resized,
            &mut padded,
            top,
            bottom,
            left,
            right,
            core::BORDER_CONSTANT,
            core::Scalar::default(),
        )?;

        let mut rgb = core::Mat::default();
        imgproc::cvt_color(
            &padded,
            &mut rgb,
            imgproc::COLOR_BGR2RGB,
            0,
            core::AlgorithmHint::ALGO_HINT_DEFAULT,
        )?;

        let mut rgb_float = core::Mat::default();
        rgb.convert_to(&mut rgb_float, core::CV_32F, 1.0 / 255.0, 0.0)?;

        let input_blob = mat_to_nhwc_blob(&rgb_float)?;
        let pad_bias = [left as f32 / scale, top as f32 / scale];
        Ok((input_blob, pad_bias, scale))
    }

    fn postprocess(
        &self,
        box_output: &core::Mat,
        score_output: &core::Mat,
        frame_size: core::Size,
        pad_bias: [f32; 2],
        scale: f32,
    ) -> opencv::Result<Vec<PalmDetection>> {
        let box_values = mat_to_f32_slice(box_output)?;
        let score_values = mat_to_f32_slice(score_output)?;
        let num_anchors = self.anchors.len().min(score_values.len());
        let box_stride = box_values.len() / num_anchors.max(1);
        if box_stride < 18 || num_anchors == 0 {
            return Ok(Vec::new());
        }

        let image_scale = INPUT_SIZE as f32 / scale;
        let mut boxes = core::Vector::<core::Rect>::new();
        let mut scores = core::Vector::<f32>::new();
        let mut landmark_sets: Vec<Vec<core::Point2f>> = Vec::new();

        for i in 0..num_anchors {
            let score = sigmoid(score_values[i]);
            if score < SCORE_THRESHOLD {
                continue;
            }

            let base = i * box_stride;
            let anchor = self.anchors[i];
            let cx = (box_values[base] / INPUT_SIZE as f32 + anchor[0]) * image_scale - pad_bias[0];
            let cy =
                (box_values[base + 1] / INPUT_SIZE as f32 + anchor[1]) * image_scale - pad_bias[1];
            let width = (box_values[base + 2] / INPUT_SIZE as f32) * image_scale;
            let height = (box_values[base + 3] / INPUT_SIZE as f32) * image_scale;

            let x1 = cx - width / 2.0;
            let y1 = cy - height / 2.0;
            let x2 = cx + width / 2.0;
            let y2 = cy + height / 2.0;
            let rect = clamp_rect(x1, y1, x2, y2, frame_size.width, frame_size.height);
            if rect.width <= 0 || rect.height <= 0 {
                continue;
            }

            let mut palm_landmarks = Vec::with_capacity(7);
            for landmark_idx in 0..7 {
                let lx =
                    (box_values[base + 4 + landmark_idx * 2] / INPUT_SIZE as f32 + anchor[0])
                        * image_scale
                        - pad_bias[0];
                let ly =
                    (box_values[base + 5 + landmark_idx * 2] / INPUT_SIZE as f32 + anchor[1])
                        * image_scale
                        - pad_bias[1];
                palm_landmarks.push(core::Point2f::new(lx, ly));
            }

            boxes.push(rect);
            scores.push(score);
            landmark_sets.push(palm_landmarks);
        }

        if boxes.is_empty() {
            return Ok(Vec::new());
        }

        let mut keep = core::Vector::<i32>::new();
        dnn::nms_boxes(
            &boxes,
            &scores,
            SCORE_THRESHOLD,
            NMS_THRESHOLD,
            &mut keep,
            1.0,
            TOP_K,
        )?;

        let mut detections = Vec::with_capacity(keep.len());
        for keep_idx in keep.iter() {
            let idx = keep_idx as usize;
            detections.push(PalmDetection {
                bounds: boxes.get(idx)?,
                palm_landmarks: landmark_sets[idx].clone(),
                score: scores.get(idx)?,
            });
        }
        detections.sort_by(|a, b| a.bounds.x.cmp(&b.bounds.x));
        Ok(detections)
    }
}

fn classify_outputs(outputs: &core::Vector<core::Mat>) -> opencv::Result<(core::Mat, core::Mat)> {
    if outputs.len() < 2 {
        return Err(opencv::Error::new(
            0,
            "palm detector returned fewer than 2 output tensors",
        ));
    }

    let first = outputs.get(0)?;
    let second = outputs.get(1)?;
    let first_last_dim = last_dim(&first)?;
    let second_last_dim = last_dim(&second)?;

    if first_last_dim > second_last_dim {
        Ok((first, second))
    } else {
        Ok((second, first))
    }
}

fn last_dim(mat: &core::Mat) -> opencv::Result<i32> {
    let dims = mat.dims();
    mat.mat_size().get(dims - 1)
}

fn mat_to_nhwc_blob(image: &core::Mat) -> opencv::Result<core::Mat> {
    let size = image.size()?;
    let mut blob = core::Mat::new_nd_with_default(
        &[1, size.height, size.width, image.channels()],
        core::CV_32F,
        core::Scalar::default(),
    )?;

    let byte_len =
        (size.width * size.height * image.channels() * std::mem::size_of::<f32>() as i32) as usize;
    unsafe {
        std::ptr::copy_nonoverlapping(image.data(), blob.data_mut(), byte_len);
    }
    Ok(blob)
}

fn mat_to_f32_slice(mat: &core::Mat) -> opencv::Result<&[f32]> {
    if !mat.is_continuous() {
        return Err(opencv::Error::new(0, "expected continuous output tensor"));
    }

    let total = mat.total() as usize;
    let ptr = mat.data() as *const f32;
    if ptr.is_null() {
        return Err(opencv::Error::new(0, "output tensor is empty"));
    }

    Ok(unsafe { std::slice::from_raw_parts(ptr, total) })
}

fn clamp_rect(x1: f32, y1: f32, x2: f32, y2: f32, max_width: i32, max_height: i32) -> core::Rect {
    let left = x1.floor().max(0.0).min(max_width as f32) as i32;
    let top = y1.floor().max(0.0).min(max_height as f32) as i32;
    let right = x2.ceil().max(0.0).min(max_width as f32) as i32;
    let bottom = y2.ceil().max(0.0).min(max_height as f32) as i32;
    core::Rect::new(left, top, (right - left).max(0), (bottom - top).max(0))
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

fn generate_anchors() -> Vec<[f32; 2]> {
    let mut anchors = Vec::with_capacity(2016);

    for y in 0..24 {
        for x in 0..24 {
            let center = [(x as f32 + 0.5) / 24.0, (y as f32 + 0.5) / 24.0];
            anchors.push(center);
            anchors.push(center);
        }
    }

    for y in 0..12 {
        for x in 0..12 {
            let center = [(x as f32 + 0.5) / 12.0, (y as f32 + 0.5) / 12.0];
            for _ in 0..6 {
                anchors.push(center);
            }
        }
    }

    anchors
}
