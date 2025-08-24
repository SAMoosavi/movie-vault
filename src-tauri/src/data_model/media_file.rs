use diesel::{
    backend::Backend,
    deserialize::FromSql,
    serialize,
    serialize::{Output, ToSql},
    sql_types::Text,
    sqlite::Sqlite,
};
use regex::Regex;
use std::path::PathBuf;

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    Ord,
    PartialOrd,
    serde::Serialize,
    serde::Deserialize,
    diesel::AsExpression,
    diesel::FromSqlRow,
)]
#[diesel(sql_type = Text)]
pub enum LanguageFormat {
    SoftSub,
    HardSub,
    Dubbed,
    Unknown,
}

impl std::fmt::Display for LanguageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let language_format = self.as_str();
        write!(f, "{language_format}")
    }
}

impl From<&String> for LanguageFormat {
    fn from(input: &String) -> Self {
        Self::from(input.as_str())
    }
}

impl From<&str> for LanguageFormat {
    fn from(input: &str) -> Self {
        if Self::detect_dubbed(input) {
            Self::Dubbed
        } else if Self::detect_hard_sub(input) {
            Self::HardSub
        } else if Self::detect_soft_sub(input) {
            Self::SoftSub
        } else {
            Self::Unknown
        }
    }
}

impl ToSql<Text, Sqlite> for LanguageFormat {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.as_str());
        Ok(diesel::serialize::IsNull::No)
    }
}

impl FromSql<Text, Sqlite> for LanguageFormat {
    fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let s = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        Ok(LanguageFormat::from(s.as_str()))
    }
}

impl Default for LanguageFormat {
    fn default() -> Self {
        Self::Unknown
    }
}

impl LanguageFormat {
    pub fn as_str(&self) -> &str {
        match &self {
            LanguageFormat::SoftSub => "soft_sub",
            LanguageFormat::HardSub => "hard_sub",
            LanguageFormat::Dubbed => "dubbed",
            LanguageFormat::Unknown => "",
        }
    }

    fn detect_dubbed(input: &str) -> bool {
        if input.contains("sub") || input.contains("subtitle") {
            return false;
        }

        let re = Regex::new(r"(?i)\b(dub|dubbed|farsi)\b").unwrap();
        re.is_match(input)
    }

    fn detect_hard_sub(input: &str) -> bool {
        let re = Regex::new(r"(?i)\bhard\s?(hardsub|sub|subtitle)\b").unwrap();
        re.is_match(input)
    }

    fn detect_soft_sub(input: &str) -> bool {
        let re = Regex::new(r"(?i)\b(softsub|sub|subtitle)\b").unwrap();
        re.is_match(input)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MediaFile {
    pub id: i32,
    pub file_name: String,
    pub path: String,
    pub quality: Option<String>,
    pub language_format: LanguageFormat,
}

impl Ord for MediaFile {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.file_name
            .cmp(&other.file_name)
            .then_with(|| self.path.cmp(&other.path))
            .then_with(|| self.quality.cmp(&other.quality))
            .then_with(|| self.language_format.cmp(&other.language_format))
    }
}

impl PartialOrd for MediaFile {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for MediaFile {
    fn eq(&self, other: &Self) -> bool {
        self.file_name == other.file_name
            && self.path == other.path
            && self.quality == other.quality
            && self.language_format == other.language_format
    }
}

impl Eq for MediaFile {}

impl From<PathBuf> for MediaFile {
    fn from(path: PathBuf) -> Self {
        let video_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        let normalized = video_stem.to_lowercase();

        Self {
            id: 0,
            file_name: video_stem.into(),
            path: path.to_str().unwrap().to_string(),
            quality: Self::detect_quality(&normalized),
            language_format: LanguageFormat::from(&normalized),
        }
    }
}

impl MediaFile {
    fn detect_quality(input: &str) -> Option<String> {
        // Case-insensitive regex for common quality tags
        let re = Regex::new(r"(?i)\b(4k|2160p|1080p|720p|480p|hd|hq)\b").unwrap();

        re.find(input)
            .map(|m| match m.as_str().to_lowercase().as_str() {
                "hd" | "hq" => "720p".into(),
                other => other.into(),
            })
    }
}

#[cfg(test)]
impl MediaFile {
    pub fn generate_random_file(random_number: u8) -> Self {
        let random_number = if random_number > 4 {
            0
        } else {
            ((random_number as i8) - 1) as usize
        };

        let random_path: Vec<PathBuf> = vec![
            "/path/to/movie.1080p.dubbed.mkv".into(),
            "/path/to/Movie.1080P.HARDSUB.MKV".into(),
            "/path/to/Movie.1080P.SUFTSUB.MKV".into(),
            "/path/to/Movie.720P.HARDSUB.MKV".into(),
        ];
        Self::from(random_path[random_number].clone())
    }
}

#[cfg(test)]
mod tests_detect_language_format {
    use super::*;

    #[test]
    fn test_detect_hard_sub_cases() {
        let positives = [
            "movie hardsub mkv",
            "hard sub release",
            "hard sub version",
            "this is hardsub",
        ];

        for case in positives {
            assert!(
                LanguageFormat::detect_hard_sub(case),
                "Expected detect_hard_sub to return true for {case:?}"
            );
        }

        let negatives = ["softsub", "subtitle", "audio.hardtrack"];
        for case in negatives {
            assert!(
                !LanguageFormat::detect_hard_sub(case),
                "Expected detect_hard_sub to return false for {case:?}"
            );
        }
    }
    #[test]
    fn test_detect_soft_sub_cases() {
        let positives = [
            "movie softsub mkv",
            "soft sub release",
            "soft sub version",
            "this is softsub",
            "soft subtitle",
            "subtitle",
        ];

        for case in positives {
            assert!(
                LanguageFormat::detect_soft_sub(case),
                "Expected detect_soft_sub to return true for {case:?}"
            );
        }

        let negatives = ["hardsub", "audio.hardtrack"];
        for case in negatives {
            assert!(
                !LanguageFormat::detect_soft_sub(case),
                "Expected detect_hard_sub to return false for {case:?}"
            );
        }
    }

    #[test]
    fn test_detect_dubbed_positive() {
        let positives = [
            "movie.dub.mkv",
            "Farsi dubbed version",
            "farsi audio",
            "official DUB release",
            "dubbed film",
        ];

        for input in positives {
            assert!(
                LanguageFormat::detect_dubbed(input),
                "Expected detect_dubbed to return true for {input:?}"
            );
        }
    }

    #[test]
    fn test_detect_dubbed_negative() {
        let negatives = [
            "dubious story",
            "redubbed version",
            "no subtitles",
            "audio track",
            "farsight analysis",
            "secret invasion s01e01 720p web-dl farsi sub",
        ];

        for input in negatives {
            assert!(
                !LanguageFormat::detect_dubbed(input),
                "Expected detect_dubbed to return false for {input:?}"
            );
        }
    }
}

#[cfg(test)]
mod tests_derives_of_language_format {
    use super::*;

    #[test]
    fn test_display_soft_sub() {
        let lang = LanguageFormat::SoftSub;
        assert_eq!(lang.to_string(), "soft_sub");
    }

    #[test]
    fn test_display_hard_sub() {
        let lang = LanguageFormat::HardSub;
        assert_eq!(lang.to_string(), "hard_sub");
    }

    #[test]
    fn test_display_dubbed() {
        let lang = LanguageFormat::Dubbed;
        assert_eq!(lang.to_string(), "dubbed");
    }

    #[test]
    fn test_display_unknown() {
        let lang = LanguageFormat::Unknown;
        assert_eq!(lang.to_string(), "");
    }

    #[test]
    fn test_partial_eq_and_eq() {
        assert_eq!(LanguageFormat::SoftSub, LanguageFormat::SoftSub);
        assert_ne!(LanguageFormat::SoftSub, LanguageFormat::HardSub);
        assert_eq!(LanguageFormat::Unknown, LanguageFormat::Unknown);
    }

    #[test]
    fn test_clone() {
        let lang = LanguageFormat::Dubbed;
        let cloned = lang.clone();
        assert_eq!(lang, cloned);
    }
}

#[cfg(test)]
mod tests_from_of_language_format {
    use super::*;

    #[test]
    fn test_from_dubbed() {
        let input = "movie.dubbed.eng.mkv";
        assert_eq!(LanguageFormat::from(input), LanguageFormat::Dubbed);
    }

    #[test]
    fn test_from_hard_sub() {
        let input = "movie.hardsub.eng.mkv";
        assert_eq!(LanguageFormat::from(input), LanguageFormat::HardSub);
    }

    #[test]
    fn test_from_soft_sub() {
        let input = "movie.softsub.eng.mkv";
        assert_eq!(LanguageFormat::from(input), LanguageFormat::SoftSub);
    }

    #[test]
    fn test_from_unknown() {
        let input = "movie.eng.mkv";
        assert_eq!(LanguageFormat::from(input), LanguageFormat::Unknown);
    }

    #[test]
    fn test_from_empty_string() {
        let input = "";
        assert_eq!(LanguageFormat::from(input), LanguageFormat::Unknown);
    }

    #[test]
    fn test_case_insensitivity() {
        let input = "Movie.farsi.sub.MKV";
        assert_eq!(LanguageFormat::from(input), LanguageFormat::SoftSub);
    }
}

#[cfg(test)]
mod tests_detect_quality {
    use super::*;

    #[test]
    fn test_detect_quality_matches() {
        let cases = vec![
            ("Movie.1080p.mkv", Some("1080p".into())),
            ("Show.4K.UltraHD", Some("4k".into())),
            ("Clip.HD.version", Some("720p".into())),
            ("Video.hq.release", Some("720p".into())),
            ("OldMovie.480p.avi", Some("480p".into())),
            ("UnknownQuality.mkv", None),
        ];

        for (input, expected) in cases {
            assert_eq!(
                MediaFile::detect_quality(input),
                expected,
                "Failed on input: {input:?}"
            );
        }
    }
}

#[cfg(test)]
mod tests_file_from_path_buf {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn from_valid_path_with_quality_and_language() {
        let path = PathBuf::from("/path/to/movie.1080p.dubbed.mkv");
        let file = MediaFile::from(path.clone());

        assert_eq!(file.id, 0);
        assert_eq!(file.file_name, "movie.1080p.dubbed");
        assert_eq!(file.path, "/path/to/movie.1080p.dubbed.mkv");
        assert_eq!(file.quality, Some("1080p".to_string()));
        assert_eq!(file.language_format, LanguageFormat::Dubbed);
    }

    #[test]
    fn from_path_with_uppercase() {
        let path = PathBuf::from("/path/to/Movie.1080P.HARDSUB.MKV");
        let file = MediaFile::from(path.clone());

        assert_eq!(file.id, 0);
        assert_eq!(file.file_name, "Movie.1080P.HARDSUB");
        assert_eq!(file.path, "/path/to/Movie.1080P.HARDSUB.MKV");
        assert_eq!(file.quality, Some("1080p".to_string()));
        assert_eq!(file.language_format, LanguageFormat::HardSub);
    }

    #[test]
    fn from_path_no_quality_no_language() {
        let path = PathBuf::from("/path/to/movie.mkv");
        let file = MediaFile::from(path.clone());

        assert_eq!(file.id, 0);
        assert_eq!(file.file_name, "movie");
        assert_eq!(file.path, "/path/to/movie.mkv");
        assert_eq!(file.quality, None);
        assert_eq!(file.language_format, LanguageFormat::Unknown);
    }

    #[test]
    fn from_path_no_file_stem() {
        let path = PathBuf::from("/");
        let file = MediaFile::from(path.clone());

        assert_eq!(file.id, 0);
        assert_eq!(file.file_name, "");
        assert_eq!(file.path, "/");
        assert_eq!(file.quality, None);
        assert_eq!(file.language_format, LanguageFormat::Unknown);
    }

    #[test]
    fn from_path_with_extension_only() {
        let path = PathBuf::from("/path/to/.hidden.mkv");
        let file = MediaFile::from(path.clone());

        assert_eq!(file.id, 0);
        assert_eq!(file.file_name, ".hidden");
        assert_eq!(file.path, "/path/to/.hidden.mkv");
        assert_eq!(file.quality, None);
        assert_eq!(file.language_format, LanguageFormat::Unknown);
    }
}
