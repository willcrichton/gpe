use std::os;
use std::io::Command;
use std::io::process::InheritFd;

static CUDA_PATH: &'static str = "/Developer/NVIDIA/CUDA-6.5/lib";

fn run(cmd: &mut Command) {
    println!("running: {}", cmd);
    let status = match cmd.stdout(InheritFd(1)).stderr(InheritFd(2)).status() {
        Ok(status) => status,
        Err(e) => panic!("failed to spawn process: {}", e),
    };
    if !status.success() {
        panic!("nonzero exit status: {}", status);
    }
}

fn main() {
    let out_dir = os::getenv("OUT_DIR").unwrap();

    // compile CUDA code into device relocatable code
    // NOTE: if this isn't sm_20, it _WILL_ break
    run(Command::new("nvcc")
        .args(&["-arch=sm_20", "-dc", "src/render.cu", "-o"])
        .arg(format!("{}/render.o", out_dir)));

    // convert relocatable code into executable code
    run(Command::new("nvcc")
        .args(&["-arch=sm_20", "-dlink", format!("{}/render.o", out_dir).as_slice(), "-o"])
        .arg(format!("{}/render_dlink.o", out_dir)));

    // turn it into a static library
    run(Command::new("ar")
        .args(&["crus", "librender.a", "render.o", "render_dlink.o"])
        .cwd(&Path::new(&out_dir)));

    println!("cargo:rustc-flags=-l render:static -L {} -l cudart -L {}", CUDA_PATH, out_dir);
}
