pub struct SystemVars {
    pub current_dir: u32,
}

impl Default for SystemVars {
    fn default() -> Self {
        SystemVars {
            current_dir: 0x0,
        }
    }   
}
pub static mut system_vars: SystemVars = SystemVars { current_dir: 0 };