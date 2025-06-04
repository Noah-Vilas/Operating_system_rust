pub struct KeyboardInputBuffer {
    pub front: usize,
    pub back: usize,
    pub buffer: [u8; 128],
    
}
impl KeyboardInputBuffer {



    pub fn read_key(&mut self) -> Option<u8> {
        if self.front == self.back {
            return None; // Buffer empty
        }
        let key = self.buffer[self.front];
        self.front = (self.front + 1) % self.buffer.len();
        Some(key)
    }

    pub fn add_key(&mut self, key: u8) -> bool {
        let next_back = (self.back + 1) % self.buffer.len();
        if next_back == self.front {
            return false;
        }
        self.buffer[self.back] = key;
        self.back = next_back;
        true
    }

}