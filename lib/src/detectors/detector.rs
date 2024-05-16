/// Detector marker trait
pub trait Detector {
    fn detect(&self) -> String;
}
