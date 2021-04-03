use std::{sync::Arc, thread::JoinHandle, time::Duration};

use parking_lot::Mutex;
use winit::window::Window;

use crate::{render::RenderState, ThreadRunner};

const TICK_RATE: u32 = 100;

pub fn spawn_update_thread<T>(
    runner: Arc<Mutex<T>>,
    renderer: Arc<RenderState>,
    window: Arc<Window>,
) -> JoinHandle<()>
where
    T: ThreadRunner + Send + Sync + 'static,
{
    std::thread::spawn(move || {
        let mut clock = crate::clock::Clock::new(TICK_RATE);
        let mut delta = (
            TICK_RATE as f32,
            Duration::from_secs_f32(1.0 / (TICK_RATE as f32)),
        );
        loop {
            clock.tick();
            clock.target_rate =
                runner
                    .lock()
                    .update(&window, &renderer.device, &renderer.queue, delta);
            delta = clock.wait();
        }
    })
}
