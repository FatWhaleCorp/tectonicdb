/// history recorder

use circular_queue::CircularQueue;
use std::{thread, time};
use std::sync::{Arc, RwLock};
use state::SharedState;

pub fn run(global: Arc<RwLock<SharedState>>) {
    let global_copy_timer = global.clone();
    let (granularity, q_capacity) = {
        (global.read().unwrap().settings.hist_granularity.clone(),
         global.read().unwrap().settings.hist_q_capacity.clone(),
        )
    };
    thread::spawn(move || {
        let dur = time::Duration::from_secs(granularity);
        loop {
            {
                let mut rwdr = global_copy_timer.write().unwrap();
                let (total, sizes) = {
                    let mut total = 0;
                    let mut sizes: Vec<(String, u64)> = Vec::new();
                    for (name, vec) in rwdr.vec_store.iter() {
                        let size = vec.1;
                        total += size;
                        sizes.push((name.clone(), size));
                    }
                    sizes.push(("total".to_owned(), total));
                    (total, sizes)
                };

                let current_t = time::SystemTime::now();
                for &(ref name, size) in sizes.iter() {
                    if !rwdr.history.contains_key(name) {
                        rwdr.history.insert(name.clone(), CircularQueue::with_capacity(q_capacity));
                    }
                    rwdr.history.get_mut(name).unwrap().push((current_t, size));
                }

                info!("Current total count: {}", total);
            }

            thread::sleep(dur);
        }
    });
}
