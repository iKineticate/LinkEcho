extern crate embed_resource;

fn main() {
    embed_resource::compile("resources/logo.rc", embed_resource::NONE).manifest_optional().unwrap();
    embed_resource::compile("resources/app.manifest.rc", embed_resource::NONE).manifest_optional().unwrap();
}
