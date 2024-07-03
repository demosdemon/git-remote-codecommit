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
