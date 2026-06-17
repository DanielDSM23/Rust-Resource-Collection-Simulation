pub struct Simulation {
    pub state: SharedState,
    robot_handles: Vec<JoinHandle<()>>,
    message_handle: Option<JoinHandle<()>>,
}

impl Simulation {
    pub fn new(width: usize, height: usize) {
        let map = GameMap::generate(width, height);
        let base = Base::new(map.base_pos);
        
    }
    
}