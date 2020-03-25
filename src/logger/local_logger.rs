use crate::logger::traits::Logger;
use log::{info};

pub struct LocalLogger {}


impl Logger for LocalLogger {
    fn log(&self, message: String) {
	info!("{}", message);
    }

    fn present(&self, message: String) {
	info!("{}", message);
    }
}
