use std::{collections::VecDeque, time::Duration};

pub struct TickBuffer {
    buffer: VecDeque<Duration>,
    first_frame: bool,
}

impl TickBuffer {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
            first_frame: false,
        }
    }

    pub fn push(&mut self, v: Duration, max: usize) {
        if self.first_frame {
            self.buffer.push_back(v);
            while self.buffer.len() > max {
                self.buffer.pop_front();
            }
        }
        self.first_frame = true;
    }
}

impl Default for TickBuffer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Clock {
    last_tick: std::time::Instant,
    pub target_rate: u32,
    buffer: TickBuffer,
}

impl Clock {
    pub fn new(target_rate: u32) -> Self {
        Self {
            last_tick: std::time::Instant::now(),
            target_rate,
            buffer: TickBuffer::new(),
        }
    }

    pub fn tick(&mut self) {
        let tick_time = self.last_tick.elapsed();
        self.last_tick = std::time::Instant::now();
        self.buffer
            .push(tick_time, (self.target_rate / 2).min(1000) as usize);
    }

    pub fn wait(&self) -> (f32, Duration) {
        let rate = self.tick_rate();
        let target_time = Duration::from_secs_f32(if self.buffer.buffer.len() > 1 {
            (rate / self.target_rate as f32).clamp(0.0984, 1.016) / self.target_rate as f32
        } else {
            1.0 / self.target_rate as f32
        });

        let curr_time = self.last_tick.elapsed();
        if curr_time < target_time {
            std::thread::sleep(target_time - curr_time);
        } else {
            std::thread::yield_now();
        }

        (rate, target_time)
    }

    pub fn tick_rate(&self) -> f32 {
        let total = self.buffer.buffer.len();
        total as f32 / self.buffer.buffer.iter().sum::<Duration>().as_secs_f32()
    }
}
