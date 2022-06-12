use crate::io::piston_io;
use clap::{ArgEnum, Parser};
use std::fs;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Cli {
    /// Path to a ROM to load
    #[clap(parse(try_from_str = open_file))]
    pub rom: fs::File,

    /// Color scheme for the display
    #[clap(short, long, arg_enum, default_value_t = ColorSchemeName::Jazz)]
    pub color_scheme: ColorSchemeName,
}

fn open_file(path: &str) -> Result<fs::File, String> {
    fs::File::open(path).map_err(|e| String::from(e.to_string()))
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
