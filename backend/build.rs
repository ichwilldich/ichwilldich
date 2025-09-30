fn main() {
  #[cfg(not(debug_assertions))]
  {
    let frontend_dir = std::env::var("FRONTEND_DIR").expect("FRONTEND_DIR must be set");
    println!("cargo:rerun-if-env-changed=FRONTEND_DIR");
    println!("cargo:rustc-env=FRONTEND_DIR={}", frontend_dir);

    let frontend_port = std::env::var("FRONTEND_PORT").unwrap_or_else(|_| "3000".to_string());
    println!("cargo:rerun-if-env-changed=FRONTEND_PORT");
    println!("cargo:rustc-env=FRONTEND_PORT={}", frontend_port);
  }
}
