extern crate embed_resource;

fn main() {
    #[cfg(windows)]
    embed_resource::compile("obs-install-manager-manifest.rc", embed_resource::NONE)
        .manifest_required()
        .unwrap();
}
