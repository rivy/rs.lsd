use std::path::{Component, Path, Prefix};

fn main() {
    // Example paths
    let paths = vec![
        r"C:\Windows\System32",
        r"\\server\share\folder\file.txt",
        r"C:\CON",
        r"relative\path\to\file",
        r"\absolute\path\to\file",
        r"\\?\UNC\hoard\server",
    ];

    for path_str in paths {
        let path = Path::new(path_str);
        println!("Path: {}", path.display());

        let mut components = path.components();

        let mut verbatim_path_os = std::ffi::OsString::from(r"\\?\");
        let maybe_prefix = components.next();
        match maybe_prefix {
            Some(std::path::Component::Prefix(prefix)) => {
                eprintln!("~ prefix: {:#?}", prefix);
                if let std::path::Prefix::UNC(server, share) = prefix.kind() {
                    verbatim_path_os.push(r"UNC\");
                    verbatim_path_os.push(server);
                    verbatim_path_os.push(r"\");
                    verbatim_path_os.push(share);
                } else {
                    verbatim_path_os.push(prefix.as_os_str());
                }
            }
            Some(c) => {
                verbatim_path_os.push(c.as_os_str());
            }
            _ => {}
        };
        eprintln!(
            "~ components: {:#?}\n~ verbatim_path_os: {:#?}",
            components, verbatim_path_os
        );

        for component in components {
            match component {
                Component::Prefix(prefix) => match prefix.kind() {
                    Prefix::Verbatim(disk) => println!("  Verbatim disk: {:?}", disk),
                    Prefix::VerbatimUNC(server, share) => {
                        println!(
                            "  Verbatim UNC: \\{}\\{}",
                            server.to_string_lossy(),
                            share.to_string_lossy()
                        )
                    }
                    Prefix::VerbatimDisk(disk) => println!("  Verbatim disk: {:?}", disk),
                    Prefix::DeviceNS(device) => println!("  Device namespace: {:?}", device),
                    Prefix::UNC(server, share) => {
                        println!(
                            "  UNC: \\{}\\{}",
                            server.to_string_lossy(),
                            share.to_string_lossy()
                        )
                    }
                    Prefix::Disk(disk) => println!("  Disk: {:?}", disk),
                },
                Component::RootDir => println!("  Root directory"),
                Component::CurDir => println!("  Current directory"),
                Component::ParentDir => println!("  Parent directory"),
                Component::Normal(segment) => println!("  Normal segment: {:?}", segment),
            }
        }

        println!();
    }
}
