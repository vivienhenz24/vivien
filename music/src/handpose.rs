use std::f32::consts::PI;
use std::path::Path;

use opencv::{core, dnn, imgproc, prelude::*};

use crate::palm::PalmDetection;

const INPUT_SIZE: i32 = 224;
const CONFIDENCE_THRESHOLD: f32 = 0.80;
// Palm detector outputs 7 keypoints (indices 0-6); use all of them in order.
// Index 0 = wrist, index 2 ≈ middle-finger MCP — used for rotation.
const PALM_LANDMARK_IDS: [usize; 7] = [0, 1, 2, 3, 4, 5, 6];
const PALM_BASE_INDEX: usize = 0;
const MIDDLE_FINGER_BASE_INDEX: usize = 2;
const PALM_BOX_PRE_SHIFT_VECTOR: [f32; 2] = [0.0, 0.0];
const PALM_BOX_PRE_ENLARGE_FACTOR: f32 = 4.0;
const PALM_BOX_SHIFT_VECTOR: [f32; 2] = [0.0, -0.4];
const PALM_BOX_ENLARGE_FACTOR: f32 = 3.0;
const HAND_BOX_SHIFT_VECTOR: [f32; 2] = [0.0, -0.1];
const HAND_BOX_ENLARGE_FACTOR: f32 = 1.65;

pub struct HandPose {
    pub bounds: core::Rect,
    pub screen_landmarks: Vec<core::Point3f>,
    pub world_landmarks: Vec<core::Point3f>,
    pub handedness: f32,
    pub confidence: f32,
}

pub struct HandPoseDetector {
    net: dnn::Net,
}

impl HandPoseDetector {
    pub fn new(model_path: &Path) -> opencv::Result<Self> {
        let model_path = model_path.to_string_lossy().into_owned();
        let mut net = dnn::read_net_from_onnx(&model_path)?;
        net.set_preferable_backend(dnn::DNN_BACKEND_OPENCV)?;
        net.set_preferable_target(dnn::DNN_TARGET_CPU)?;
        Ok(Self { net })
    }

    pub fn infer(&mut self, frame: &core::Mat, palm: &PalmDetection) -> opencv::Result<Option<HandPose>> {
        let pre = self.preprocess(frame, palm)?;
        self.net.set_input(&pre.input_blob, "", 1.0, core::Scalar::default())?;

        let out_names = self.net.get_unconnected_out_layers_names()?;
        let mut outputs = core::Vector::<core::Mat>::new();
        self.net.forward(&mut outputs, &out_names)?;
        if outputs.len() < 4 {
            return Err(opencv::Error::new(0, "handpose model returned fewer than 4 tensors"));
        }

        self.postprocess(&outputs, pre)
    }

    fn preprocess(&self, frame: &core::Mat, palm: &PalmDetection) -> opencv::Result<PreprocessResult> {
        let palm_bbox = rect_to_bbox(palm.bounds);
        let palm_landmarks = PALM_LANDMARK_IDS
            .iter()
            .map(|&index| {
                palm.palm_landmarks
                    .get(index)
                    .copied()
                    .ok_or_else(|| opencv::Error::new(0, "missing palm landmark"))
            })
            .collect::<opencv::Result<Vec<_>>>()?;

        let (cropped, padded_palm_bbox, bias) = crop_and_pad_from_palm(
            frame,
            palm_bbox,
            PALM_BOX_PRE_SHIFT_VECTOR,
            PALM_BOX_PRE_ENLARGE_FACTOR,
            true,
        )?;

        let mut rgb = core::Mat::default();
        imgproc::cvt_color(
            &cropped,
            &mut rgb,
            imgproc::COLOR_BGR2RGB,
            0,
            core::AlgorithmHint::ALGO_HINT_DEFAULT,
        )?;

        let local_palm_bbox = shift_bbox(padded_palm_bbox, [-bias[0], -bias[1]]);
        let local_landmarks: Vec<core::Point2f> = palm_landmarks
            .into_iter()
            .map(|point| core::Point2f::new(point.x - bias[0], point.y - bias[1]))
            .collect();

        let p1 = local_landmarks[PALM_BASE_INDEX];
        let p2 = local_landmarks[MIDDLE_FINGER_BASE_INDEX];
        let mut radians = PI / 2.0 - (-(p2.y - p1.y)).atan2(p2.x - p1.x);
        radians -= 2.0 * PI * ((radians + PI) / (2.0 * PI)).floor();
        let angle = radians.to_degrees();

        let center = bbox_center(local_palm_bbox);
        let rotation_matrix =
            imgproc::get_rotation_matrix_2d(core::Point2f::new(center[0], center[1]), angle as f64, 1.0)?;

        let mut rotated = core::Mat::default();
        imgproc::warp_affine(
            &rgb,
            &mut rotated,
            &rotation_matrix,
            rgb.size()?,
            imgproc::INTER_LINEAR,
            core::BORDER_CONSTANT,
            core::Scalar::default(),
        )?;

        let rot = affine_matrix(&rotation_matrix)?;
        let rotated_palm_landmarks: Vec<[f32; 2]> = local_landmarks
            .iter()
            .map(|point| apply_affine_no_translation(rot, [point.x, point.y]))
            .collect();
        let rotated_palm_bbox = bbox_from_points(&rotated_palm_landmarks);

        let (hand_crop, rotated_hand_bbox, hand_bias) = crop_and_pad_from_palm(
            &rotated,
            rotated_palm_bbox,
            PALM_BOX_SHIFT_VECTOR,
            PALM_BOX_ENLARGE_FACTOR,
            false,
        )?;
        let _ = hand_bias;

        let mut resized = core::Mat::default();
        imgproc::resize(
            &hand_crop,
            &mut resized,
            core::Size::new(INPUT_SIZE, INPUT_SIZE),
            0.0,
            0.0,
            imgproc::INTER_AREA,
        )?;

        let mut float = core::Mat::default();
        resized.convert_to(&mut float, core::CV_32F, 1.0 / 255.0, 0.0)?;
        let input_blob = mat_to_nhwc_blob(&float)?;

        Ok(PreprocessResult {
            input_blob,
            rotated_hand_bbox,
            angle,
            rotation_matrix,
            pad_bias: bias,
        })
    }

    fn postprocess(
        &self,
        outputs: &core::Vector<core::Mat>,
        pre: PreprocessResult,
    ) -> opencv::Result<Option<HandPose>> {
        let screen_output = outputs.get(0)?;
        let confidence_output = outputs.get(1)?;
        let handedness_output = outputs.get(2)?;
        let world_output = outputs.get(3)?;

        let confidence_values = mat_to_f32_slice(&confidence_output)?;
        let confidence = *confidence_values.first().unwrap_or(&0.0);
        if confidence < CONFIDENCE_THRESHOLD {
            return Ok(None);
        }

        let handedness = *mat_to_f32_slice(&handedness_output)?.first().unwrap_or(&0.5);
        let mut screen_landmarks = mat_to_point3f_vec(&screen_output)?;
        let mut world_landmarks = mat_to_point3f_vec(&world_output)?;

        let hand_bbox_size = [
            pre.rotated_hand_bbox[1][0] - pre.rotated_hand_bbox[0][0],
            pre.rotated_hand_bbox[1][1] - pre.rotated_hand_bbox[0][1],
        ];
        let scale = hand_bbox_size[0].max(hand_bbox_size[1]) / INPUT_SIZE as f32;
        let rotation = affine_matrix(&pre.rotation_matrix)?;
        let origin_rotation =
            imgproc::get_rotation_matrix_2d(core::Point2f::new(0.0, 0.0), pre.angle as f64, 1.0)?;
        let origin_rot = affine_matrix(&origin_rotation)?;
        let inverse_rotation = inverse_affine_matrix(&pre.rotation_matrix)?;
        let center = bbox_center(pre.rotated_hand_bbox);
        let original_center = apply_affine(inverse_rotation, center);

        for landmark in &mut screen_landmarks {
            landmark.x = (landmark.x - INPUT_SIZE as f32 / 2.0) * scale;
            landmark.y = (landmark.y - INPUT_SIZE as f32 / 2.0) * scale;
            landmark.z *= scale;

            let rotated_xy = apply_affine_no_translation(origin_rot, [landmark.x, landmark.y]);
            landmark.x = rotated_xy[0] + original_center[0] + pre.pad_bias[0];
            landmark.y = rotated_xy[1] + original_center[1] + pre.pad_bias[1];
        }

        for landmark in &mut world_landmarks {
            let rotated_xy = apply_affine_no_translation(origin_rot, [landmark.x, landmark.y]);
            landmark.x = rotated_xy[0];
            landmark.y = rotated_xy[1];
        }

        let bbox_points: Vec<[f32; 2]> =
            screen_landmarks.iter().map(|point| [point.x, point.y]).collect();
        let mut bbox = bbox_from_points(&bbox_points);
        let wh = [
            bbox[1][0] - bbox[0][0],
            bbox[1][1] - bbox[0][1],
        ];
        bbox = shift_bbox(
            bbox,
            [HAND_BOX_SHIFT_VECTOR[0] * wh[0], HAND_BOX_SHIFT_VECTOR[1] * wh[1]],
        );
        let center = bbox_center(bbox);
        let half = [
            wh[0] * HAND_BOX_ENLARGE_FACTOR / 2.0,
            wh[1] * HAND_BOX_ENLARGE_FACTOR / 2.0,
        ];
        bbox = [
            [center[0] - half[0], center[1] - half[1]],
            [center[0] + half[0], center[1] + half[1]],
        ];

        let bounds = clamp_rect_from_bbox(bbox);

        let _ = rotation;
        Ok(Some(HandPose {
            bounds,
            screen_landmarks,
            world_landmarks,
            handedness,
            confidence,
        }))
    }
}

struct PreprocessResult {
    input_blob: core::Mat,
    rotated_hand_bbox: [[f32; 2]; 2],
    angle: f32,
    rotation_matrix: core::Mat,
    pad_bias: [f32; 2],
}

fn rect_to_bbox(rect: core::Rect) -> [[f32; 2]; 2] {
    [
        [rect.x as f32, rect.y as f32],
        [(rect.x + rect.width) as f32, (rect.y + rect.height) as f32],
    ]
}

fn crop_and_pad_from_palm(
    image: &core::Mat,
    mut bbox: [[f32; 2]; 2],
    shift_vector: [f32; 2],
    enlarge_factor: f32,
    for_rotation: bool,
) -> opencv::Result<(core::Mat, [[f32; 2]; 2], [f32; 2])> {
    let wh = [bbox[1][0] - bbox[0][0], bbox[1][1] - bbox[0][1]];
    bbox = shift_bbox(bbox, [shift_vector[0] * wh[0], shift_vector[1] * wh[1]]);

    let center = bbox_center(bbox);
    let new_half = [wh[0] * enlarge_factor / 2.0, wh[1] * enlarge_factor / 2.0];
    bbox = [
        [center[0] - new_half[0], center[1] - new_half[1]],
        [center[0] + new_half[0], center[1] + new_half[1]],
    ];

    let size = image.size()?;
    bbox[0][0] = bbox[0][0].clamp(0.0, size.width as f32);
    bbox[0][1] = bbox[0][1].clamp(0.0, size.height as f32);
    bbox[1][0] = bbox[1][0].clamp(0.0, size.width as f32);
    bbox[1][1] = bbox[1][1].clamp(0.0, size.height as f32);

    let rect = clamp_rect_from_bbox(bbox);
    let cropped = image.roi(rect)?;
    let mut crop = core::Mat::default();
    cropped.copy_to(&mut crop)?;

    let crop_size = crop.size()?;
    let side_len = if for_rotation {
        ((crop_size.width.pow(2) + crop_size.height.pow(2)) as f64).sqrt().ceil() as i32
    } else {
        crop_size.width.max(crop_size.height)
    };

    let pad_h = side_len - crop_size.height;
    let pad_w = side_len - crop_size.width;
    let left = pad_w / 2;
    let top = pad_h / 2;
    let right = pad_w - left;
    let bottom = pad_h - top;

    let mut padded = core::Mat::default();
    core::copy_make_border(
        &crop,
        &mut padded,
        top,
        bottom,
        left,
        right,
        core::BORDER_CONSTANT,
        core::Scalar::default(),
    )?;

    let bias = [rect.x as f32 - left as f32, rect.y as f32 - top as f32];
    Ok((padded, rect_to_bbox(rect), bias))
}

fn bbox_center(bbox: [[f32; 2]; 2]) -> [f32; 2] {
    [
        (bbox[0][0] + bbox[1][0]) / 2.0,
        (bbox[0][1] + bbox[1][1]) / 2.0,
    ]
}

fn shift_bbox(bbox: [[f32; 2]; 2], shift: [f32; 2]) -> [[f32; 2]; 2] {
    [
        [bbox[0][0] + shift[0], bbox[0][1] + shift[1]],
        [bbox[1][0] + shift[0], bbox[1][1] + shift[1]],
    ]
}

fn bbox_from_points(points: &[[f32; 2]]) -> [[f32; 2]; 2] {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for point in points {
        min_x = min_x.min(point[0]);
        min_y = min_y.min(point[1]);
        max_x = max_x.max(point[0]);
        max_y = max_y.max(point[1]);
    }
    [[min_x, min_y], [max_x, max_y]]
}

fn clamp_rect_from_bbox(bbox: [[f32; 2]; 2]) -> core::Rect {
    let left = bbox[0][0].floor() as i32;
    let top = bbox[0][1].floor() as i32;
    let right = bbox[1][0].ceil() as i32;
    let bottom = bbox[1][1].ceil() as i32;
    core::Rect::new(left, top, (right - left).max(1), (bottom - top).max(1))
}

fn affine_matrix(mat: &core::Mat) -> opencv::Result<[[f32; 3]; 2]> {
    Ok([
        [
            *mat.at_2d::<f64>(0, 0)? as f32,
            *mat.at_2d::<f64>(0, 1)? as f32,
            *mat.at_2d::<f64>(0, 2)? as f32,
        ],
        [
            *mat.at_2d::<f64>(1, 0)? as f32,
            *mat.at_2d::<f64>(1, 1)? as f32,
            *mat.at_2d::<f64>(1, 2)? as f32,
        ],
    ])
}

fn inverse_affine_matrix(mat: &core::Mat) -> opencv::Result<[[f32; 3]; 2]> {
    let mut inverse = core::Mat::default();
    imgproc::invert_affine_transform(mat, &mut inverse)?;
    affine_matrix(&inverse)
}

fn apply_affine(mat: [[f32; 3]; 2], point: [f32; 2]) -> [f32; 2] {
    [
        point[0] * mat[0][0] + point[1] * mat[0][1] + mat[0][2],
        point[0] * mat[1][0] + point[1] * mat[1][1] + mat[1][2],
    ]
}

fn apply_affine_no_translation(mat: [[f32; 3]; 2], point: [f32; 2]) -> [f32; 2] {
    [
        point[0] * mat[0][0] + point[1] * mat[0][1],
        point[0] * mat[1][0] + point[1] * mat[1][1],
    ]
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

fn mat_to_point3f_vec(mat: &core::Mat) -> opencv::Result<Vec<core::Point3f>> {
    let values = mat_to_f32_slice(mat)?;
    if values.len() < 63 {
        return Err(opencv::Error::new(0, "expected 63 hand landmark values"));
    }
    Ok(values[..63]
        .chunks_exact(3)
        .map(|chunk| core::Point3f::new(chunk[0], chunk[1], chunk[2]))
        .collect())
}
