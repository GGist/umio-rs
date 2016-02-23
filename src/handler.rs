trait Handler {
    type Timeout;
    type Message: Send;
    
    fn incoming(&mut self, eloop: &mut ELoop, message: &[u8], route: RouteInfo) {
        ()
    }
    
    fn notify(&mut self, eloop: &mut ELoop, msg: Self::Message) {
        ()
    }
    
    fn timeout(&mut self, elop: &mut ELoop, timeout: Self::Timeout) {
        ()
    }
}