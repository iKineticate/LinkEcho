fn main() {
    if !cfg!(debug_assertions) {
        embed_resource::compile("assets/app.manifest.rc", embed_resource::NONE)
            .manifest_optional()
            .unwrap();
    }

    windows_reactor_setup::as_framework_dependent();
}
