pub struct InferredHostname<'a> {
    region: &'a str,
}

impl<'a> InferredHostname<'a> {
    pub fn new(region: &'a str) -> Self {
        Self { region }
    }
}

impl core::fmt::Display for InferredHostname<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self { region } = self;
        let partition = if region.starts_with("cn-") {
            "amazonaws.com.cn"
        } else {
            "amazonaws.com"
        };
        write!(f, "git-codecommit.{region}.{partition}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cn() {
        let s = InferredHostname::new("cn-north-1").to_string();
        assert_eq!(s, "git-codecommit.cn-north-1.amazonaws.com.cn");
    }

    #[test]
    fn test_iad() {
        let s = InferredHostname::new("us-east-1").to_string();
        assert_eq!(s, "git-codecommit.us-east-1.amazonaws.com");
    }
}
