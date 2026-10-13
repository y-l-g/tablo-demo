fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=styles.css");
    tablo_build::tailwind().expect("build the Tailwind stylesheet");
}
