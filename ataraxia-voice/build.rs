fn main() {
    // Inject the missing Abseil ObjC shim on macOS — see src/absl_shim.m
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        use std::path::PathBuf;

        let out = std::env::var("OUT_DIR").unwrap();
        let src = "src/absl_shim.m";
        let obj = format!("{}/absl_shim.o", out);

        // Compile to object file (not archive) so the __objc_catlist section survives
        let sdk = Command::new("xcrun")
            .args(["--sdk", "macosx", "--show-sdk-path"])
            .output()
            .expect("xcrun failed")
            .stdout;
        let sdk = std::str::from_utf8(&sdk).unwrap().trim();

        let status = Command::new("clang")
            .args([
                "-fobjc-arc",
                "-arch", "arm64",
                "-mmacosx-version-min=13.0",
                &format!("-isysroot{sdk}"),
                "-c", src,
                "-o", &obj,
            ])
            .status()
            .expect("clang failed");
        assert!(status.success(), "absl_shim.m compile failed");

        // Tell the linker to include the object file and
        // use -ObjC so all ObjC sections are loaded
        println!("cargo:rustc-link-arg={obj}");
        println!("cargo:rustc-link-arg=-ObjC");
        println!("cargo:rustc-link-lib=framework=Foundation");
    }
}
