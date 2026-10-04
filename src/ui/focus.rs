//! Fokus- og tastaturstyring for modaler, formularer og handlingsknapper.

/// Styrer cyklisk fokus over et fast antal handlingsknapper eller interaktive elementer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FocusCycle {
    current: Option<usize>,
    count: usize,
}

impl FocusCycle {
    /// Opretter en ny fokuscyklus over `count` elementer, hvor element 0 har fokus hvis count > 0.
    pub fn new(count: usize) -> Self {
        Self {
            current: if count > 0 { Some(0) } else { None },
            count,
        }
    }

    /// Opretter en ny fokuscyklus med et specifikt initialt fokus.
    pub fn with_initial(count: usize, initial: Option<usize>) -> Self {
        let valid_initial = match initial {
            Some(i) if i < count => Some(i),
            _ => None,
        };
        Self {
            current: valid_initial,
            count,
        }
    }

    /// Returnerer indekset for det aktuelt fokuserede element, hvis noget er fokuseret.
    pub fn current(&self) -> Option<usize> {
        self.current
    }

    /// Sætter eksplicit det fokuserede indeks.
    pub fn set_current(&mut self, index: Option<usize>) {
        if let Some(i) = index {
            if i < self.count {
                self.current = Some(i);
                return;
            }
        }
        self.current = None;
    }

    /// Returnerer sandt hvis det givne indeks aktuelt er fokuseret.
    pub fn is_focused(&self, index: usize) -> bool {
        self.current == Some(index)
    }

    /// Rykker fokus til næste element (Tab). Looper rundt til starten hvis bunden nås.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<usize> {
        if self.count == 0 {
            self.current = None;
            return None;
        }
        self.current = match self.current {
            None => Some(0),
            Some(idx) => {
                if idx + 1 < self.count {
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
        if self.count == 0 {
            self.current = None;
            return None;
        }
        self.current = match self.current {
            None => Some(self.count.saturating_sub(1)),
            Some(0) => Some(self.count.saturating_sub(1)),
            Some(idx) => Some(idx - 1),
        };
        self.current
    }

    /// Nulstiller fokus, så intet knap-element har fokus.
    pub fn reset(&mut self) {
        self.current = None;
    }

    /// Returnerer det samlede antal elementer i cyklussen.
    pub fn count(&self) -> usize {
        self.count
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
}
