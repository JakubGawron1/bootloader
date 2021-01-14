use std::{
    path::{Path, PathBuf},
    process::Command,
};

const QEMU_ARGS: &[&str] = &[
    "-device",
    "isa-debug-exit,iobase=0xf4,iosize=0x04",
    "-serial",
    "stdio",
    "-display",
    "none",
    "--no-reboot",
];

fn main() {
    let kernel_binary_path = {
        let path = PathBuf::from(std::env::args().nth(1).unwrap());
        path.canonicalize().unwrap()
    };

    let disk_image = create_disk_image(&kernel_binary_path, false);

    let mut run_cmd = Command::new("qemu-system-x86_64");
    run_cmd
        .arg("-drive")
        .arg(format!("format=raw,file={}", disk_image.display()));
    run_cmd.args(QEMU_ARGS);
    run_cmd.args(std::env::args().skip(2).collect::<Vec<_>>());

    let exit_status = run_cmd.status().unwrap();
    match exit_status.code() {
        Some(33) => {}                     // success
        Some(35) => panic!("Test failed"), // success
        other => panic!("Test failed with unexpected exit code `{:?}`", other),
    }
}

pub fn create_disk_image(kernel_binary_path: &Path, bios_only: bool) -> PathBuf {
    let bootloader_manifest_path = bootloader_locator::locate_bootloader("bootloader").unwrap();
    let kernel_manifest_path = locate_cargo_manifest::locate_manifest().unwrap();

    let mut build_cmd = Command::new(env!("CARGO"));
    build_cmd.current_dir(bootloader_manifest_path.parent().unwrap());
    build_cmd.arg("builder");
    build_cmd
        .arg("--kernel-manifest")
        .arg(&kernel_manifest_path);
    build_cmd.arg("--kernel-binary").arg(&kernel_binary_path);
    build_cmd
        .arg("--target-dir")
        .arg(kernel_manifest_path.parent().unwrap().join("target"));
    build_cmd
        .arg("--out-dir")
        .arg(kernel_binary_path.parent().unwrap());
    //build_cmd.arg("--quiet");
    if bios_only {
        build_cmd.arg("--firmware").arg("bios");
    }

    if !build_cmd.status().unwrap().success() {
        panic!("build failed");
    }

    let kernel_binary_name = kernel_binary_path.file_name().unwrap().to_str().unwrap();
    let disk_image = kernel_binary_path
        .parent()
        .unwrap()
        .join(format!("bootimage-bios-{}.img", kernel_binary_name));
    if !disk_image.exists() {
        panic!(
            "Disk image does not exist at {} after bootloader build",
            disk_image.display()
        );
    }
    disk_image
}
