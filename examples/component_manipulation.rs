use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, Prefix};

struct UncPath {
    _server: Box<[u8]>,
    _share: Box<[u8]>,
    path_os: Box<OsString>,
}

impl UncPath {
    fn new(server: &OsStr, share: &OsStr) -> Self {
        let mut path_os = OsString::from(r"\\?\UNC\");
        path_os.push(server);
        path_os.push(r"\");
        path_os.push(share);
        Self {
            _server: server.as_encoded_bytes().into(),
            _share: share.as_encoded_bytes().into(),
            path_os: Box::new(path_os),
        }
    }

    fn to_component(&self) -> Component<'static> {
        let path_str = Box::leak(self.path_os.clone().into_boxed_os_str());
        let path = Path::new(path_str);
        path.components().next().unwrap()
    }

    fn _server(&self) -> &OsStr {
        unsafe { OsStr::from_encoded_bytes_unchecked(&self._server) }
    }
    fn _share(&self) -> &OsStr {
        unsafe { OsStr::from_encoded_bytes_unchecked(&self._share) }
    }
    fn _path_os(&self) -> OsString {
        (*self.path_os).clone()
    }
}

fn create_unc_prefix(server: &OsStr, share: &OsStr) -> Component<'static> {
    UncPath::new(server, share).to_component()
}
fn main() {
    let path = Path::new(r"\\server\share\Users\Roy\Documents\file.txt");
    println!("Original path: {}", path.display());

    let components: Vec<Component> = path.components().collect();
    println!("Original components: {:?}", components);

    let modified_components: Vec<Component> = components
        .into_iter()
        .map(|component| match component {
            Component::Prefix(prefix) => match prefix.kind() {
                Prefix::UNC(server, share) => create_unc_prefix(server, share),
                _ => Component::Prefix(prefix),
            },
            _ => component,
        })
        .collect();

    println!("Modified components: {:?}", modified_components);
}
