use std::time::Instant;

use opencv::{core::Mat, prelude::*};

pub struct FramePacket {
    pub frame_id: u64,
    pub captured_at: Instant,
    pub image: Mat,
}

impl FramePacket {
    pub fn new(frame_id: u64, image: Mat) -> Self {
        Self {
            frame_id,
            captured_at: Instant::now(),
            image,
        }
    }

    pub fn duplicate(&self) -> opencv::Result<Self> {
        Ok(Self {
            frame_id: self.frame_id,
            captured_at: self.captured_at,
            image: self.image.try_clone()?,
        })
    }
}
