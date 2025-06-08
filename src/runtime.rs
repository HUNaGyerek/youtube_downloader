use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, MutexGuard,
    },
};

use crate::models::music::Music;

pub trait RuntimeTrait {
    fn get_url_buffer(&self) -> MutexGuard<VecDeque<Music>>;
    fn drain_buffer(&self) -> Vec<Music>;
    fn clear_url_buffer(&self);
    fn start(&self) -> bool;
    fn stop(&self);
}

pub struct Runtime {
    pub url_buffer: Arc<Mutex<VecDeque<Music>>>,
    pub state: Arc<AtomicBool>,
}

impl RuntimeTrait for Runtime {
    fn get_url_buffer(&self) -> MutexGuard<VecDeque<Music>> {
        self.url_buffer.lock().unwrap()
    }

    fn drain_buffer(&self) -> Vec<Music> {
        self.url_buffer.lock().unwrap().drain(..).collect()
    }

    fn clear_url_buffer(&self) {
        let mut buffer = self.url_buffer.lock().unwrap();
        buffer.clear();
    }

    fn start(&self) -> bool {
        self.state.load(Ordering::SeqCst)
    }

    fn stop(&self) {
        self.state.store(false, Ordering::SeqCst);
    }
}
