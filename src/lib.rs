use chrono::{NaiveTime, Timelike};
use std::fs::File;
use std::io::Read;

/// # Summary
///
/// ファイルから JSON データを読み込み、デシリアライズして返す
///
/// # Arguments
///
/// - `file`: ファイル
///
/// # Returns
///
/// - `Ok(serde_json::Value)`: 任意の JSON データ
/// - `Err(String)`: エラーメッセージ
///
/// # Errors
///
/// - ファイルのオープンに失敗した場合
/// - ファイルのリードに失敗した場合
/// - JSON データをデシリアライズできなかった場合
///
/// # Examples
///
/// ```
/// use mp3player::read_json_data;
///
/// let result = read_json_data("tests/assets/playlist.json".to_string());
/// assert!(result.is_ok());
/// let json_data = result.unwrap();
/// assert_eq!(json_data["playlist"][0]["file"], "assets/tracks/MusMus-BGM-136.mp3");
/// assert_eq!(json_data["playlist"][0]["position"], "00:02:20");
/// assert_eq!(json_data["playlist"][1]["file"], "assets/tracks/MusMus-BGM-162.mp3");
/// assert_eq!(json_data["playlist"][1]["position"], "00:02:30");
/// assert_eq!(json_data["volume"], 0.1);
///
/// let result = read_json_data("nonexistent_file.json".to_string());
/// assert!(result.is_err());
/// let error_message = result.unwrap_err();
/// assert_eq!(error_message, "Failed to open file: file -> \"nonexistent_file.json\", e -> Os { code: 2, kind: NotFound, message: \"No such file or directory\" }");
///
/// // read_to_string() のテストはなし
/// // オープンできるがリードできないテストファイルを用意できないため
///
/// let result = read_json_data("tests/assets/dummy.txt".to_string());
/// assert!(result.is_err());
/// let error_message = result.unwrap_err();
/// assert_eq!(error_message, "Failed to deserialize json data: contents -> \"dummy\\n\", e -> Error(\"expected value\", line: 1, column: 1)");
/// ```
pub fn read_json_data(file: String) -> Result<serde_json::Value, String> {
    File::open(&file)
        .map_err(|e| format!("Failed to open file: file -> {file:?}, e -> {e:?}"))
        .and_then(|mut f| {
            let mut contents = String::new();
            f.read_to_string(&mut contents)
                .map_err(|e| format!("Failed to read file: file -> {file:?}, e -> {e:?}"))
                .map(|_| contents)
        })
        .and_then(|contents| {
            serde_json::from_str::<serde_json::Value>(&contents).map_err(|e| {
                format!("Failed to deserialize json data: contents -> {contents:?}, e -> {e:?}")
            })
        })
}

/// # Summary
///
/// プレイリストの JSON データを再帰的にパースして、その結果を各変数に設定する
///
/// # Arguments
///
/// - `key`: JSON データのキー
/// - `value`: JSON データのバリュー
///
/// 以降の引数はパースした結果
///
/// - `base_path`: 再生対象ファイルのベースパス
/// - `files`: 再生対象ファイル
/// - `positions`: シーク位置(時刻文字列)
/// - `repeat`: リピート再生するかどうか
/// - `takes`: 再生時間(時刻文字列)
/// - `volume`: ボリューム(1 を 100% とした数値)
pub fn parse_json_data(
    key: &str,
    value: serde_json::Value,
    base_path: &mut String,
    files: &mut Vec<String>,
    positions: &mut Vec<String>,
    repeat: &mut bool,
    takes: &mut Vec<String>,
    volume: &mut f64,
) {
    println!("key -> {key:?}, value -> {value:?}");

    match value {
        serde_json::Value::Array(a) => {
            for v in a {
                parse_json_data("0", v, base_path, files, positions, repeat, takes, volume);
            }
        }
        serde_json::Value::Bool(b) => match key {
            "repeat" => {
                *repeat = b;
            }
            "dummy" => {}
            _ => {}
        },
        serde_json::Value::Null => {}
        serde_json::Value::Number(n) => match key {
            "skip" => {}
            "volume" => {
                *volume = n.as_f64().unwrap_or(1.0);
            }
            _ => {}
        },
        serde_json::Value::Object(o) => {
            for (k, v) in o {
                parse_json_data(
                    k.as_str(),
                    v,
                    base_path,
                    files,
                    positions,
                    repeat,
                    takes,
                    volume,
                );
            }
        }
        serde_json::Value::String(s) => match key {
            "base_path" => {
                *base_path = s;
            }
            "file" => {
                files.push(s);
            }
            "dummy" => {}
            "position" => {
                positions.push(s);
            }
            "take" => {
                takes.push(s);
            }
            _ => {}
        },
    }
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
/// assert_eq!(time_string_to_seconds("01:23:45").unwrap(), 5025);
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
