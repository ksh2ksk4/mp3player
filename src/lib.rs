use chrono::{NaiveTime, Timelike};

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
