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
