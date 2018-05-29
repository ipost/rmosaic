extern crate time;
use lib::params::print_timings;
use lib::timing::time::PreciseTime;

pub fn start_timer() -> time::PreciseTime {
    PreciseTime::now()
}

pub fn stop_timer(timer: time::PreciseTime, message: &str) {
    if print_timings() {
        let duration = timer.to(PreciseTime::now()).num_milliseconds();
        println!("{}{}ms", message, duration);
    }
}
