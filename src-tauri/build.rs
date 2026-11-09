fn main() {
    // MinGW 修复：GNU 链接器在链接 cdylib（*_lib.dll）时会自动导出所有来自静态库的
    // 全局符号，而 PE/COFF 的导出序号是 16 位（上限 65535）。Tauri 依赖树庞大
    // （wry/webview2/rusqlite/reqwest…），会超出上限导致
    // "error: export ordinal too large"。--exclude-libs=ALL 隐藏来自静态库的符号，
    // 只保留本 crate 自己的导出。桌面可执行文件链接的是 rlib/staticlib 产物而非该
    // DLL，因此收窄导出表不影响运行时。仅对 GNU 工具链生效——MSVC 链接器没有
    // 这种自动导出行为，会拒绝该 -Wl 参数。
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu") {
        println!("cargo::rustc-link-arg=-Wl,--exclude-libs=ALL");
    }
    tauri_build::build()
}
