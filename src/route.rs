// Favors newer rtt values over old ones.
const RTT_ALPHA: f64 = 0.75;

struct RouteInfo {
    rtt:     u64,
    address: SocketAddr
}

impl RouteInfo {
    pub fn new(rtt: u64, address: SocketAddr) -> RouteInfo {
        RouterInfo{ rtt: rtt, address: address}
    }
    
    pub fn compound_rtt(old_rtt: u64, new_rtt: u64, address: SocketAddr) -> RouteInfo {
        RouteInfo::new(compute_rtt(old_rtt, new_rtt, RTT_ALPHA), address)
    }
    
    pub fn address(&self) -> SocketAddr {
        self.address
    }
    
    pub fn average_rtt(&self) -> u64 {
        self.rtt
    }
}

fn compute_rtt(old_rtt: u64, new_rtt: u64, alpha: f64) -> u64 {
    let (f_old_rtt, f_new_rtt) = (old_rtt as f64, new_rtt as f64);
    
    ((alpha * f_old_rtt) + ((1 - alpha) * f_new_rtt)).round() as u64
}