use std::{collections::VecDeque, sync::Arc, thread::JoinHandle};

use parking_lot::{Condvar, Mutex};

use crate::{event::RunnerEvent, ThreadRunner};

pub fn spawn_input_thread<T>(
    queue: Arc<(Mutex<VecDeque<RunnerEvent>>, Condvar)>,
    runner: Arc<Mutex<T>>,
) -> JoinHandle<()>
where
    T: ThreadRunner + Send + Sync + 'static,
{
    std::thread::spawn(move || {
        let (lock, cvar) = &*queue;
        let mut queue = lock.lock();
        loop {
            let len = queue.len();
            if len > 20 {
                log::warn!("input queue backed up: {}", len)
            }
            while let Some(event) = queue.pop_front() {
                let mut runner = runner.lock();
                runner.input(event);
            }
            cvar.wait(&mut queue);
        }
    })
}
