#[macro_export]
macro_rules! path {
  () => {
    std::path::PathBuf::new()
  };
  ($($segment:expr),+ $(,)?) => {
    {
      let mut path = std::path::PathBuf::new();
      $(
        path.push($segment);
      )+
      path
    }
  };
}
