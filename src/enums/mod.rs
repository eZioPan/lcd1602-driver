//! common enums

/// basic command related enums
pub mod basic_command {
    #[derive(Clone, Copy, PartialEq, Default)]
    pub enum MoveDirection {
        RightToLeft,
        #[default]
        LeftToRight,
    }

    #[derive(Clone, Copy, Default)]
    pub enum ShiftType {
        #[default]
        CursorOnly,
        CursorAndDisplay,
    }

    #[derive(Clone, Copy, PartialEq, Default)]
    pub enum State {
        Off,
        #[default]
        On,
    }

    #[derive(Clone, Copy, Default)]
    pub enum DataWidth {
        #[default]
        Bit4,
        #[allow(dead_code)]
        Bit8,
    }

    #[derive(Clone, Copy, Default, PartialEq)]
    pub enum LineMode {
        OneLine,
        #[default]
        TwoLine,
    }

    #[derive(Clone, Copy, Default, PartialEq)]
    pub enum Font {
        #[default]
        Font5x8,
        Font5x11,
    }

    /// The type of memory to access
    #[derive(Clone, Copy, PartialEq)]
    pub enum RAMType {
        /// Display Data RAM
        DDRAM,
        /// Character Generator RAM
        CGRAM,
    }
}

/// animation related enums
pub mod animation {
    /// The style of the offset display window
    pub enum MoveStyle {
        /// Always move to left
        ForceMoveLeft,
        /// Always move to right
        ForceMoveRight,
        /// Top left of display window won't cross display boundary
        NoCrossBoundary,
        /// Automatic find the shortest path
        Shortest,
    }

    /// The flip style of split flap display
    pub enum FlipStyle {
        /// Flip first character to target character, then flip next one
        Sequential,
        /// Flip all characters at once, automatically stop when the characters reach the target one
        Simultaneous,
    }
}
