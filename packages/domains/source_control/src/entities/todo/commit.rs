use thiserror::Error;

pub struct Commit {
    pub sha: CommitSha,
    pub message: String,
    pub previous_commit: Option<Box<Commit>>
}

#[derive(Clone, Copy, Debug)]
pub struct CommitSha(pub [u8; 20]);

impl CommitSha {
    pub fn from_string(sha: &str) -> Result<Self, StringToShaError> {
        // Ensure the input is a valid 40-character hexadecimal string
        if sha.len() != 40 {
            return Err(StringToShaError::WrongLengthString);
        }

        // Use the hex crate to decode the string into bytes
        match hex::decode(sha) {
            Ok(decoded) => {
                // Check that the decoded result is exactly 20 bytes
                if decoded.len() == 20 {
                    let mut byte_array = [0u8; 20];
                    byte_array.copy_from_slice(&decoded);
                    Ok(CommitSha(byte_array))
                } else {
                    Err(StringToShaError::WrongLengthString)
                }
            }
            Err(_) => Err(StringToShaError::InvalidHexString),
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum StringToShaError {
    #[error("Invalid hexadecimal string")]
    InvalidHexString,
    #[error("String SHA should be a 40-character hexadecimal string")]
    WrongLengthString,
}

#[cfg(test)]
mod test {
    use crate::entities::commit::StringToShaError;

    use super::CommitSha;

    #[test]
    fn should_decode_valid_sha_string() {
        let str = "5da1da398896dae624e6403cd9073b392ec448b7";
        let sha = CommitSha::from_string(str);

        assert!(sha.is_ok());

        let commit_sha = sha.unwrap();

        assert_eq!(
            commit_sha.0,
            [
                93, 161, 218, 57, 136, 150, 218, 230, 36, 230, 64, 60, 217, 7, 59, 57, 46, 196, 72,
                183
            ]
        )
    }

    #[test]
    fn should_fail_for_too_long_string() {
        let str = "5da1da398896dae624e6403cd9073b392ec448b77";
        let sha = CommitSha::from_string(str);

        assert!(sha.is_err());

        let error = sha.unwrap_err();

        assert_eq!(error, StringToShaError::WrongLengthString)
    }

    #[test]
    fn should_fail_for_non_hex() {
        let str = "5da1da398896dae624e6403cd9073b392ec448bg";
        let sha = CommitSha::from_string(str);

        assert!(sha.is_err());

        let error = sha.unwrap_err();

        assert_eq!(error, StringToShaError::InvalidHexString)
    }
}
