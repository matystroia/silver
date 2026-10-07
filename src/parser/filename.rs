use std::{
    collections::{HashMap, VecDeque},
    path::{Component, Path},
    sync::LazyLock,
};

use chrono::Datelike;
use itertools::Itertools;
use regex::Regex;

#[derive(Debug, PartialEq, Eq)]
pub enum Filename {
    Movie(MovieDetails),
    Episode(EpisodeDetails),
}

#[derive(Debug, PartialEq, Eq)]
pub struct MovieDetails {
    pub title: String,
    pub year: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub struct EpisodeDetails {
    pub series: String,
    pub season: u16,
    pub episode: u16,
}

static CUR_YEAR: LazyLock<u16> = LazyLock::new(|| chrono::Utc::now().year() as u16);

static EP_S1E1_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^\(?s(\d+)e(\d+)\)?$").unwrap());
static EP_1X1_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^(\d+)x(\d+)$").unwrap());
static EP_E1_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^e(\d+)$").unwrap());

static SEASON_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(?:season|series)$").unwrap());
static EPISODE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^ep(?:isode|\.)?$").unwrap());

static EP_NUMBER_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)\.").unwrap());
static EP_NUMBER_V_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)v[1-9]$").unwrap());

static TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\[[^\[\]]+\]|\[[^\[\]]+\]$").unwrap());
static GROUP_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"-+[a-zA-Z0-9]+$").unwrap());

static RES_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(?:\d{3,4}[pi]|4k|uhd|hd)\b").unwrap());

static SOURCE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)\b(?:web[ ._-]?dl|web[ ._-]?rip|webrip|web|bd[ ._-]?rip|br[ ._-]?rip|",
        r"bd[ ._-]?remux|remux|blu[ ._-]?ray|bluray|hd[ ._-]?dvd|dvd[ ._-]?rip|",
        r"dvd[ ._-]?scr|dvdscr|dvd|hdtv|pdtv|sdtv|hdrip|hd[ ._-]?cam|cam[ ._-]?rip|",
        r"ppv|dvb|vhs|vod|workprint|hdr10\+?|hdr|dolby[ ._-]?vision|hlg|sdr|nf|it|",
        r"itunes|hmax|atvp|hulu|amzn)\b",
    ))
    .unwrap()
});

static CODEC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)\b(?:[xh][ ._-]?26[45]|avc|hevc|xvid|divx|vc[ ._-]?1|",
        r"mpeg[ ._-]?2|vp9|av1|10[ ._-]?bit|8[ ._-]?bit)\b",
    ))
    .unwrap()
});

static AUDIO_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)\b(?:ddp?\+?[ ._-]?\d(?:[ ._-]?\d)?|dts[ ._-]?hd[ ._-]?ma|dts[ ._-]?hd|",
        r"dts|true[ ._-]?hd|truehd|atmos|e?ac[ ._-]?3|aac(?:[ ._-]?lc)?|flac|opus|",
        r"mp3|l?pcm|vorbis|\d[ ._-]?\d[ ._-]?ch|[2567][ ._-][01])\b",
    ))
    .unwrap()
});

static EDITION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)\b(?:extended(?:[ ._-]?cut)?|directors?(?:[ ._-]?cut)?|unrated|uncut|",
        r"remastered|criterion|imax|theatrical|special[ ._-]?edition|limited|",
        r"repack|proper|internal|complete|hybrid|restored|rough[ ._-]cut)\b",
    ))
    .unwrap()
});

static MISC_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\b(?:\w{2}subbed)\b").unwrap());

static SEP_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[._-]").unwrap());

static DIR_SEASON_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:(.+) )?(?:season|series) (\d+)$").unwrap());
static DIR_SEASONS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:(.+) )(?:seasons|series)").unwrap());

impl Filename {
    pub fn new(path: &Path) -> Option<Self> {
        let filename = path.file_stem().unwrap().to_string_lossy().into_owned();

        let mut words = Self::collect_words(&filename);

        let slice = words.make_contiguous();
        if let ret @ Some(_) = Self::try_movie(slice) {
            return ret;
        }
        Self::try_episode(slice, path)
    }

    fn collect_words(filename: &str) -> VecDeque<String> {
        // Count separators before stripping metadata
        let mut char_count: HashMap<char, usize> = HashMap::new();
        filename.chars().for_each(|ch| {
            char_count.entry(ch).and_modify(|c| *c += 1).or_insert(1);
        });

        let filename = Self::strip_meta(filename);

        // Split by most common separator
        let sep = ['.', '_', ' ']
            .into_iter()
            .max_by_key(|sep| char_count.get(sep).unwrap_or(&0))
            .unwrap();

        filename
            .split(sep)
            .filter(|w| !w.is_empty())
            .map(str::to_lowercase)
            .collect()
    }

    fn strip_meta(filename: &str) -> String {
        let mut filename = filename.to_string();

        // Group (last)
        if let Some(m) = GROUP_RE.find(&filename) {
            filename.drain(m.range());
        }

        // [Tags] (first or last)
        while TAG_RE.is_match(&filename) {
            filename = TAG_RE.replace_all(&filename, "").into_owned();
        }

        loop {
            let mut found = false;
            for re in [
                &RES_RE,
                &SOURCE_RE,
                &CODEC_RE,
                &AUDIO_RE,
                &EDITION_RE,
                &MISC_RE,
                &SEP_RE,
            ] {
                if let Some(m) = re.find_iter(&filename).last()
                    && m.range().end == filename.len()
                {
                    filename.drain(m.range());
                    found = true;
                }
            }
            if !found {
                break;
            }
        }

        filename
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Movie(movie) => &movie.title,
            Self::Episode(episode) => &episode.series,
        }
    }

    /// Year in title
    fn try_movie(words: &[String]) -> Option<Self> {
        // Last year in case of movies with year in title
        let (year_i, year) = words.iter().enumerate().rev().find_map(|(i, w)| {
            w.parse()
                .ok()
                .and_then(|y| (1930..=*CUR_YEAR).contains(&y).then_some((i, y)))
        })?;

        if year_i == 0 {
            return None;
        }

        let title = words[..year_i].join(" ");

        // Overfitting - don't care :)
        if title.ends_with(" -") {
            return None;
        }

        Some(Filename::Movie(MovieDetails { title, year }))
    }

    fn try_episode(words: &[String], path: &Path) -> Option<Self> {
        let mut words = words.to_vec();
        let mut parts = EpisodeParts::default();

        for extract in [
            Self::extract_sxex,
            Self::extract_season_episode,
            Self::extract_exx,
            Self::extract_episode,
            Self::extract_leading_number,
            Self::extract_dash_split,
            Self::extract_xxxx,
            Self::extract_single_number,
        ] {
            extract(&mut words, &mut parts);
            if parts.is_complete() {
                return parts.into_episode();
            }
        }

        Self::extract_dir_context(path, &mut parts);

        parts.into_episode()
    }

    /// Dashes used to separate components
    fn extract_dash_split(words: &mut Vec<String>, parts: &mut EpisodeParts) {
        let split = words.split(|w| w == "-").collect_vec();
        if let [left, right] = split.as_slice() {
            // 01 - Episode Title
            if let [word] = left
                && let Ok(left_num) = word.parse::<u16>()
            {
                parts.episode(left_num);
                words.clear();
            } else if let [word] = right
                && let Ok(right_num) = word.parse::<u16>()
            {
                parts.name(left.join(" "));
                parts.episode(right_num);
                words.clear();
            }
        } else if let [left, middle, _right] = split.as_slice() {
            parts.name(left.join(" "));
            let _ = std::mem::replace(words, middle.to_vec());
        }
    }

    /// 'S01E01' or '1x1'
    fn extract_sxex(words: &mut Vec<String>, parts: &mut EpisodeParts) {
        for re in [&EP_S1E1_RE, &EP_1X1_RE] {
            if let Some((i, caps)) = words
                .iter()
                .enumerate()
                .find_map(|(i, w)| re.captures(w).map(|c| (i, c)))
                && let Ok(season) = caps[1].parse()
                && let Ok(episode) = caps[2].parse()
            {
                parts.season(season);
                parts.episode(episode);
                if i > 0 {
                    parts.name(words[..i].join(" "));
                }
                words.drain(..i);
                return;
            };
        }
    }

    /// Season 1 Episode 1
    fn extract_season_episode(words: &mut Vec<String>, parts: &mut EpisodeParts) {
        for (i, window) in words.windows(4).enumerate() {
            if SEASON_RE.is_match(&window[0])
                && let Ok(season) = window[1].parse()
                && EPISODE_RE.is_match(&window[2])
                && let Ok(episode) = window[3].parse()
            {
                parts.season(season);
                parts.episode(episode);
                if i > 0 {
                    parts.name(words[..i].join(" "));
                }
                words.drain(..i + 4);
                return;
            }
        }
    }

    /// E01
    fn extract_exx(words: &mut Vec<String>, parts: &mut EpisodeParts) {
        if let Some((i, caps)) = words
            .iter()
            .enumerate()
            .find_map(|(i, w)| EP_E1_RE.captures(w).map(|c| (i, c)))
            && let Ok(episode) = caps[1].parse()
        {
            parts.episode(episode);
            if i > 0 {
                parts.name(words[..i].join(" "));
            }
            words.drain(..i);
        }
    }

    /// Episode 1
    fn extract_episode(words: &mut Vec<String>, parts: &mut EpisodeParts) {
        for (i, window) in words.windows(2).enumerate() {
            if EPISODE_RE.is_match(&window[0])
                && let Ok(episode) = window[1].parse()
            {
                parts.episode(episode);
                if i > 0 {
                    parts.name(words[..i].join(" "));
                }
                words.drain(..i + 2);
                return;
            }
        }
    }

    /// 01. Episode Title
    fn extract_leading_number(words: &mut Vec<String>, parts: &mut EpisodeParts) {
        if let Some(w) = words.first()
            && let Some(caps) = EP_NUMBER_RE.captures(w)
            && let Ok(episode) = caps[1].parse()
        {
            parts.episode(episode);
            words.clear();
        }
    }

    /// 101, 405, 1015, etc.
    fn extract_xxxx(words: &mut Vec<String>, parts: &mut EpisodeParts) {
        let numbers: Vec<(usize, u16)> = words
            .iter()
            .enumerate()
            .filter_map(|(i, w)| w.parse().ok().map(|w| (i, w)))
            .collect();
        if let [(i, num)] = *numbers.as_slice()
            && (101..=3999).contains(&num)
        {
            parts.season(num / 100);
            parts.episode(num % 100);
            parts.name(words[..i].join(" "));
            words.drain(..i);
        }
    }

    fn extract_single_number(words: &mut Vec<String>, parts: &mut EpisodeParts) {
        let numbers: Vec<(usize, u16)> = words
            .iter()
            .enumerate()
            .filter_map(|(i, w)| {
                w.parse().ok().map(|num| (i, num)).or(EP_NUMBER_V_RE
                    .captures(w)
                    .and_then(|caps| caps[1].parse().ok())
                    .map(|num| (i, num)))
            })
            .collect();
        if let [(i, num)] = *numbers.as_slice()
            && (1..=100).contains(&num)
        {
            parts.episode(num);
            parts.name(words[..i].join(" "));
            words.drain(..i);
        }
    }

    /// Desperately searching for data in ancestors
    fn extract_dir_context(path: &Path, parts: &mut EpisodeParts) {
        let comps: Vec<_> = path
            .parent()
            .unwrap()
            .components()
            .filter_map(|component| {
                if let Component::Normal(c) = component {
                    Some(c.to_string_lossy().to_lowercase())
                } else {
                    None
                }
            })
            .collect();

        if let Some(parent) = comps.last()
            && let Some(caps) = DIR_SEASON_RE.captures(parent)
            && let Ok(season) = caps[2].parse()
        {
            // Season 1 / ...
            parts.season(season);
            if let Some(cap) = caps.get(1) {
                // Show Season 1 / ...
                parts.name(cap.as_str().to_string());
            } else if let Some(grandparent) = comps.iter().rev().nth(1) {
                if let Some(caps) = DIR_SEASONS_RE.captures(grandparent) {
                    // Show Seasons 1-10 / Season 1 / ...
                    parts.name(caps[1].to_string())
                } else {
                    // Show / Season 1 / ...
                    parts.name(grandparent.clone())
                }
            }
        }
    }

    pub fn to_ost_query(&self) -> Vec<(&str, String)> {
        match self {
            Self::Movie(movie) => vec![
                ("type", "movie".to_string()),
                ("query", movie.title.clone()),
                ("year", movie.year.to_string()),
            ],
            Self::Episode(episode) => {
                vec![
                    ("type", "episode".to_string()),
                    ("query", episode.series.clone()),
                    ("episode_number", episode.episode.to_string()),
                    ("season_number", episode.season.to_string()),
                ]
            }
        }
    }

    pub fn to_tmdb_query(&self) -> Vec<(&str, String)> {
        match self {
            Self::Movie(movie) => vec![
                ("query", movie.title.to_string()),
                ("year", movie.year.to_string()),
            ],
            Self::Episode(episode) => vec![("query", episode.series.clone())],
        }
    }
}

#[derive(Default)]
pub struct EpisodeParts {
    name: Option<String>,
    season: Option<u16>,
    episode: Option<u16>,
}

impl EpisodeParts {
    fn name(&mut self, name: String) {
        if self.name.is_none() {
            self.name = Some(name.trim_suffix(" -").into());
        }
    }

    fn season(&mut self, season: u16) {
        if self.season.is_none() {
            self.season = Some(season);
        }
    }

    fn episode(&mut self, episode: u16) {
        if self.episode.is_none() {
            self.episode = Some(episode);
        }
    }

    fn is_complete(&self) -> bool {
        self.name.is_some() && self.season.is_some() && self.episode.is_some()
    }

    fn into_episode(self) -> Option<Filename> {
        if let Some(name) = self.name
            && let Some(episode) = self.episode
        {
            Some(Filename::Episode(EpisodeDetails {
                series: name,
                season: self.season.unwrap_or(1),
                episode,
            }))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(
        "28.Years.Later.2025.2160p.iT.WEB-DL.DDP5.1.Atmos.HDR.H.265-HONE",
        "28 years later",
        2025
    )]
    #[case("Barton.Fink.1991.1080p.BluRay.DD2.0.x264-DON", "barton fink", 1991)]
    #[case(
        "Beau.Is.Afraid.2023.1080p.BluRay.DD+5.1.x264-playHD",
        "beau is afraid",
        2023
    )]
    #[case(
        "Blood.Simple.1984.Directors.Cut.1080p.BluRay.DD5.1.x264.RoSubbed-DON",
        "blood simple",
        1984
    )]
    #[case(
        "Dressed.To.Kill.1980.Unrated.REPACK.1080p.BluRay.DDP.5.1.x264-c0kE",
        "dressed to kill",
        1980
    )]
    #[case(
        "I'm.Thinking.of.Ending.Things.2020.2160p.NF.WEB-DL.DD+5.1.Atmos.H.265-CADiLLAC",
        "i'm thinking of ending things",
        2020
    )]
    #[case(
        "Vampyr.1932.Criterion.1080p.BluRay.x265.HEVC.AAC-SARTRE",
        "vampyr",
        1932
    )]
    fn it_parses_movies(
        #[case] filename: &str,
        #[case] expected_title: &str,
        #[case] expected_year: u16,
    ) {
        let path = PathBuf::from("/home/media/").join(format!("{filename}.mkv"));
        assert_eq!(
            Filename::new(&path),
            Some(Filename::Movie(MovieDetails {
                title: expected_title.to_string(),
                year: expected_year,
            })),
        );
    }

    #[rstest]
    #[case(
        "28.Years.Later.2025.2160p.iT.WEB-DL.DDP5.1.Atmos.HDR.H.265-HONE",
        "28.Years.Later.2025"
    )]
    #[case("Barton.Fink.1991.1080p.BluRay.DD2.0.x264-DON", "Barton.Fink.1991")]
    #[case(
        "Beau.Is.Afraid.2023.1080p.BluRay.DD+5.1.x264-playHD",
        "Beau.Is.Afraid.2023"
    )]
    #[case(
        "Blood.Simple.1984.Directors.Cut.1080p.BluRay.DD5.1.x264.RoSubbed-DON",
        "Blood.Simple.1984"
    )]
    #[case("modern.family.s02e01.dvdrip.xvid-reward", "modern.family.s02e01")]
    #[case(
        "Seinfeld.S03E01.The.Note.DVDRip.x264-DiGG",
        "Seinfeld.S03E01.The.Note"
    )]
    #[case(
        "simpsons_-_4x01_-_kamp_krusty.dvd.fov.xvid",
        "simpsons_-_4x01_-_kamp_krusty"
    )]
    #[case(
        "Www.RachelOrmont.com.2024.ROUGH.CUT.1080p.WEBRip.x264.AAC-LAMA",
        "Www.RachelOrmont.com.2024"
    )]
    #[case("[TCL]_Soul_Eater_51_[Blu-Ray][1080p][CC41A8E8]", "Soul_Eater_51")]
    fn it_strips_metadata(#[case] filename: &str, #[case] expected_filename: &str) {
        assert_eq!(Filename::strip_meta(filename), expected_filename);
    }

    #[rstest]
    #[case(
        "Adventure.Time.S01E08.Business.Time.1080p.BluRay.Remux.VC-1.DD2.0.H.264-SA89",
        "adventure time",
        1,
        8
    )]
    #[case("modern.family.s02e01.dvdrip.xvid-reward", "modern family", 2, 1)]
    #[case("Seinfeld.S03E01.The.Note.DVDRip.x264-DiGG", "seinfeld", 3, 1)]
    #[case("Whose Line US - 2x01 (show 209)", "whose line us", 2, 1)]
    #[case("[HorribleSubs] Kill la Kill - 01 [1080p]", "kill la kill", 1, 1)]
    #[case(
        "[CBM]_Neon_Genesis_Evangelion_-_23_-_Tears_[720p]_[7B7BA511]",
        "neon genesis evangelion",
        1,
        23
    )]
    #[case("The.Simpsons.S01E01.DVDRip.XviD-SChiZO", "the simpsons", 1, 1)]
    #[case(
        "the.simpsons.03x01.stark.raving.dad--dvd.xvid--fov",
        "the simpsons",
        3,
        1
    )]
    #[case("simpsons_-_4x01_-_kamp_krusty.dvd.fov.xvid", "simpsons", 4, 1)]
    #[case("the.simpsons.712-med", "the simpsons", 7, 12)]
    #[case("Simpsons 13x02 - The Parent Rap", "simpsons", 13, 2)]
    #[case(
        "The Simpsons - 2001 - Sex, Pies and Idiot Scrapes",
        "the simpsons",
        20,
        1
    )]
    #[case("Inuyasha ep 156", "inuyasha", 1, 156)]
    #[case("Inuyasha ep 201", "inuyasha", 1, 201)]
    #[case(
        "Ghost in the Shell Stand Alone Complex 2nd GIG E05 'Inductance'",
        "ghost in the shell stand alone complex 2nd gig",
        1,
        5
    )]
    #[case("[TCL]_Soul_Eater_51_[Blu-Ray][1080p][CC41A8E8]", "soul eater", 1, 51)]
    #[case(
        "[SubsPlease] Wonder Egg Priority - 01v2 (1080p) [BEBD5590]",
        "wonder egg priority",
        1,
        1
    )]
    fn it_parses_episodes(
        #[case] filename: &str,
        #[case] expected_series: &str,
        #[case] expected_season: u16,
        #[case] expected_episode: u16,
    ) {
        assert_eq!(
            Filename::new(&PathBuf::from("/home/media/").join(format!("{filename}.mkv"))),
            Some(Filename::Episode(EpisodeDetails {
                series: expected_series.into(),
                season: expected_season,
                episode: expected_episode,
            }))
        )
    }

    #[rstest]
    #[case(
        "/Xavier Renegade Angel/Season 1/1. What Life D-D-Doth",
        "xavier renegade angel",
        1,
        1
    )]
    #[case("/Skins Seasons 1 - 5/Season 1/01 - Tony", "skins", 1, 1)]
    #[case(
        "/Skins Seasons 1 - 5/Series 5/Skins Series 5 Episode 5",
        "skins",
        5,
        5
    )]
    fn it_parses_episodes_in_context(
        #[case] path: &str,
        #[case] expected_name: &str,
        #[case] expected_season: u16,
        #[case] expected_episode: u16,
    ) {
        assert_eq!(
            Filename::new(&PathBuf::from(format!("{path}.mkv"))),
            Some(Filename::Episode(EpisodeDetails {
                series: expected_name.into(),
                season: expected_season,
                episode: expected_episode
            }))
        )
    }
}
