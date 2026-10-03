fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/monolai.ico");
        res.set("ProductName", "Monolai");
        res.set("FileDescription", "Monolai Local AI Desktop");
        if let Err(e) = res.compile() {
            eprintln!("Failed to compile Windows resources: {}", e);
        }
    }
}
