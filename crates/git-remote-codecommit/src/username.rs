pub struct Username<'a> {
    pub access_key_id: &'a str,
    pub session_token: Option<&'a str>,
}

impl core::fmt::Display for Username<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.access_key_id)?;
        if let Some(token) = self.session_token {
            f.write_str("%")?;
            f.write_str(token)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_without_token() {
        let s = Username {
            access_key_id: "AKIAIOSFODNN7EXAMPLE",
            session_token: None,
        }
        .to_string();
        assert_eq!(s, "AKIAIOSFODNN7EXAMPLE");
    }

    #[test]
    fn test_with_token() {
        let s = Username {
            access_key_id: "AKIAIOSFODNN7EXAMPLE",
            session_token: Some("IQoJb3JpZ2luX2VjELT//////////EXAMPLE"),
        }
        .to_string();
        assert_eq!(
            s,
            "AKIAIOSFODNN7EXAMPLE%IQoJb3JpZ2luX2VjELT//////////EXAMPLE"
        );
    }
}
