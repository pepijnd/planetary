use std::{collections::VecDeque, sync::Arc, thread::JoinHandle};

use parking_lot::Mutex;
use winit::window::Window;

use crate::{ThreadRunner, render::RenderState};

const TICK_RATE: u32 = 100;
const TICK_DURATION: std::time::Duration =
    std::time::Duration::from_secs_f32(1.0 / (TICK_RATE as f32));

pub fn spawn_update_thread<T>(
    runner: Arc<Mutex<T>>,
    renderer: Arc<RenderState>,
    window: Arc<Window>
) -> JoinHandle<()>
where
    T: ThreadRunner + Send + Sync + 'static,
{
    std::thread::spawn(move || {
        let mut time = std::time::Instant::now();
        loop {
            let _last_tick = time.elapsed();
            time = std::time::Instant::now();
            runner.lock().update(&window, &renderer.device, &renderer.queue);
            let curr_update = time.elapsed();
            if curr_update < TICK_DURATION {
                std::thread::sleep(TICK_DURATION - curr_update)
            }
        }
    })
}
