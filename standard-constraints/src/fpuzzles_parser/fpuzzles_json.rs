use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FPuzzlesBoard {
    #[serde(default = "default_size")]
    pub size: i32,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub ruleset: String,
    #[serde(default)]
    pub grid: Vec<Vec<FPuzzlesGridEntry>>,
    #[serde(rename = "diagonal+", default = "default_false")]
    pub diagonal_p: bool,
    #[serde(rename = "diagonal-", default = "default_false")]
    pub diagonal_n: bool,
    #[serde(default = "default_false")]
    pub antiknight: bool,
    #[serde(default = "default_false")]
    pub disjointgroups: bool,
    #[serde(default = "default_false")]
    pub nonconsecutive: bool,
    #[serde(default)]
    pub negative: Vec<String>,
    #[serde(default)]
    pub arrow: Vec<FPuzzlesArrowEntry>,
    #[serde(default)]
    pub killercage: Vec<FPuzzlesKillerCageEntry>,
    #[serde(default)]
    pub cage: Vec<FPuzzlesKillerCageEntry>,
    #[serde(default)]
    pub littlekillersum: Vec<FPuzzlesLittleKillerSumEntry>,
    #[serde(default)]
    pub odd: Vec<FPuzzlesCell>,
    #[serde(default)]
    pub even: Vec<FPuzzlesCell>,
    #[serde(default)]
    pub minimum: Vec<FPuzzlesCell>,
    #[serde(default)]
    pub maximum: Vec<FPuzzlesCell>,
    #[serde(default)]
    pub rowindexer: Vec<FPuzzlesCells>,
    #[serde(default)]
    pub columnindexer: Vec<FPuzzlesCells>,
    #[serde(default)]
    pub boxindexer: Vec<FPuzzlesCells>,
    #[serde(default)]
    pub extraregion: Vec<FPuzzlesCells>,
    #[serde(default)]
    pub thermometer: Vec<FPuzzlesLines>,
    #[serde(default)]
    pub palindrome: Vec<FPuzzlesLines>,
    #[serde(default)]
    pub renban: Vec<FPuzzlesLines>,
    #[serde(default)]
    pub whispers: Vec<FPuzzlesLines>,
    #[serde(default)]
    pub regionsumline: Vec<FPuzzlesLines>,
    #[serde(default)]
    pub difference: Vec<FPuzzlesCells>,
    #[serde(default)]
    pub xv: Vec<FPuzzlesCells>,
    #[serde(default)]
    pub ratio: Vec<FPuzzlesCells>,
    #[serde(default)]
    pub clone: Vec<FPuzzlesClone>,
    #[serde(default)]
    pub quadruple: Vec<FPuzzlesQuadruple>,
    #[serde(default)]
    pub betweenline: Vec<FPuzzlesLines>,
    #[serde(default)]
    pub sandwichsum: Vec<FPuzzlesCell>,
    #[serde(default)]
    pub xsum: Vec<FPuzzlesCell>,
    #[serde(default)]
    pub skyscraper: Vec<FPuzzlesCell>,
    #[serde(default)]
    pub entropicline: Vec<FPuzzlesLines>,
    #[serde(default)]
    pub disabledlogic: Vec<String>,
    #[serde(default)]
    pub truecandidatesoptions: Vec<String>,
}

impl FPuzzlesBoard {
    pub fn from_lzstring_json(lz_str: &str) -> Result<FPuzzlesBoard, String> {
        let decompressed = lz_str::decompress_from_base64(lz_str);
        if decompressed.is_none() {
            return Err("Failed to decompress string".to_owned());
        }
        let decompressed = decompressed.unwrap();

        let decompressed_str = String::from_utf16(&decompressed);
        if let Err(error) = decompressed_str {
            return Err(format!(
                "Failed to convert decompressed string to UTF-16: {}",
                error
            ));
        }
        let decompressed_str = decompressed_str.unwrap();

        let board = Self::from_json(&decompressed_str);
        if let Err(error) = board {
            return Err(format!("Failed to parse JSON: {}", error));
        }
        let board = board.unwrap();

        Ok(board)
    }

    pub fn from_json(json: &str) -> Result<FPuzzlesBoard, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FPuzzlesGridEntry {
    #[serde(default = "default_zero")]
    pub value: i32,
    #[serde(default = "default_false")]
    pub given: bool,
    #[serde(rename = "centerPencilMarks", default = "Vec::default")]
    pub center_pencil_marks: Vec<i32>,
    #[serde(
        rename = "givenPencilMarks",
        default = "Vec::default",
        deserialize_with = "deserialize_null_default"
    )]
    pub given_pencil_marks: Vec<i32>,
    #[serde(
        default = "default_neg1",
        deserialize_with = "deserialize_null_as_neg1"
    )]
    pub region: i32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FPuzzlesArrowEntry {
    #[serde(default)]
    pub lines: Vec<Vec<String>>,
    #[serde(default)]
    pub cells: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FPuzzlesKillerCageEntry {
    #[serde(default)]
    pub cells: Vec<String>,
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FPuzzlesLittleKillerSumEntry {
    #[serde(default)]
    pub cell: String,
    #[serde(default)]
    pub direction: String,
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FPuzzlesCell {
    #[serde(default)]
    pub cell: String,
    #[serde(default)]
    pub value: String,
}

impl std::fmt::Display for FPuzzlesCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.value, self.cell)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FPuzzlesCells {
    #[serde(default)]
    pub cells: Vec<String>,
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FPuzzlesLines {
    #[serde(default)]
    pub lines: Vec<Vec<String>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FPuzzlesClone {
    #[serde(default)]
    pub cells: Vec<String>,
    #[serde(rename = "cloneCells", default = "Vec::default")]
    pub clone_cells: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FPuzzlesQuadruple {
    #[serde(default)]
    pub cells: Vec<String>,
    #[serde(default)]
    pub values: Vec<i32>,
}

fn default_size() -> i32 {
    9
}

fn default_zero() -> i32 {
    0
}

fn default_neg1() -> i32 {
    -1
}

fn default_false() -> bool {
    false
}

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

fn deserialize_null_as_neg1<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or(-1))
}

#[cfg(test)]
mod test {
    use super::super::fpuzzles_test_data::test::FPUZZLES_TEST_DATA;

    use super::*;

    #[test]
    fn test_no_decompress_errors() {
        for (i, data) in FPUZZLES_TEST_DATA.iter().enumerate() {
            let board = FPuzzlesBoard::from_lzstring_json(data.0);
            assert!(
                board.is_ok(),
                "Failed to parse board {}: {}",
                i,
                board.unwrap_err()
            );
        }
    }

    #[test]
    fn test_deserialize_clipped() {
        let board = FPuzzlesBoard::from_lzstring_json(FPUZZLES_TEST_DATA[0].0).unwrap();
        assert_eq!(board.size, 9);
        assert_eq!(board.title, "Clipped");
        assert_eq!(board.author, "Philipp Blume aka glum_hippo");
        assert_eq!(
            board.ruleset,
            r##"Sudoku - every row, column, and 3x3 box must contain all the digits from 1-9. Some digits are already given.

Anti-King - equal digits may not touch diagonally.

Thermometer - digits must strictly increase from bulb to tip.

Arrow - the circled digit constitutes the sum of the digits along the arrow.
"##
        );
        assert_eq!(board.grid.len(), 9);
        for i in 0..9 {
            assert_eq!(board.grid[i].len(), 9);
        }
        assert!(board.grid[2][4].given);
        assert_eq!(board.grid[2][4].value, 6);
        assert_eq!(board.arrow.len(), 5);
        assert_eq!(board.thermometer.len(), 4);
    }
}
