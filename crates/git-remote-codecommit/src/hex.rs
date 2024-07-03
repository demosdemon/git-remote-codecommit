pub struct HexDisplay<T>(T);

impl<T: AsRef<[u8]>> core::fmt::Display for HexDisplay<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for b in self.0.as_ref() {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}

pub trait HexDisplayExt: AsRef<[u8]> {
    fn hex_display(self) -> HexDisplay<Self>
    where
        Self: Sized,
    {
        HexDisplay(self)
    }
}

impl<T: AsRef<[u8]>> HexDisplayExt for T {}
