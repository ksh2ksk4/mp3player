use chrono::{NaiveTime, Timelike};
use std::fs::File;
use std::io::Read;

/// # Summary
///
/// ファイルから JSON データを読み込み、その JSON データを返す
///
/// # Arguments
///
/// - `file`: JSON ファイル
pub fn read_json_data(file: String) -> Result<serde_json::Value, serde_json::Error> {
    match File::open(file) {
        Ok(mut f) => {
            let mut contents = String::new();

            f.read_to_string(&mut contents).unwrap();
            serde_json::from_str::<serde_json::Value>(&contents)
        }
        Err(e) => {
            println!("e -> {e:?}");
            panic!("{}", e.to_string());
        }
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
