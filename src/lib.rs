use chrono::{NaiveTime, Timelike};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::Read;
use std::time::Duration;

/// # Summary
///
/// プレイリスト
///
/// # Fields
///
/// - `base_path`: 再生対象ファイルのベースパス
/// - `tracks`: 再生対象トラックのリスト
/// - `repeat`: リピート再生するかどうか
/// - `volume`: ボリューム(1 を 100% とした数値)
#[derive(Debug, Deserialize, Serialize)]
pub struct Playlist {
    base_path: String,
    tracks: Vec<Track>,
    repeat: bool,
    volume: f64,
}

impl Playlist {
    pub fn base_path(&self) -> &str {
        &self.base_path
    }

    pub fn tracks(&self) -> &Vec<Track> {
        &self.tracks
    }

    pub fn repeat(&self) -> bool {
        self.repeat
    }

    pub fn volume(&self) -> f64 {
        self.volume
    }
}

/// # Summary
///
/// トラックリスト
///
/// # Fields
///
/// - `file`: 再生対象ファイル
/// - `start_position`: 開始位置(時刻文字列)
/// - `rank`: ランク
/// - `playback_duration`: 再生時間(時刻文字列)
#[derive(Debug, Deserialize, Serialize)]
pub struct Track {
    file: String,
    start_position: String,
    rank: u64,
    playback_duration: String,
}

impl Track {
    pub fn file(&self) -> &str {
        &self.file
    }

    pub fn start_position(&self) -> Result<Duration, String> {
        let start_position = &self.start_position;
        Ok(Duration::from_secs(if start_position == "" {
            0
        } else {
            time_string_to_seconds(start_position)?
        }))
    }

    pub fn rank(&self) -> u64 {
        self.rank
    }

    pub fn playback_duration(&self) -> &str {
        &self.playback_duration
    }
}

/// # Summary
///
/// プレイリストファイルを読み込み、プレイリストデータを返す
///
/// # Arguments
///
/// - `file`: プレイリストファイル
///
/// # Returns
///
/// - `Ok(Playlist)`: プレイリストデータ
/// - `Err(String)`: エラーメッセージ
///
/// # Errors
///
/// - プレイリストファイルのオープンに失敗した場合
/// - プレイリストファイルのリードに失敗した場合
/// - プレイリストデータをデシリアライズできなかった場合
///
/// # Examples
///
/// ```
/// use mp3player::get_playlist;
/// use std::time::Duration;
///
/// let result = get_playlist("tests/assets/playlist.json".to_string());
/// assert!(result.is_ok());
///
/// let playlist = result.unwrap();
/// assert_eq!(playlist.base_path(), "/foo");
/// assert_eq!(playlist.repeat(), false);
/// assert_eq!(playlist.volume(), 0.1);
///
/// let tracks = playlist.tracks();
/// let track = &tracks[0];
/// assert_eq!(track.file(), "assets/tracks/MusMus-BGM-136.mp3");
/// let result = track.start_position();
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap(), Duration::from_secs(140));
/// assert_eq!(track.rank(), 1);
/// assert_eq!(track.playback_duration(), "00:00:10");
///
/// let track = &tracks[1];
/// assert_eq!(track.file(), "assets/tracks/MusMus-BGM-162.mp3");
/// let result = track.start_position();
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap(), Duration::from_secs(150));
/// assert_eq!(track.rank(), 2);
/// assert_eq!(track.playback_duration(), "00:00:20");
///
/// let result = get_playlist("nonexistent_file.json".to_string());
/// assert!(result.is_err());
/// let error_message = result.unwrap_err();
/// assert_eq!(error_message, "Failed to open file: file -> \"nonexistent_file.json\", e -> Os { code: 2, kind: NotFound, message: \"No such file or directory\" }");
///
/// // read_to_string() のテストはなし
/// // オープンできるがリードできないテストファイルを用意できないため
///
/// let result = get_playlist("tests/assets/dummy.txt".to_string());
/// assert!(result.is_err());
/// let error_message = result.unwrap_err();
/// assert_eq!(error_message, "Failed to deserialize json data: contents -> \"dummy\\n\", e -> Error(\"expected value\", line: 1, column: 1)");
/// ```
pub fn get_playlist(file: String) -> Result<Playlist, String> {
    File::open(&file)
        .map_err(|e| format!("Failed to open file: file -> {file:?}, e -> {e:?}"))
        .and_then(|mut f| {
            let mut contents = String::new();
            f.read_to_string(&mut contents)
                .map_err(|e| format!("Failed to read file: file -> {file:?}, e -> {e:?}"))
                .map(|_| contents)
        })
        .and_then(|contents| {
            serde_json::from_str::<Playlist>(&contents).map_err(|e| {
                format!("Failed to deserialize json data: contents -> {contents:?}, e -> {e:?}")
            })
        })
}

/// # Summary
///
/// 時刻文字列を秒数に変換する
///
/// # Arguments
///
/// - `time_string`: 時刻文字列(形式: `HH:MM:SS`)
///
/// # Returns
///
/// - `Ok(u64)`: 秒数
/// - `Err(String)`: エラーメッセージ
///
/// # Errors
///
/// - 時刻文字列のパースに失敗した場合
///
/// # Examples
///
/// ```
/// use mp3player::time_string_to_seconds;
///
/// let result = time_string_to_seconds("01:23:45");
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap(), 5025);
///
/// let result = time_string_to_seconds("99:99:99");
/// assert!(result.is_err());
/// assert_eq!(result.unwrap_err(), "Failed to parse time-formatted string: time_string -> \"99:99:99\", e -> ParseError(OutOfRange)");
/// ```
pub fn time_string_to_seconds(time_string: &str) -> Result<u64, String> {
    NaiveTime::parse_from_str(time_string, "%H:%M:%S")
        .map_err(|e| {
            format!(
                "Failed to parse time-formatted string: time_string -> {time_string:?}, e -> {e:?}"
            )
        })
        .map(|parsed_time| parsed_time.num_seconds_from_midnight() as u64)
}
