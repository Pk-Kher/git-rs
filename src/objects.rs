#[derive(Debug)]
pub(crate) enum Kind {
    Blob,
    Tree,
}
impl Kind {
    pub(crate) fn from_mode(mode: &[u8]) -> Result<Self, anyhow::Error> {
        match mode {
            b"40000" => Ok(Kind::Tree),
            b"100644" | b"100755" | b"120000" => Ok(Kind::Blob),
            _ => anyhow::bail!("unknown mode:{:?}", std::str::from_utf8(mode)),
        }
    }

    pub(crate) fn to_str(&self) -> &str {
        match self {
            Kind::Blob => "blob",
            Kind::Tree => "tree",
        }
    }
}
