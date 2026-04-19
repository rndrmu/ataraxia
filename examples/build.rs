fn main() {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        use std::env;

        let out = env::var("OUT_DIR").unwrap();
        let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
        let src = format!("{}/../ataraxia-voice/src/absl_shim.m", manifest);
        let obj = format!("{}/absl_shim.o", out);

        let sdk = Command::new("xcrun")
            .args(["--sdk", "macosx", "--show-sdk-path"])
            .output()
            .expect("xcrun failed");
        let sdk = String::from_utf8(sdk.stdout).unwrap();
        let sdk = sdk.trim().to_string();

        let status = Command::new("clang")
            .args([
                "-fobjc-arc",
                "-arch", "arm64",
                "-mmacosx-version-min=13.0",
                &format!("-isysroot{sdk}"),
                "-c", &src,
                "-o", &obj,
            ])
            .status()
            .expect("clang compile failed");
        assert!(status.success(), "absl_shim.m compile failed");

        println!("cargo:rustc-link-arg={obj}");
        println!("cargo:rustc-link-arg=-ObjC");
        println!("cargo:rustc-link-lib=framework=Foundation");
    }
}
