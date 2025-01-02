// use std::io;
// use std::path;

pub fn to_absolute_path<P>(path: P) -> std::io::Result<std::path::PathBuf>
where
    P: AsRef<std::path::Path>,
{
    let path = path.as_ref();
    if !cfg!(windows) {
        // non-WinOS ~ use `std::path::absolute()`` without any modifications
        return std::path::absolute(path);
    }
    // WinOS
    // * std::{fs,path}::absolute() resolves any path with a basename containing a "device" name into a device path (eg, `CON` or `..\CON.txt` -> `\\.\CON`), regardless of the parent path
    let path_protected = path.join("."); // "protect" path from being mis-handled by rust std library `std::path::absolute()`
    return std::path::absolute(path_protected);
}

pub fn to_verbatim_path<P>(path: P) -> std::io::Result<std::path::PathBuf>
where
    P: AsRef<std::path::Path>,
{
    // convert path to a verbatim format (`\\?\...`), avoiding rust std library mis-handling of files resembling device paths
    // * eg, `CON` or `./CON` is translated to `\\?\C:\...\CON` (as opposed to the usual rust std library translation to `\\.\CON`)

    // ref: [File path formats](https://learn.microsoft.com/en-us/dotnet/standard/io/file-path-formats) @@ <https://archive.is/0shPL>
    // ref: [Naming Files, Paths, and Namespaces](https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file) @@ <https://archive.is/mQOTg>
    // ref: [WinOS Paths](https://chrisdenton.github.io/omnipath/print.html) @@ <https://archive.is/90Elx>
    //... ref: <https://github.com/rivy-t/rs.omnipath> , <https://github.com/ChrisDenton/omnipath>

    let path = path.as_ref();

    // * short-circuit for non-WinOS
    if !cfg!(windows) {
        return Ok(path.to_path_buf());
    }

    // * quick short-circuit if path is already in verbatim format
    if path.as_os_str().as_encoded_bytes().starts_with(b"\\\\?\\") {
        return Ok(path.to_path_buf());
    }

    let absolute_path = to_absolute_path(path)?;
    // eprintln!("to_verbatim_path() ~ absolute_path: {:#?}", absolute_path);

    // not needed? ~ `std::path::absolute()` should already never return a de-novo verbatim path
    // // avoid forcing verbatim prefix onto paths that already have it
    // match absolute_path.components().nth(0) {
    //     Some(std::path::Component::Prefix(c)) if c.kind().is_verbatim() => {
    //         return Ok(absolute_path);
    //     }
    //     _ => {}
    // };

    // add verbatim prefix (`\\?\`) to path
    // * an intermediary OsString is used to avoid `PathBuf::push()` logic which will otherwise overwrite the prefix with a subsequent absolute path
    let mut verbatim_path_os = std::ffi::OsString::from(r"\\?\");
    let mut components = absolute_path.components();

    // * special handling is required for paths with a UNC (`\\server\share`) prefix
    if let Some(std::path::Component::Prefix(prefix)) = components.next() {
        if let std::path::Prefix::UNC(server, share) = prefix.kind() {
            verbatim_path_os.push(r"UNC\");
            verbatim_path_os.push(server);
            verbatim_path_os.push(r"\");
            verbatim_path_os.push(share);
        } else {
            verbatim_path_os.push(prefix.as_os_str());
        }
    }

    verbatim_path_os.push(components);

    let verbatim_path = std::path::PathBuf::from(verbatim_path_os);
    // eprintln!("to_verbatim_path() ~ verbatim_path: {:#?}", verbatim_path);
    return Ok(verbatim_path);
}

pub fn relative_path<P1, P2>(path: P1, base_path: P2) -> std::path::PathBuf
where
    P1: AsRef<std::path::Path>,
    P2: AsRef<std::path::Path>,
{
    let path = path.as_ref();
    let base_path = base_path.as_ref();
    // eprintln!(
    //     "relative_path()\n~ path: {:#?}\n~ base_path: {:#?}",
    //     path, base_path,
    // );

    // if (self.path == base_path) || (self.path == *PATHBUF_CURRENT_DIR) {
    if path == base_path {
        return std::path::PathBuf::from(AsRef::<std::path::Path>::as_ref(
            &std::path::Component::CurDir,
        ));
    }

    let path_verbatim = to_verbatim_path(&path).expect("failed to convert self.path to verbatim");

    let base_path_verbatim =
        to_verbatim_path(&base_path).expect("failed to convert base_path to verbatim");

    if path_verbatim == base_path_verbatim {
        return std::path::PathBuf::from(AsRef::<std::path::Path>::as_ref(
            &std::path::Component::CurDir,
        ));
    }

    let shared_components: std::path::PathBuf = path_verbatim
        .components()
        .zip(base_path_verbatim.components())
        .take_while(|(target_component, base_component)| target_component == base_component)
        .map(|tuple| tuple.0)
        .collect();

    // eprintln!(
    //         "relative_path()\n~ path_verbatim: {:#?}\n~ base_path_verbatim: {:#?}\n~ shared_components: {:#?}",
    //         path_verbatim, base_path_verbatim, shared_components
    //     );

    base_path_verbatim
        .strip_prefix(&shared_components)
        .unwrap()
        .components()
        .map(|_| std::path::Component::ParentDir)
        .chain(
            path_verbatim
                .strip_prefix(&shared_components)
                .unwrap()
                .components(),
        )
        .collect()
}
