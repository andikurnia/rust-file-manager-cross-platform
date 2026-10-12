use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct NavigationState {
    pub current: PathBuf,
    pub back_stack: Vec<PathBuf>,
    pub forward_stack: Vec<PathBuf>,
}

impl NavigationState {
    pub fn new(initial: PathBuf) -> Self {
        Self {
            current: initial,
            back_stack: Vec::new(),
            forward_stack: Vec::new(),
        }
    }

    pub fn navigate_to(&mut self, path: PathBuf) {
        self.back_stack.push(self.current.clone());
        self.current = path;
        self.forward_stack.clear();
    }

    pub fn go_back(&mut self) -> Option<PathBuf> {
        if let Some(prev) = self.back_stack.pop() {
            self.forward_stack.push(self.current.clone());
            self.current = prev.clone();
            Some(prev)
        } else {
            None
        }
    }

    pub fn go_forward(&mut self) -> Option<PathBuf> {
        if let Some(next) = self.forward_stack.pop() {
            self.back_stack.push(self.current.clone());
            self.current = next.clone();
            Some(next)
        } else {
            None
        }
    }

    pub fn can_go_back(&self) -> bool {
        !self.back_stack.is_empty()
    }

    pub fn can_go_forward(&self) -> bool {
        !self.forward_stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_back_forward() {
        let mut nav = NavigationState::new("/home".into());
        nav.navigate_to("/home/Documents".into());
        nav.navigate_to("/home/Downloads".into());

        assert!(nav.can_go_back());
        assert!(!nav.can_go_forward());

        let back = nav.go_back().unwrap();
        assert_eq!(back, std::path::PathBuf::from("/home/Documents"));
        assert!(nav.can_go_forward());

        let forward = nav.go_forward().unwrap();
        assert_eq!(forward, std::path::PathBuf::from("/home/Downloads"));
    }

    #[test]
    fn test_navigation_clears_forward_on_new_nav() {
        let mut nav = NavigationState::new("/home".into());
        nav.navigate_to("/home/Documents".into());
        nav.go_back();
        assert!(nav.can_go_forward());

        nav.navigate_to("/home/Pictures".into());
        assert!(!nav.can_go_forward());
    }
}
