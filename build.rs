extern crate embed_resource;

fn main() {
    embed_resource::compile("assets/logo.rc", embed_resource::NONE);
    embed_resource::compile("assets/LinkEcho-manifest.rc", embed_resource::NONE);
}