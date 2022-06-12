use crate::io::piston_io;
use clap::{ArgEnum, Parser};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Cli {
    /// Path to a ROM to load
    pub rom_path: String,

    /// Color scheme for the display
    #[clap(short, long, arg_enum, default_value_t = ColorSchemeName::Jazz)]
    pub color_scheme: ColorSchemeName,
}

#[derive(ArgEnum, Clone, Debug)]
pub enum ColorSchemeName {
    BlackOnWhite,
    WhiteOnBlack,
    Jazz,
}

impl Into<piston_io::ColorScheme> for ColorSchemeName {
    fn into(self) -> piston_io::ColorScheme {
        match self {
            ColorSchemeName::BlackOnWhite => piston_io::BLACK_ON_WHITE,
            ColorSchemeName::WhiteOnBlack => piston_io::WHITE_ON_BLACK,
            ColorSchemeName::Jazz => piston_io::JAZZ_COLORS,
        }
    }
}
