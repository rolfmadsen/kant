//! Fokus- og tastaturstyring for modaler, formularer og handlingsknapper.

/// Repræsenterer et fokuserbart mål i en formular eller modal: enten et tekstfelt (med et unikt ID) eller en handlingsknap (med et indeks).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusTarget {
    Input(&'static str),
    Button(usize),
}

/// Styrer cyklisk fokus over et ordnet sæt af elementer (inputfelter og/eller handlingsknapper).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FocusCycle {
    current: Option<usize>,
    targets: Vec<FocusTarget>,
}

impl FocusCycle {
    /// Opretter en ny fokuscyklus over `count` knapper, hvor knap 0 har fokus hvis count > 0.
    pub fn new(count: usize) -> Self {
        let targets = (0..count).map(FocusTarget::Button).collect();
        Self {
            current: if count > 0 { Some(0) } else { None },
            targets,
        }
    }

    /// Opretter en ny fokuscyklus over knapper med et specifikt initialt fokus.
    pub fn with_initial(count: usize, initial: Option<usize>) -> Self {
        let valid_initial = match initial {
            Some(i) if i < count => Some(i),
            _ => None,
        };
        let targets = (0..count).map(FocusTarget::Button).collect();
        Self {
            current: valid_initial,
            targets,
        }
    }

    /// Opretter en ny fokuscyklus med en vilkårlig liste af fokuserbare mål (inputfelter og knapper).
    pub fn with_targets(targets: Vec<FocusTarget>, initial: Option<usize>) -> Self {
        let count = targets.len();
        let valid_initial = match initial {
            Some(i) if i < count => Some(i),
            _ => None,
        };
        Self {
            current: valid_initial,
            targets,
        }
    }

    /// Returnerer det aktuelle mål (enten tekstfelt eller knap).
    pub fn current_target(&self) -> Option<&FocusTarget> {
        self.current.and_then(|idx| self.targets.get(idx))
    }

    /// Returnerer knap-indekset, hvis det aktuelle mål er en knap.
    pub fn focused_button(&self) -> Option<usize> {
        match self.current_target() {
            Some(FocusTarget::Button(b)) => Some(*b),
            _ => None,
        }
    }

    /// Returnerer tekstfelt-ID'et, hvis det aktuelle mål er et tekstfelt.
    pub fn focused_input(&self) -> Option<&'static str> {
        match self.current_target() {
            Some(FocusTarget::Input(id)) => Some(*id),
            _ => None,
        }
    }

    /// Returnerer indekset for det aktuelt fokuserede mål i listen.
    pub fn current(&self) -> Option<usize> {
        self.current
    }

    /// Sætter eksplicit det fokuserede indeks.
    pub fn set_current(&mut self, index: Option<usize>) {
        if let Some(i) = index {
            if i < self.targets.len() {
                self.current = Some(i);
                return;
            }
        }
        self.current = None;
    }

    /// Synkroniserer det aktuelle fokus til et specifikt tekstfelt via dets ID.
    pub fn sync_input(&mut self, input_id: &str) {
        if let Some(pos) = self.targets.iter().position(|t| match t {
            FocusTarget::Input(id) => *id == input_id,
            _ => false,
        }) {
            self.current = Some(pos);
        }
    }

    /// Returnerer sandt hvis den givne knap aktuelt er fokuseret.
    pub fn is_focused(&self, button_index: usize) -> bool {
        self.focused_button() == Some(button_index)
    }

    /// Alias for `is_focused` for semantisk klarhed.
    pub fn is_button_focused(&self, button_index: usize) -> bool {
        self.is_focused(button_index)
    }

    /// Rykker fokus til næste element (Tab). Looper rundt til starten hvis bunden nås.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<usize> {
        if self.targets.is_empty() {
            self.current = None;
            return None;
        }
        self.current = match self.current {
            None => Some(0),
            Some(idx) => {
                if idx + 1 < self.targets.len() {
                    Some(idx + 1)
                } else {
                    Some(0)
                }
            }
        };
        self.current
    }

    /// Rykker fokus til forrige element (Shift+Tab). Looper rundt til slutningen hvis toppen nås.
    pub fn previous(&mut self) -> Option<usize> {
        if self.targets.is_empty() {
            self.current = None;
            return None;
        }
        self.current = match self.current {
            None => Some(self.targets.len().saturating_sub(1)),
            Some(0) => Some(self.targets.len().saturating_sub(1)),
            Some(idx) => Some(idx - 1),
        };
        self.current
    }

    /// Nulstiller fokus, så intet element har fokus.
    pub fn reset(&mut self) {
        self.current = None;
        self.targets.clear();
    }

    /// Returnerer det samlede antal elementer i cyklussen.
    pub fn count(&self) -> usize {
        self.targets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_cycle_navigation() {
        let mut cycle = FocusCycle::new(3);
        assert_eq!(cycle.current(), Some(0));
        assert!(cycle.is_focused(0));
        assert!(!cycle.is_focused(1));

        // Tab -> 1
        assert_eq!(cycle.next(), Some(1));
        // Tab -> 2
        assert_eq!(cycle.next(), Some(2));
        // Tab -> wrap to 0
        assert_eq!(cycle.next(), Some(0));

        // Shift+Tab -> wrap to 2
        assert_eq!(cycle.previous(), Some(2));
        // Shift+Tab -> 1
        assert_eq!(cycle.previous(), Some(1));
        // Shift+Tab -> 0
        assert_eq!(cycle.previous(), Some(0));
    }

    #[test]
    fn test_focus_cycle_with_initial_none() {
        let mut cycle = FocusCycle::with_initial(2, None);
        assert_eq!(cycle.current(), None);
        assert_eq!(cycle.next(), Some(0));
        assert_eq!(cycle.next(), Some(1));
        assert_eq!(cycle.next(), Some(0));
    }

    #[test]
    fn test_empty_focus_cycle() {
        let mut cycle = FocusCycle::new(0);
        assert_eq!(cycle.current(), None);
        assert_eq!(cycle.next(), None);
        assert_eq!(cycle.previous(), None);
    }

    #[test]
    fn test_focus_cycle_with_mixed_targets() {
        let targets = vec![
            FocusTarget::Input("remote_url"),
            FocusTarget::Input("target_dir"),
            FocusTarget::Button(0),
            FocusTarget::Button(1),
        ];
        let mut cycle = FocusCycle::with_targets(targets, Some(0));

        // Initial: target 0 is Input("remote_url")
        assert_eq!(cycle.focused_input(), Some("remote_url"));
        assert_eq!(cycle.focused_button(), None);
        assert!(!cycle.is_focused(0));

        // Tab -> target 1: Input("target_dir")
        assert_eq!(cycle.next(), Some(1));
        assert_eq!(cycle.focused_input(), Some("target_dir"));
        assert_eq!(cycle.focused_button(), None);

        // Tab -> target 2: Button(0)
        assert_eq!(cycle.next(), Some(2));
        assert_eq!(cycle.focused_input(), None);
        assert_eq!(cycle.focused_button(), Some(0));
        assert!(cycle.is_focused(0));
        assert!(!cycle.is_focused(1));

        // Tab -> target 3: Button(1)
        assert_eq!(cycle.next(), Some(3));
        assert_eq!(cycle.focused_button(), Some(1));
        assert!(cycle.is_focused(1));

        // Tab -> wrap to target 0: Input("remote_url")
        assert_eq!(cycle.next(), Some(0));
        assert_eq!(cycle.focused_input(), Some("remote_url"));
        assert_eq!(cycle.focused_button(), None);

        // Sync input: click target_dir
        cycle.sync_input("target_dir");
        assert_eq!(cycle.current(), Some(1));
        assert_eq!(cycle.focused_input(), Some("target_dir"));
    }
}
