use chrono::Local;

fn main() {
    let local = Local::now().format("-%Y.%m.%d.%H.%M.%S");
    println!("cargo:rustc-env=ASTRAL_COMPILE_DATETIME={}", local);
}
