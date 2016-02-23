trait Handler {
    fn incoming(&mut self, address: SocketAddr, bytes: &[u8]) {
        
    }
    
    fn outgoing(&mut self, address: SocketAddr, bytes: &[u8]) {
        
    }
}