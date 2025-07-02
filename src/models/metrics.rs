
#[derive(Debug, Default)]
pub struct Metrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub fastest_response: f64, 
    pub slowest_response: f64, 
    pub total_duration: f64,  
}
