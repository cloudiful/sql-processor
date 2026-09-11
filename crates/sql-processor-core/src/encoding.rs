use encoding_rs::GB18030;

use crate::CoreError;

pub fn decode_gb18030(data: &[u8]) -> Result<String, CoreError> {
    let (decoded, _, had_errors) = GB18030.decode(data);
    if had_errors {
        return Err(CoreError::Decode("invalid GB18030 sequence".to_string()));
    }
    Ok(decoded.into_owned())
}

pub fn ensure_gb2312_memory(path: &str, data: &[u8]) -> Result<(Vec<u8>, Vec<String>), CoreError> {
    let has_bom = data.starts_with(b"\xef\xbb\xbf");
    let is_utf8 = std::str::from_utf8(data).is_ok();
    let has_non_ascii = data.iter().any(|byte| *byte > 127);
    if !has_bom && (!is_utf8 || !has_non_ascii) {
        return Ok((data.to_vec(), Vec::new()));
    }

    let input = if has_bom { &data[3..] } else { data };
    let text = std::str::from_utf8(input).map_err(|error| CoreError::Encode(error.to_string()))?;
    let (encoded, _, had_errors) = GB18030.encode(text);
    if had_errors {
        return Err(CoreError::Encode(
            "input contains characters GB18030 cannot encode".to_string(),
        ));
    }
    Ok((
        encoded.into_owned(),
        vec![format!(
            "File {path} is UTF-8, auto converting to GB2312 in memory..."
        )],
    ))
}
