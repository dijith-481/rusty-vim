use std::collections::HashSet;
pub struct PendingOperations {
    pub repeat: usize,
    pub action: char,
    valid_actions: HashSet<char>,
    valid_modifiers: HashSet<char>,
    valid_motions: HashSet<char>,
    pub modifier: char,
    pub motion: char,
}

impl PendingOperations {
    pub fn new() -> PendingOperations {
        let keys_action = ['d', 'f', 'g'];
        let valid_actions: HashSet<char> = keys_action.iter().cloned().collect();
        let keys_modifier = ['i', 'a', 'f'];
        let valid_modifiers: HashSet<char> = keys_modifier.iter().cloned().collect();
        let keys_motion = [
            'h', 'j', 'k', 'l', 'x', 'd', 'g', 'G', 'a', 'i', 'A', 'o', 'O', 'H', 'M', 'L', 'w',
            'W', 'e', '{', '}', ':', 'y', '^', '$', '0', '\x7F',
        ];
        let valid_motions: HashSet<char> = keys_motion.iter().cloned().collect();
        Self {
            repeat: 0,
            action: 0 as char,
            modifier: 0 as char,
            motion: 0 as char,
            valid_actions,
            valid_modifiers,
            valid_motions,
        }
    }
    pub fn reset(&mut self) {
        self.repeat = 0;
        self.action = 0 as char;
        self.modifier = 0 as char;
        self.motion = 0 as char;
    }
    pub fn is_action_given(&self) -> bool {
        self.action != '\0'
    }
    fn is_modifier_given(&self) -> bool {
        self.modifier != '\0'
    }
    pub fn is_motion_given(&self) -> bool {
        self.motion != '\0'
    }
    pub fn insert_key(&mut self, key: char) {
        if key != '0' && key.is_numeric() {
            self.repeat = self.repeat.saturating_mul(10);
            self.repeat = self
                .repeat
                .saturating_add(key.to_digit(10).map_or(0, |digit| digit as usize));
        } else if !self.is_action_given() && self.valid_actions.contains(&key) {
            self.action = key;
        } else if !self.is_modifier_given()
            && self.is_motion_given()
            && self.valid_modifiers.contains(&key)
        {
            self.modifier = key;
        } else if !self.is_motion_given() && self.valid_motions.contains(&key) {
            self.motion = key;
        } else {
        }
    }
}
