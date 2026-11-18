fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("icon.ico");
        res.set("ProductName", "Better FN");
        res.set("FileDescription", "Better FN - keyboard helper");
        res.set("CompanyName", "тхфа");
        res.set("LegalCopyright", "© 2026 тхфа");
        res.set("OriginalFilename", "better_fn.exe");
        res.compile().unwrap();
    }
}