fn main() {
    println!("cargo:rustc-link-search=./assets");
    println!("cargo:rustc-link-lib=static=judger");
    println!("cargo:rustc-link-lib=dylib=seccomp");
    println!("cargo:rustc-link-lib=dylib=pthread");
}
