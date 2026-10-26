use std::{
    collections::VecDeque,
    sync::Arc,
    time::Duration,
};

#[derive(Debug)]
pub struct VideoBlock {
    pub id: u32,
    pub data: Vec<u8>,
    pub duration: Duration,
}

pub struct ReplayBuffer {
    blocks: VecDeque<Arc<VideoBlock>>,
    target_duration: Duration,
    total_duration: Duration,
}

impl ReplayBuffer {
    pub fn new(target_duration: Duration) -> Result<Self, String> {
        if target_duration.is_zero() {
            return Err("Buffer must have at least one block".to_string());
        }

        Ok(Self {
            blocks: VecDeque::new(),
            target_duration,
            total_duration: Duration::ZERO,
        })
    }

    pub fn push(&mut self, block: VideoBlock) -> Result<(), String> {
        if block.duration.is_zero() {
            return Err("Block must be longer than 0".to_string());
        }

        self.total_duration = self.total_duration
            .checked_add(block.duration)
            .ok_or("Sum is too big".to_string())?;

        self.blocks.push_back(Arc::new(block));

        while let Some(oldest) = self.blocks.front() {
            let remaining = self.total_duration - oldest.duration;

            if remaining < self.target_duration {
                break;
            }

            self.blocks.pop_front();
            self.total_duration = remaining;
        }

        Ok(())
    }

    pub fn snapshot(&self) -> Vec<Arc<VideoBlock>> {
        self.blocks.iter().map(Arc::clone).collect()
    }
}
