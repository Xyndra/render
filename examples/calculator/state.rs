use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

use crate::logic::{
    CalculationPart,
    CalculationResult::{self, Number},
};

lazy_static! {
    pub static ref EQUATION: Arc<Mutex<Vec<CalculationPart>>> = Arc::new(Mutex::new(vec![]));
    pub static ref LAST_RESULT: Arc<Mutex<CalculationResult>> = Arc::new(Mutex::new(Number(0.0)));
}
