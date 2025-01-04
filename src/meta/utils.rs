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
    // for WinOS, `std::{fs,path}::absolute()` (as of 2024-12 / v1.85.0) resolves any path with a basename containing a "device" name into a device path (eg, `CON` or `..\CON.txt` -> `\\.\CON`), regardless of the parent path
    // * adding a '/.' suffix to the path will prevent this behavior
    let path_protected = path.join("."); // prevent mis-handling of "device"-like paths by `std::{fs,path}::absolute()`
    return std::path::absolute(path_protected);
}

pub fn to_verbatim_path<P>(path: P) -> std::io::Result<std::path::PathBuf>
where
    P: AsRef<std::path::Path>,
{
    // convert path to a verbatim format (`\\?\...`), avoiding rust std library mis-handling of "device"-similar file paths
    // * eg, `CON` or `./CON` is translated to `\\?\C:\...\CON` (as opposed to the usual rust std library translation to `\\.\CON`)

    // ref: [File path formats](https://learn.microsoft.com/en-us/dotnet/standard/io/file-path-formats) @@ <https://archive.is/0shPL>
    // ref: [Naming Files, Paths, and Namespaces](https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file) @@ <https://archive.is/mQOTg>
    // ref: [WinOS Paths](https://chrisdenton.github.io/omnipath/print.html) @@ <https://archive.is/90Elx>
    //... ref: <https://github.com/rivy-t/rs.omnipath> , <https://github.com/ChrisDenton/omnipath>

    let path = path.as_ref();

    // * short-circuit return for non-WinOS
    if !cfg!(windows) {
        return Ok(path.to_path_buf());
    }

    // * empty paths have no verbatim equivalent
    if path.as_os_str().len() == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "cannot convert an empty path to verbatim format",
        ));
    }

    // // * quick short-circuit if path is already in verbatim format
    // if path.as_os_str().as_encoded_bytes().starts_with(b"\\\\?\\") {
    //     return Ok(path.to_path_buf());
    // }

    let absolute_path = to_absolute_path(path)?;
    // eprintln!("to_verbatim_path() ~ absolute_path: {:#?}", absolute_path);

    // add verbatim prefix (`\\?\`) to path
    // * an intermediary OsString is used to avoid `PathBuf::push()` logic which can otherwise overwrite the prefix with a subsequent absolute path
    let mut components = absolute_path.components();
    // eprintln!("to_verbatim_path()\n~ components: {:#?}", components);

    let component = components.next();
    let verbatim_path_os = match component {
        Some(std::path::Component::Prefix(prefix)) => {
            let mut prefix_os = std::ffi::OsString::from(r"\\?\");
            match prefix.kind() {
                std::path::Prefix::DeviceNS(device) => {
                    prefix_os.push(device);
                    let path = std::path::Path::new(&prefix_os);
                    path.join(components).as_os_str().to_os_string()
                }
                std::path::Prefix::Disk(_disk) => {
                    // prefix_os.push(char::from(disk).to_string());
                    // prefix_os.push(r":");
                    prefix_os.push(prefix.as_os_str());
                    let path = std::path::Path::new(&prefix_os);
                    path.join(components).as_os_str().to_os_string()
                }
                std::path::Prefix::UNC(server, share) => {
                    prefix_os.push(r"UNC\");
                    prefix_os.push(server);
                    prefix_os.push(r"\");
                    prefix_os.push(share);
                    let path = std::path::Path::new(&prefix_os);
                    // valid verbatim paths must have at least one component after the prefix; `\\server\share` *may* sometimes be missing any further components
                    let mut residual_path = components.as_path();
                    if residual_path == std::path::Path::new("") {
                        // * add a `RootDir` if no other components (of `absolute_path`) exist
                        residual_path = std::path::Path::new(r"\");
                    }
                    path.join(residual_path).as_os_str().to_os_string()
                }
                std::path::Prefix::VerbatimDisk(_disk) => absolute_path.as_os_str().to_os_string(),
                std::path::Prefix::VerbatimUNC(_server, _share) => {
                    // valid verbatim paths must have at least one component after the prefix; `\\?\UNC\server\share` *may* be constructed to be missing any further components
                    let path = if components.next().is_none() {
                        // * add a `RootDir` if no other components (of `absolute_path`) exist
                        std::path::Path::new(&absolute_path).join(r"\")
                    } else {
                        absolute_path
                    };
                    path.as_os_str().to_os_string()
                }
                // * unreachable!("all `std::path::Prefix` variants should be handled")
                _ /* includes `std::path::Prefix::Verbatim(_)` */ => absolute_path.as_os_str().to_os_string(),
            }
        }
        _ => unreachable!(
            "first component of absolute path ('{:#?}'; from '{:#?}') is not a std::path::Prefix",
            absolute_path, path
        ),
    };

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

    let shared_components = path_verbatim
        .components()
        .zip(base_path_verbatim.components())
        .take_while(|(target_component, base_component)| target_component == base_component)
        .map(|(component, _)| component);
    let shared_path = shared_components.clone().collect::<std::path::PathBuf>();

    // eprintln!(
    //         "relative_path()\n~ path_verbatim: {:#?}\n~ base_path_verbatim: {:#?}\n~ shared_components: {:#?}\n~ shared_path: {:#?}",
    //         path_verbatim, base_path_verbatim, shared_components, shared_path
    //     );

    base_path_verbatim
        .strip_prefix(&shared_path)
        .unwrap()
        .components()
        .map(|_| std::path::Component::ParentDir)
        .chain(
            path_verbatim
                .strip_prefix(&shared_path)
                .unwrap()
                .components(),
        )
        .collect()
}
