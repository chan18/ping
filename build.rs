

fn main() {
        // Add the path to the Npcap SDK Lib folder
    println!("cargo:rustc-link-search=native=H:\\Downloads\\npcap-sdk-1.13\\Lib\\x64");

    // Specify the library to link
    println!("cargo:rustc-link-lib=dylib=wpcap");
}