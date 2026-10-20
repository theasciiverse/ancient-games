//! Who plays a side, and how hard the computer plays.

/// How hard the computer plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strength {
    /// A beginner: advances and takes pieces, never thinks about danger.
    Easy,
    /// Weighs progress, safety and danger for the move at hand.
    Normal,
    /// Also looks at the throws and replies of the next two turns.
    Hard,
}

impl Strength {
    pub fn name(self) -> &'static str {
        match self {
            Strength::Easy => "easy",
            Strength::Normal => "normal",
            Strength::Hard => "hard",
        }
    }
}

/// Who plays a side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Player {
    Human,
    Computer(Strength),
}

